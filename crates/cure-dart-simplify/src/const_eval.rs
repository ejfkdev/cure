//! Dart 常量表达式求值：只接受不依赖运行时、且在 VM/Web 上语义一致的域。
//!
//! 安全域：32 位有符号整数、bool、简单字符串。`/`（double）、浮点、
//! BigInt 边界、`is`/`as`、可空值一律返回“未知”。求值器同时负责
//! 消费完整表达式，因此调用方可以安全地替换整个 token 区间。

use cure_dart_ast::{simple_string, DartAst, TokKind};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Const {
    Int(i64),
    Bool(bool),
    Str(String),
}

pub struct Eval<'a> {
    pub ast: &'a DartAst,
    pub i: usize,
    pub end: usize,
    min_bin: u8,
    ternary: bool,
}

pub struct EvalResult {
    pub value: Const,
    pub end: usize,
    /// 该区间内被消费的二元运算符的最低优先级；没有则为 u8::MAX。
    pub min_prec: u8,
    /// 区间内是否包含（未被括号隔开的）三元表达式。
    pub ternary: bool,
}

pub fn eval_range(ast: &DartAst, start: usize, end: usize) -> Option<(Const, usize)> {
    let r = eval_range_tracked(ast, start, end)?;
    Some((r.value, r.end))
}

pub fn eval_range_tracked(ast: &DartAst, start: usize, end: usize) -> Option<EvalResult> {
    let mut e = Eval {
        ast,
        i: start,
        end,
        min_bin: u8::MAX,
        ternary: false,
    };
    let (v, ok) = e.parse_conditional();
    if !ok {
        return None;
    }
    Some(EvalResult {
        value: v?,
        end: e.i,
        min_prec: e.min_bin,
        ternary: e.ternary,
    })
}

/// `+`/`-` 是否是一元前缀（上一 token 不能结束一个表达式）。
fn sign_is_prefix(ast: &DartAst, op_i: usize) -> bool {
    match ast.prev_sig(op_i) {
        None => true,
        Some(p) => {
            if ast.tokens[p].kind.is_word() {
                matches!(
                    ast.text(p),
                    "return"
                        | "throw"
                        | "yield"
                        | "await"
                        | "case"
                        | "when"
                        | "in"
                        | "is"
                        | "as"
                        | "assert"
                        | "else"
                        | "do"
                        | "var"
                        | "final"
                        | "const"
                        | "late"
                        | "static"
                        | "required"
                        | "covariant"
                        | "factory"
                        | "get"
                        | "set"
                        | "operator"
                        | "external"
                        | "abstract"
                )
            } else {
                !matches!(
                    ast.text(p),
                    ")" | "]"
                        | "}"
                        | ">"
                        | ">>"
                        | ">>>"
                        | "!"
                        | "++"
                        | "--"
                        | "."
                        | "?."
                        | ".."
                        | "?.."
                        | "@"
                        | "#"
                        | "..."
                        | "...?"
                )
            }
        }
    }
}

/// 从某个操作数开始折叠时，前一个 token 的上下文是否允许。
/// 例如 `a - 1 + 2` 中从 `1` 折叠 `1 + 2` 会改变左结合分组，必须拒绝。
pub fn context_allows(ast: &DartAst, start: usize, r: &EvalResult) -> bool {
    let Some(p) = ast.prev_sig(start) else {
        return true;
    };
    let prev = ast.text(p);
    if matches!(prev, "!" | "~" | "await") {
        // 一元前缀的操作数不能再吃掉任何二元/三元运算符。
        return !r.ternary && r.min_prec == u8::MAX;
    }
    if matches!(prev, "+" | "-") && sign_is_prefix(ast, p) {
        return !r.ternary && r.min_prec == u8::MAX;
    }
    let Some((prec, right_assoc)) = bin_prec(prev) else {
        return true;
    };
    if r.ternary {
        return false;
    }
    if right_assoc {
        r.min_prec >= prec
    } else {
        r.min_prec > prec
    }
}

impl<'a> Eval<'a> {
    fn sig(&self, mut i: usize) -> Option<usize> {
        while i < self.end {
            if !self.ast.tokens[i].is_comment() {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    fn peek(&self) -> Option<usize> {
        self.sig(self.i)
    }

    fn text_at(&self, i: usize) -> &str {
        self.ast.text(i)
    }

    fn consume_sig(&mut self) -> Option<usize> {
        let i = self.peek()?;
        self.i = i + 1;
        Some(i)
    }

    fn parse_conditional(&mut self) -> (Option<Const>, bool) {
        // 不在常量折叠里消费三元分支：`true ? 1 : x` 之类只折条件会丢弃
        // 另一个分支（可能含赋值/影响静态流分析）。三元选择交给
        // `dart_ternary_const`，它要求两分支是同类字面量。
        self.parse_bin(0)
    }

    fn parse_bin(&mut self, min_prec: u8) -> (Option<Const>, bool) {
        let (mut left, ok) = self.parse_unary();
        if !ok {
            return (None, false);
        }
        loop {
            let Some(op_i) = self.peek() else {
                break;
            };
            let op = self.text_at(op_i).to_string();
            if matches!(op.as_str(), "is" | "as") {
                let prec = 5;
                if prec < min_prec {
                    break;
                }
                self.consume_sig();
                if !self.skip_type() {
                    return (None, false);
                }
                left = None;
                continue;
            }
            let Some((prec, right_assoc)) = bin_prec(&op) else {
                break;
            };
            if prec < min_prec {
                break;
            }
            self.min_bin = self.min_bin.min(prec);
            self.consume_sig();
            let (right, ok2) = self.parse_bin(if right_assoc { prec } else { prec + 1 });
            if !ok2 {
                return (None, false);
            }
            left = apply_bin(&op, left, right);
        }
        (left, true)
    }

    fn parse_unary(&mut self) -> (Option<Const>, bool) {
        let Some(i) = self.peek() else {
            return (None, false);
        };
        match self.text_at(i) {
            "-" | "+" | "~" | "!" => {
                self.consume_sig();
                let (v, ok) = self.parse_unary();
                if !ok {
                    return (None, false);
                }
                let v = match (self.text_at(i), v) {
                    ("-", Some(Const::Int(n))) if in_i32(n) && in_i32(-n) => Some(Const::Int(-n)),
                    ("+", Some(Const::Int(n))) if in_i32(n) => Some(Const::Int(n)),
                    ("~", Some(Const::Int(n))) if in_i32(n) => Some(Const::Int(!n)),
                    ("!", Some(Const::Bool(b))) => Some(Const::Bool(!b)),
                    _ => None,
                };
                (v, true)
            }
            "await" | "throw" => {
                self.consume_sig();
                let (_, ok) = self.parse_unary();
                (None, ok)
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> (Option<Const>, bool) {
        let Some(i) = self.peek() else {
            return (None, false);
        };
        let kind = self.ast.tokens[i].kind;
        let text = self.text_at(i).to_string();
        let mut value = match kind {
            TokKind::Number => {
                self.consume_sig();
                int_literal(&text).map(Const::Int)
            }
            TokKind::Str => {
                self.consume_sig();
                simple_string(&text).map(Const::Str)
            }
            TokKind::Ident if text == "true" || text == "false" => {
                self.consume_sig();
                Some(Const::Bool(text == "true"))
            }
            TokKind::Ident if text == "null" => {
                self.consume_sig();
                None
            }
            TokKind::Ident if text == "new" || text == "const" => {
                self.consume_sig();
                let (_, ok) = self.parse_primary();
                if !ok {
                    return (None, false);
                }
                None
            }
            TokKind::Ident if text == "switch" => {
                self.consume_sig();
                if let Some(p) = self.peek() {
                    if self.text_at(p) == "(" {
                        if let Some(close) = self.ast.matching(p) {
                            self.i = close + 1;
                        } else {
                            return (None, false);
                        }
                    }
                }
                if let Some(b) = self.peek() {
                    if self.text_at(b) == "{" {
                        if let Some(close) = self.ast.matching(b) {
                            self.i = close + 1;
                        } else {
                            return (None, false);
                        }
                    }
                }
                None
            }
            TokKind::Ident => {
                self.consume_sig();
                None
            }
            TokKind::Punct if text == "(" => {
                let Some(close) = self.ast.matching(i) else {
                    return (None, false);
                };
                if close >= self.end {
                    return (None, false);
                }
                let inner = eval_range(self.ast, i + 1, close);
                self.i = close + 1;
                match inner {
                    Some((v, used)) if used == close => Some(v),
                    _ => None,
                }
            }
            TokKind::Punct if text == "[" || text == "{" => {
                let Some(close) = self.ast.matching(i) else {
                    return (None, false);
                };
                if close >= self.end {
                    return (None, false);
                }
                self.i = close + 1;
                None
            }
            _ => return (None, false),
        };

        // 后缀链：成员/调用/下标/空断言/泛型实参都会让常量变成未知。
        loop {
            let Some(j) = self.peek() else {
                break;
            };
            match self.text_at(j) {
                "." => {
                    self.consume_sig();
                    let Some(name_i) = self.consume_sig() else {
                        return (None, false);
                    };
                    let name = self.text_at(name_i).to_string();
                    let mut args: Option<Vec<Const>> = None;
                    let mut called = false;
                    if self.peek().is_some_and(|k| self.text_at(k) == "(") {
                        let open = self.peek().unwrap();
                        let Some(close) = self.ast.matching(open) else {
                            return (None, false);
                        };
                        if close >= self.end {
                            value = None;
                            break;
                        }
                        args = self.const_args(open + 1, close);
                        called = true;
                        self.i = close + 1;
                    }
                    value = apply_member(value, &name, args.as_deref(), called);
                }
                "?." => {
                    self.consume_sig();
                    if self.consume_sig().is_none() {
                        return (None, false);
                    }
                    value = None;
                }
                "!" | "++" | "--" => {
                    self.consume_sig();
                    value = None;
                }
                "[" | "(" => {
                    let Some(close) = self.ast.matching(j) else {
                        return (None, false);
                    };
                    if close >= self.end {
                        break;
                    }
                    self.i = close + 1;
                    value = None;
                }
                "<" => {
                    if !self.skip_type_args() {
                        break;
                    }
                    value = None;
                }
                _ => break,
            }
        }
        (value, true)
    }

    /// 解析调用实参：每个实参必须是可求值常量；未知实参返回 None
    /// （调用方仍会把整个调用消费掉，只保留“未知”结果）。
    fn const_args(&mut self, open: usize, close: usize) -> Option<Vec<Const>> {
        let mut out = Vec::new();
        let mut i = open;
        loop {
            while i < close && self.ast.tokens[i].is_comment() {
                i += 1;
            }
            if i >= close {
                break;
            }
            let mut j = i;
            while j < close {
                match self.text_at(j) {
                    "(" | "[" | "{" => {
                        j = self.ast.matching(j)? + 1;
                        continue;
                    }
                    ")" | "]" | "}" => return None,
                    "," => break,
                    _ => {}
                }
                j += 1;
            }
            if j > close {
                return None;
            }
            let (v, used) = eval_range(self.ast, i, j)?;
            if used != j {
                return None;
            }
            out.push(v);
            if j >= close {
                break;
            }
            i = j + 1;
        }
        Some(out)
    }

    fn skip_type_args(&mut self) -> bool {
        let Some(open) = self.peek() else {
            return false;
        };
        if self.text_at(open) != "<" {
            return false;
        }
        let mut depth = 0i32;
        let mut j = open;
        while j < self.end {
            let t = &self.ast.tokens[j];
            if t.is_comment() {
                j += 1;
                continue;
            }
            match t.text.as_ref() {
                "<" => depth += 1,
                ">" => {
                    depth -= 1;
                    if depth == 0 {
                        self.i = j + 1;
                        return true;
                    }
                }
                ">>" => {
                    depth -= 2;
                    if depth <= 0 {
                        self.i = j + 1;
                        return true;
                    }
                }
                ">>>" => {
                    depth -= 3;
                    if depth <= 0 {
                        self.i = j + 1;
                        return true;
                    }
                }
                _ if t.kind == TokKind::Ident
                    || t.kind == TokKind::Number
                    || matches!(t.text.as_ref(), "." | "," | "?" | "*" | "(" | ")" | "[" | "]") =>
                {}
                _ => return false,
            }
            j += 1;
        }
        false
    }

    fn skip_type(&mut self) -> bool {
        let Some(i) = self.peek() else {
            return false;
        };
        if self.text_at(i) == "(" {
            let Some(close) = self.ast.matching(i) else {
                return false;
            };
            if close >= self.end {
                return false;
            }
            self.i = close + 1;
            return true;
        }
        if self.ast.tokens[i].kind != TokKind::Ident {
            return false;
        }
        self.consume_sig();
        if self.peek().is_some_and(|j| self.text_at(j) == "<") {
            if !self.skip_type_args() {
                return false;
            }
        }
        while self.peek().is_some_and(|j| self.text_at(j) == "?") {
            self.consume_sig();
        }
        true
    }
}

fn bin_prec(op: &str) -> Option<(u8, bool)> {
    Some(match op {
        "??" => (1, true),
        "||" => (2, false),
        "&&" => (3, false),
        "==" | "!=" => (4, false),
        "<" | ">" | "<=" | ">=" => (5, false),
        "|" => (6, false),
        "^" => (7, false),
        "&" => (8, false),
        "<<" | ">>" | ">>>" => (9, false),
        "+" | "-" => (10, false),
        "*" | "/" | "%" | "~/" => (11, false),
        _ => return None,
    })
}

fn in_i32(v: i64) -> bool {
    (i32::MIN as i64..=i32::MAX as i64).contains(&v)
}

fn int_literal(text: &str) -> Option<i64> {
    let s: String = text.chars().filter(|&c| c != '_').collect();
    if s.len() > 1 && (s.starts_with("0x") || s.starts_with("0X")) {
        return i64::from_str_radix(&s[2..], 16).ok();
    }
    if s.len() > 1 && (s.starts_with("0b") || s.starts_with("0B")) {
        return i64::from_str_radix(&s[2..], 2).ok();
    }
    if s.contains('.') || s.contains('e') || s.contains('E') {
        return None;
    }
    s.parse().ok()
}

fn apply_bin(op: &str, a: Option<Const>, b: Option<Const>) -> Option<Const> {
    let (a, b) = (a?, b?);
    match (op, a, b) {
        ("+", Const::Int(x), Const::Int(y)) if in_i32(x) && in_i32(y) => {
            let z = x.checked_add(y)?;
            in_i32(z).then_some(Const::Int(z))
        }
        ("-", Const::Int(x), Const::Int(y)) if in_i32(x) && in_i32(y) => {
            let z = x.checked_sub(y)?;
            in_i32(z).then_some(Const::Int(z))
        }
        ("*", Const::Int(x), Const::Int(y)) if in_i32(x) && in_i32(y) => {
            let z = x.checked_mul(y)?;
            in_i32(z).then_some(Const::Int(z))
        }
        ("~/", Const::Int(x), Const::Int(y)) if in_i32(x) && in_i32(y) && y != 0 => {
            let z = x / y;
            in_i32(z).then_some(Const::Int(z))
        }
        ("%", Const::Int(x), Const::Int(y)) if in_i32(x) && in_i32(y) && y != 0 => {
            let mut r = x % y;
            if r < 0 {
                r += y.abs();
            }
            in_i32(r).then_some(Const::Int(r))
        }
        ("<<", Const::Int(x), Const::Int(y))
            if in_i32(x) && (0..=30).contains(&y) =>
        {
            let z = x.checked_shl(y as u32)?;
            in_i32(z).then_some(Const::Int(z))
        }
        (">>", Const::Int(x), Const::Int(y)) if in_i32(x) && (0..=30).contains(&y) => {
            Some(Const::Int(x >> y))
        }
        (">>>", Const::Int(x), Const::Int(y))
            if in_i32(x) && x >= 0 && (0..=30).contains(&y) =>
        {
            Some(Const::Int(x >> y))
        }
        ("&", Const::Int(x), Const::Int(y)) if in_i32(x) && in_i32(y) => {
            Some(Const::Int(x & y))
        }
        ("|", Const::Int(x), Const::Int(y)) if in_i32(x) && in_i32(y) => {
            Some(Const::Int(x | y))
        }
        ("^", Const::Int(x), Const::Int(y)) if in_i32(x) && in_i32(y) => {
            Some(Const::Int(x ^ y))
        }
        ("+", Const::Str(x), Const::Str(y)) => Some(Const::Str(format!("{x}{y}"))),
        ("==", a, b) if same_kind(&a, &b) && comparable_int(&a) => Some(Const::Bool(a == b)),
        ("!=", a, b) if same_kind(&a, &b) && comparable_int(&a) => Some(Const::Bool(a != b)),
        ("<", Const::Int(x), Const::Int(y)) if in_i32(x) && in_i32(y) => {
            Some(Const::Bool(x < y))
        }
        ("<=", Const::Int(x), Const::Int(y)) if in_i32(x) && in_i32(y) => {
            Some(Const::Bool(x <= y))
        }
        (">", Const::Int(x), Const::Int(y)) if in_i32(x) && in_i32(y) => {
            Some(Const::Bool(x > y))
        }
        (">=", Const::Int(x), Const::Int(y)) if in_i32(x) && in_i32(y) => {
            Some(Const::Bool(x >= y))
        }
        ("&&", Const::Bool(x), Const::Bool(y)) => Some(Const::Bool(x && y)),
        ("||", Const::Bool(x), Const::Bool(y)) => Some(Const::Bool(x || y)),
        _ => None,
    }
}

fn comparable_int(c: &Const) -> bool {
    match c {
        Const::Int(n) => in_i32(*n),
        _ => true,
    }
}

fn same_kind(a: &Const, b: &Const) -> bool {
    matches!(
        (a, b),
        (Const::Int(_), Const::Int(_))
            | (Const::Bool(_), Const::Bool(_))
            | (Const::Str(_), Const::Str(_))
    )
}

fn one_int(args: Option<&[Const]>) -> Option<i64> {
    match args? {
        [Const::Int(n)] => Some(*n),
        _ => None,
    }
}

fn one_str<'a>(args: Option<&'a [Const]>) -> Option<&'a str> {
    match args? {
        [Const::Str(s)] => Some(s),
        _ => None,
    }
}

fn empty_args(args: Option<&[Const]>) -> bool {
    args.is_some_and(|a| a.is_empty())
}

fn utf16_find(hay: &[u16], needle: &[u16]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

fn radix_string(n: i64, radix: u32) -> String {
    let neg = n < 0;
    let mut v = n.unsigned_abs();
    let mut digits = Vec::new();
    loop {
        let d = (v % radix as u64) as u32;
        digits.push(char::from_digit(d, radix).unwrap_or('0'));
        v /= radix as u64;
        if v == 0 {
            break;
        }
    }
    if neg {
        digits.push('-');
    }
    digits.iter().rev().collect()
}

/// 内建不可变值上的确定性成员：字符串/整数的只读属性与纯方法。
/// 只折叠与 SDK 版本无关、且不依赖区域设置的结果。
fn apply_member(
    value: Option<Const>,
    name: &str,
    args: Option<&[Const]>,
    called: bool,
) -> Option<Const> {
    let v = value?;
    match (&v, name, called) {
        (Const::Str(s), "length", false) => {
            let n = s.encode_utf16().count() as i64;
            in_i32(n).then_some(Const::Int(n))
        }
        (Const::Str(s), "isEmpty", false) => Some(Const::Bool(s.is_empty())),
        (Const::Str(s), "isNotEmpty", false) => Some(Const::Bool(!s.is_empty())),
        (Const::Str(s), "codeUnitAt", true) => {
            let i = one_int(args)?;
            if !(0..=i32::MAX as i64).contains(&i) {
                return None;
            }
            let units: Vec<u16> = s.encode_utf16().collect();
            units.get(i as usize).map(|u| Const::Int(*u as i64))
        }
        (Const::Str(s), "contains", true) => {
            let needle = one_str(args)?;
            let hay: Vec<u16> = s.encode_utf16().collect();
            let needle: Vec<u16> = needle.encode_utf16().collect();
            Some(Const::Bool(utf16_find(&hay, &needle).is_some()))
        }
        (Const::Str(s), "startsWith", true) => {
            let needle = one_str(args)?;
            let hay: Vec<u16> = s.encode_utf16().collect();
            let needle: Vec<u16> = needle.encode_utf16().collect();
            Some(Const::Bool(hay.starts_with(&needle)))
        }
        (Const::Str(s), "endsWith", true) => {
            let needle = one_str(args)?;
            let hay: Vec<u16> = s.encode_utf16().collect();
            let needle: Vec<u16> = needle.encode_utf16().collect();
            Some(Const::Bool(hay.ends_with(&needle)))
        }
        (Const::Str(s), "indexOf", true) => {
            let needle = one_str(args)?;
            let hay: Vec<u16> = s.encode_utf16().collect();
            let needle: Vec<u16> = needle.encode_utf16().collect();
            let pos = utf16_find(&hay, &needle).map(|p| p as i64).unwrap_or(-1);
            Some(Const::Int(pos))
        }
        (Const::Str(s), "toString", true) if empty_args(args) => {
            Some(Const::Str(s.clone()))
        }
        (Const::Int(n), "isEven", false) => Some(Const::Bool(n % 2 == 0)),
        (Const::Int(n), "isOdd", false) => Some(Const::Bool(n % 2 != 0)),
        (Const::Int(n), "isNegative", false) => Some(Const::Bool(*n < 0)),
        (Const::Int(n), "sign", false) => Some(Const::Int(n.signum())),
        (Const::Int(n), "bitLength", false) if *n >= 0 => {
            let bits = 64 - (*n as u64).leading_zeros() as i64;
            Some(Const::Int(bits))
        }
        (Const::Int(n), "abs", true) if empty_args(args) => {
            let a = n.checked_abs()?;
            in_i32(a).then_some(Const::Int(a))
        }
        (Const::Int(n), "toInt", true) if empty_args(args) => Some(Const::Int(*n)),
        (Const::Int(n), "toString", true) if empty_args(args) => {
            Some(Const::Str(n.to_string()))
        }
        (Const::Int(n), "toRadixString", true) => {
            let radix = one_int(args)?;
            if !(2..=36).contains(&radix) {
                return None;
            }
            Some(Const::Str(radix_string(*n, radix as u32)))
        }
        (Const::Bool(b), "toString", true) if empty_args(args) => {
            Some(Const::Str(if *b { "true" } else { "false" }.to_string()))
        }
        _ => None,
    }
}

/// 字符串插值里的简单标识符（不含关键字）。
pub fn simple_interp_ident(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let Some(c) = chars.next() else {
        return false;
    };
    if !(c == '_' || c.is_alphabetic()) || !chars.all(|c| c == '_' || c.is_alphanumeric()) {
        return false;
    }
    !matches!(
        s,
        "true"
            | "false"
            | "null"
            | "this"
            | "super"
            | "if"
            | "else"
            | "for"
            | "while"
            | "do"
            | "switch"
            | "case"
            | "default"
            | "return"
            | "break"
            | "continue"
            | "throw"
            | "try"
            | "catch"
            | "finally"
            | "class"
            | "mixin"
            | "enum"
            | "extension"
            | "typedef"
            | "var"
            | "final"
            | "const"
            | "new"
            | "is"
            | "as"
            | "in"
            | "void"
            | "dynamic"
            | "with"
            | "on"
            | "required"
            | "late"
            | "await"
            | "yield"
            | "async"
            | "sync"
            | "import"
            | "export"
            | "library"
            | "part"
            | "show"
            | "hide"
            | "of"
            | "external"
            | "abstract"
            | "static"
            | "covariant"
            | "factory"
            | "get"
            | "set"
            | "operator"
            | "assert"
            | "when"
    )
}