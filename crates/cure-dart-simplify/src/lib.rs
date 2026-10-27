//! Dart 简化：在无损 token 文档上做版本无关的安全改写。
//!
//! 规则刻意保守：
//! - 常量折叠只在 32 位整数 / bool / 简单字符串域内，VM 与 Web 语义一致；
//! - `if (常量) { ... }` 只选块分支，保留块作用域；
//! - 不删除 `new`：Dart 2 才允许省略，Dart 1 需要它；
//! - 括号删除只针对原子表达式，避开控制条件/参数表/函数表达式；
//! - 字符串插值只压缩安全的 `${ident}`，且绝不重写 raw/嵌套字符串；
//! - 语句级改写只在整条语句可无歧义切分时进行（if→三元、不可达、
//!   循环尾 continue、空 finally）。

mod const_eval;
mod stmt;

use std::collections::BTreeMap;

use cure_dart_ast::{make_simple_string, BraceKind, DartAst, TokKind, Token};
use cure_engine::Config;
use const_eval::{context_allows, eval_range, eval_range_tracked, simple_interp_ident, Const};
use stmt::{branch_assign, branch_return, is_labeled, is_terminal, split_block, statement_context, token_text};

#[derive(Clone, Debug, Default)]
pub struct Report {
    pub edits: usize,
    pub iterations: usize,
    pub by_rule: BTreeMap<&'static str, usize>,
}

impl Report {
    fn hit(&mut self, rule: &'static str) {
        self.edits += 1;
        *self.by_rule.entry(rule).or_insert(0) += 1;
    }
}

pub fn default_rules() -> &'static [&'static str] {
    &[
        "dart_const_fold",
        "dart_ternary_const",
        "dart_ternary_bool",
        "dart_ternary_not",
        "dart_bool_literal_op",
        "dart_double_negation",
        "dart_negated_eq",
        "dart_const_if",
        "dart_if_return",
        "dart_if_assign_ternary",
        "dart_else_after_terminal",
        "dart_nested_if",
        "dart_empty_then",
        "dart_assign_compound",
        "dart_unreachable",
        "dart_trailing_continue",
        "dart_trailing_return",
        "dart_trailing_assign_return",
        "dart_empty_finally",
        "dart_try_unwrap",
        "dart_adjacent_strings",
        "dart_paren_removal",
        "dart_empty_else",
        "dart_string_interp",
    ]
}

struct Edit {
    start: usize,
    end: usize,
    tokens: Vec<Token>,
}

pub fn simplify_module(ast: &mut DartAst, cfg: &Config) -> Report {
    let mut report = Report::default();
    for _ in 0..32 {
        report.iterations += 1;
        let mut changed = false;
        if enabled(cfg, "dart_const_fold") {
            changed |= const_fold(ast, &mut report);
        }
        if enabled(cfg, "dart_ternary_const") {
            changed |= ternary_const(ast, &mut report);
        }
        if enabled(cfg, "dart_ternary_bool") {
            changed |= ternary_bool(ast, &mut report);
        }
        if enabled(cfg, "dart_ternary_not") {
            changed |= ternary_not(ast, &mut report);
        }
        if enabled(cfg, "dart_bool_literal_op") {
            changed |= bool_literal_op(ast, &mut report);
        }
        if enabled(cfg, "dart_double_negation") {
            changed |= double_negation(ast, &mut report);
        }
        if enabled(cfg, "dart_negated_eq") {
            changed |= negated_eq(ast, &mut report);
        }
        if enabled(cfg, "dart_const_if") {
            changed |= const_if(ast, &mut report);
        }
        if enabled(cfg, "dart_if_return") {
            changed |= if_return(ast, &mut report);
        }
        if enabled(cfg, "dart_if_assign_ternary") {
            changed |= if_assign_ternary(ast, &mut report);
        }
        if enabled(cfg, "dart_else_after_terminal") {
            changed |= else_after_terminal(ast, &mut report);
        }
        if enabled(cfg, "dart_nested_if") {
            changed |= nested_if(ast, &mut report);
        }
        if enabled(cfg, "dart_empty_then") {
            changed |= empty_then(ast, &mut report);
        }
        if enabled(cfg, "dart_assign_compound") {
            changed |= assign_compound(ast, &mut report);
        }
        if enabled(cfg, "dart_unreachable") {
            changed |= unreachable(ast, &mut report);
        }
        if enabled(cfg, "dart_trailing_continue") {
            changed |= trailing_continue(ast, &mut report);
        }
        if enabled(cfg, "dart_trailing_return") {
            changed |= trailing_return(ast, &mut report);
        }
        if enabled(cfg, "dart_trailing_assign_return") {
            changed |= trailing_assign_return(ast, &mut report);
        }
        if enabled(cfg, "dart_empty_finally") {
            changed |= empty_finally(ast, &mut report);
        }
        if enabled(cfg, "dart_try_unwrap") {
            changed |= try_unwrap(ast, &mut report);
        }
        if enabled(cfg, "dart_adjacent_strings") {
            changed |= adjacent_strings(ast, &mut report);
        }
        if enabled(cfg, "dart_paren_removal") {
            changed |= paren_removal(ast, &mut report);
        }
        if enabled(cfg, "dart_empty_else") {
            changed |= empty_else(ast, &mut report);
        }
        if enabled(cfg, "dart_string_interp") {
            changed |= string_interp(ast, &mut report);
        }
        if !changed {
            break;
        }
    }
    report
}

fn enabled(cfg: &Config, rule: &str) -> bool {
    !cfg.disabled_rules.contains(rule)
}

fn is_word(ast: &DartAst, i: usize) -> bool {
    ast.tokens[i].kind.is_word()
}

fn expr_intro(s: &str) -> bool {
    matches!(
        s,
        "return" | "throw" | "yield" | "await" | "case" | "when" | "in"
    )
}

fn prev_allows_start(ast: &DartAst, i: usize) -> bool {
    let Some(p) = ast.prev_sig(i) else {
        return true;
    };
    let t = ast.text(p);
    if expr_intro(t) {
        return true;
    }
    if is_word(ast, p) {
        return false;
    }
    !matches!(
        t,
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
            | "-"
            | "+"
            | "~"
    )
}

/// 布尔字面量是否处于一个完整逻辑操作数的起点。
/// `a == true && b` 里的 `true` 是 `==` 的右操作数，不能当作 `&&` 的左操作数。
fn logical_operand_start(ast: &DartAst, i: usize) -> bool {
    let Some(p) = ast.prev_sig(i) else {
        return true;
    };
    if ast.tokens[p].kind.is_word() {
        return matches!(
            ast.text(p),
            "return" | "throw" | "yield" | "case" | "when" | "in" | "else"
        );
    }
    matches!(
        ast.text(p),
        "(" | "["
            | "{"
            | ","
            | ";"
            | ":"
            | "="
            | "=>"
            | "?"
            | "??"
            | "??="
            | "+="
            | "-="
            | "*="
            | "/="
            | "%="
            | "~/="
            | "&="
            | "|="
            | "^="
            | "<<="
            | ">>="
            | ">>>="
    )
}

fn boundary_after(ast: &DartAst, end: usize) -> bool {
    match ast.next_sig(end.saturating_sub(1)) {
        None => true,
        Some(n) => matches!(
            ast.text(n),
            ";" | "," | ")" | "]" | "}" | ":" | "?" | "=>" | "when"
        ),
    }
}

fn can_start_expr(ast: &DartAst, i: usize) -> bool {
    match ast.tokens[i].kind {
        TokKind::Number | TokKind::Str => true,
        TokKind::Ident => matches!(ast.text(i), "true" | "false"),
        TokKind::Punct => matches!(ast.text(i), "(" | "-" | "+" | "!" | "~"),
        _ => false,
    }
}

/// 从三元运算符的 `?` 向前扫描条件表达式起点。
fn expr_start(ast: &DartAst, q: usize) -> Option<usize> {
    let mut j = q;
    while j > 0 {
        j -= 1;
        let t = ast.text(j);
        if matches!(t, ")" | "]" | "}") {
            let open = ast.matching(j)?;
            if t == "}"
                && matches!(
                    ast.brace_kind(open),
                    Some(BraceKind::Block) | Some(BraceKind::SwitchBlock)
                )
            {
                // 前一条语句的块结束，不能并入条件。
                return Some(j + 1);
            }
            j = open;
            continue;
        }
        match t {
            "(" | "[" | "{" => return Some(j + 1),
            ";" | ","
            | "="
            | "??="
            | "+="
            | "-="
            | "*="
            | "/="
            | "%="
            | "~/="
            | "&="
            | "|="
            | "^="
            | "<<="
            | ">>="
            | ">>>="
            | "=>"
            | ":"
            | "?"
            | ".."
            | "?.." => return Some(j + 1),
            _ if ast.tokens[j].kind == TokKind::Ident
                && matches!(
                    t,
                    "return"
                        | "throw"
                        | "yield"
                        | "if"
                        | "while"
                        | "switch"
                        | "case"
                        | "when"
                        | "in"
                        | "assert"
                        | "for"
                        | "do"
                        | "else"
                ) =>
            {
                return Some(j + 1)
            }
            _ => {}
        }
    }
    Some(0)
}

/// 找到三元 `?` 对应的 `:`；不是三元（可空类型、map 冒号）时返回 None。
fn ternary_colon(ast: &DartAst, q: usize) -> Option<usize> {
    let mut nested = 0usize;
    let mut j = q + 1;
    while j < ast.tokens.len() {
        let t = ast.text(j);
        if matches!(t, "(" | "[" | "{") {
            j = ast.matching(j)? + 1;
            continue;
        }
        match t {
            "?" => nested += 1,
            ":" => {
                if nested == 0 {
                    return Some(j);
                }
                nested -= 1;
            }
            ";" | "," | ")" | "]" | "}" | "=>" => return None,
            _ => {}
        }
        j += 1;
    }
    None
}

/// else 分支的结束位置：顶层 `,`/闭括号/`;`/外层 `:` 都算结束。
fn ternary_else_end(ast: &DartAst, start: usize) -> Option<usize> {
    let mut nested = 0usize;
    let mut j = start;
    while j < ast.tokens.len() {
        let t = ast.text(j);
        if matches!(t, "(" | "[" | "{") {
            j = ast.matching(j)? + 1;
            continue;
        }
        match t {
            "?" => nested += 1,
            ":" => {
                if nested == 0 {
                    return Some(j);
                }
                nested -= 1;
            }
            ";" | "," | ")" | "]" | "}" => return Some(j),
            _ => {}
        }
        j += 1;
    }
    Some(ast.tokens.len())
}

/// `true`/`false`（可带一层括号），返回值与结束位置。`i` 是首 token。
fn bool_atom(ast: &DartAst, i: usize, limit: usize) -> Option<(bool, usize)> {
    if i >= limit || i >= ast.tokens.len() {
        return None;
    }
    let value = |j: usize| -> Option<bool> {
        match ast.text(j) {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    };
    let v = if ast.text(i) == "(" {
        let close = ast.matching(i)?;
        if close >= limit {
            return None;
        }
        let inner = ast.next_sig(i)?;
        if inner >= close {
            return None;
        }
        if ast.next_sig(inner).is_some_and(|n| n < close) {
            return None;
        }
        let v = value(inner)?;
        return Some((v, close + 1));
    } else {
        value(i)?
    };
    Some((v, i + 1))
}

/// 单操作数结束位置：一元前缀 + 原子 / 成对括号。`i` 是首 token。
fn operand_end(ast: &DartAst, mut i: usize, limit: usize) -> Option<usize> {
    if i >= limit || i >= ast.tokens.len() {
        return None;
    }
    let mut guard = 0;
    while matches!(ast.text(i), "!" | "~" | "-" | "+" | "await") {
        guard += 1;
        if guard > 8 {
            return None;
        }
        i = ast.next_sig(i)?;
        if i >= limit {
            return None;
        }
    }
    if matches!(ast.text(i), "(" | "[" | "{") {
        let close = ast.matching(i)?;
        return (close < limit).then_some(close + 1);
    }
    is_word(ast, i).then_some(i + 1)
}

/// 逻辑运算右侧（`&&`/`||` 的右操作数）结束位置。
/// `true || X` 的 X 按 `||` 优先级扫描（`&&` 属于 X），
/// `false && X` 的 X 按 `&&` 优先级扫描（`||` 结束）。
fn logic_rhs_end(ast: &DartAst, start: usize, op: &str) -> Option<usize> {
    let mut j = start;
    while j < ast.tokens.len() {
        match ast.text(j) {
            "(" | "[" | "{" => {
                j = ast.matching(j)? + 1;
                continue;
            }
            ";" | "," | ")" | "]" | "}" | ":" | "?" | "=>" | "when" | "??" => {
                return Some(j)
            }
            "&&" if op == "&&" => return Some(j),
            "||" => return Some(j),
            _ => {}
        }
        j += 1;
    }
    Some(ast.tokens.len())
}

/// `X && true` / `X || false` 中 X 的起点（同/低优先级运算符处截断）。
fn logic_lhs_start(ast: &DartAst, op_i: usize) -> Option<usize> {
    let mut j = op_i;
    while j > 0 {
        j -= 1;
        match ast.text(j) {
            ")" | "]" | "}" => {
                j = ast.matching(j)?;
            }
            "(" | "[" | "{" => return Some(j + 1),
            ";" | "," | "=" | "=>" | "?" | ":" | "??" | "??=" | "+=" | "-=" | "*=" | "/="
            | "%=" | "~/=" | "&=" | "|=" | "^=" | "<<=" | ">>=" | ">>>=" | "&&" | "||" => {
                return Some(j + 1)
            }
            _ if ast.tokens[j].kind == TokKind::Ident
                && matches!(
                    ast.text(j),
                    "return"
                        | "throw"
                        | "yield"
                        | "if"
                        | "while"
                        | "switch"
                        | "case"
                        | "when"
                        | "in"
                        | "assert"
                        | "for"
                        | "do"
                        | "else"
                        | "var"
                        | "final"
                        | "const"
                        | "late"
                ) =>
            {
                return Some(j + 1)
            }
            _ => {}
        }
    }
    Some(0)
}

fn logic_boundary(ast: &DartAst, end: usize, op: &str) -> bool {
    match ast.next_sig(end.saturating_sub(1)) {
        None => true,
        Some(n) => {
            let t = ast.text(n);
            matches!(
                t,
                ";" | "," | ")" | "]" | "}" | ":" | "?" | "=>" | "when" | "&&" | "||"
            ) && !(op == "||" && t == "&&")
        }
    }
}

/// `!` 是否处于前缀（逻辑非）位置；`x!` 的后缀空断言返回 false。
fn bang_is_prefix(ast: &DartAst, i: usize) -> bool {
    match ast.prev_sig(i) {
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

/// 标出所有 Dart 3 模式 token：switch 语句/表达式 arm 的 pattern 部分，
/// 以及 if-case 的 pattern 部分。模式里的 `&&`/`||` 是模式组合器，不是
/// 布尔表达式；switch 表达式的 arm（`pattern => value`）没有 `case`
/// 关键字，必须按块扫描，不能靠回看关键字判断。
fn pattern_mask(ast: &DartAst) -> Vec<bool> {
    let mut mask = vec![false; ast.tokens.len()];
    for i in 0..ast.tokens.len() {
        if ast.text(i) == "{" && ast.brace_kind(i) == Some(BraceKind::SwitchBlock) {
            if let Some(close) = ast.matching(i) {
                mark_switch_patterns(ast, i, close, &mut mask);
            }
        }
    }
    for i in 0..ast.tokens.len() {
        if ast.text(i) == "case" {
            mark_if_case(ast, i, &mut mask);
        }
    }
    mask
}

fn mark_switch_patterns(ast: &DartAst, sb: usize, close: usize, mask: &mut [bool]) {
    let mut pattern = true;
    let mut k = sb + 1;
    while k < close && k < ast.tokens.len() {
        let t = ast.text(k);
        if matches!(t, "(" | "[" | "{") {
            if let Some(m) = ast.matching(k) {
                if pattern {
                    let end = m.min(mask.len() - 1);
                    for slot in &mut mask[k..=end] {
                        *slot = true;
                    }
                }
                k = m + 1;
                continue;
            }
        }
        if pattern {
            mask[k] = true;
        }
        match t {
            "=>" | ":" => pattern = false,
            "," | "case" | "default" => pattern = true,
            _ => {}
        }
        k += 1;
    }
}

/// if-case 的模式在 `case` 之后、`when`/条件右括号之前。
fn mark_if_case(ast: &DartAst, case_i: usize, mask: &mut [bool]) {
    let mut depth = 0usize;
    let mut open = None;
    let mut j = case_i;
    while j > 0 {
        j -= 1;
        match ast.text(j) {
            ")" => depth += 1,
            "(" => {
                if depth == 0 {
                    open = Some(j);
                    break;
                }
                depth -= 1;
            }
            "]" => {
                let Some(m) = ast.matching(j) else {
                    return;
                };
                j = m;
            }
            "}" => {
                let Some(m) = ast.matching(j) else {
                    return;
                };
                if ast.brace_kind(m) != Some(BraceKind::Collection) {
                    return;
                }
                j = m;
            }
            "[" | "{" | ";" => return,
            _ => {}
        }
    }
    let Some(o) = open else {
        return;
    };
    let Some(c) = ast.matching(o) else {
        return;
    };
    if !(o < case_i && case_i < c) {
        return;
    }
    let mut end = c;
    let mut k = case_i + 1;
    while k < c {
        match ast.text(k) {
            "(" | "[" | "{" => {
                let Some(m) = ast.matching(k) else {
                    return;
                };
                k = m + 1;
                continue;
            }
            "when" => {
                end = k;
                break;
            }
            _ => {}
        }
        k += 1;
    }
    for slot in &mut mask[case_i + 1..end] {
        *slot = true;
    }
}

/// 标出控制流条件 token（if/while/switch/assert/for 的条件、三元条件）。
/// Dart 的流分析对 `1 < 2` 这类非常量字面量条件不做折叠；把条件改成
/// `true`/`false` 会改变 late 变量的 definite assignment、返回类型推断等
/// 静态诊断，所以条件里的布尔折叠必须跳过。
fn control_condition_mask(ast: &DartAst) -> Vec<bool> {
    let mut mask = vec![false; ast.tokens.len()];
    for i in 0..ast.tokens.len() {
        if matches!(ast.text(i), "if" | "while" | "switch" | "assert" | "for") {
            let Some(open) = ast.next_sig(i) else {
                continue;
            };
            if ast.text(open) != "(" {
                continue;
            }
            let Some(close) = ast.matching(open) else {
                continue;
            };
            for slot in &mut mask[open + 1..close] {
                *slot = true;
            }
        }
    }
    mask
}

/// 控制流条件 + 所有三元条件。用于 const_fold / bool_literal_op：
/// 三元分支选择也会影响 late 变量的静态流分析。
fn condition_mask(ast: &DartAst) -> Vec<bool> {
    let mut mask = control_condition_mask(ast);
    for q in 0..ast.tokens.len() {
        if ast.text(q) != "?" || ternary_colon(ast, q).is_none() {
            continue;
        }
        if let Some(s) = expr_start(ast, q) {
            for slot in &mut mask[s..q] {
                *slot = true;
            }
        }
    }
    mask
}

/// 表达式的静态结果是否必然为 bool。Dart 的 `!` 是运行时类型检查，
/// `!!dynamicValue` 对非 bool 会抛错；只有被否定的表达式本身必然产出
/// bool 时（比较/逻辑/is）去掉双重否定才不改变行为。
fn is_bool_expr(ast: &DartAst, start: usize, end: usize) -> bool {
    let mut s = start;
    let mut e = end;
    while s + 1 < e && ast.text(s) == "(" && ast.matching(s) == Some(e - 1) {
        s += 1;
        e -= 1;
    }
    if s >= e {
        return false;
    }
    if matches!(ast.text(s), "!" | "true" | "false") {
        return true;
    }
    let mut j = s;
    while j < e {
        let t = ast.text(j);
        if matches!(t, "(" | "[" | "{") {
            let Some(m) = ast.matching(j) else {
                return false;
            };
            if m >= e {
                return false;
            }
            j = m + 1;
            continue;
        }
        if matches!(t, "==" | "!=" | "<=" | ">=" | "&&" | "||" | "is") {
            return true;
        }
        j += 1;
    }
    false
}

/// 整个区间是否就是一个操作数（原子/一元前缀/括号组）。
fn single_operand(ast: &DartAst, start: usize, end: usize) -> bool {
    start < end && operand_end(ast, start, end) == Some(end)
}

/// 替换成较低优先级表达式后，是否需要在两侧加括号。
fn needs_parens_in_context(ast: &DartAst, start: usize, end: usize) -> bool {
    if let Some(p) = ast.prev_sig(start) {
        if matches!(ast.text(p), "!" | "~" | "-" | "+" | "++" | "--" | "await") {
            return true;
        }
    }
    match ast.next_sig(end.saturating_sub(1)) {
        None => false,
        Some(n) => matches!(
            ast.text(n),
            "." | "?."
                | ".."
                | "?.."
                | "["
                | "("
                | "!"
                | "++"
                | "--"
                | "*"
                | "/"
                | "%"
                | "~/"
                | "+"
                | "-"
                | "<<"
                | ">>"
                | ">>>"
                | "&"
                | "^"
                | "|"
                | "is"
                | "as"
        ),
    }
}

fn const_fold(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let pattern = pattern_mask(ast);
    let conditions = condition_mask(ast);
    let mut i = 0usize;
    while i < ast.tokens.len() {
        if ast.tokens[i].is_comment()
            || !can_start_expr(ast, i)
            || !prev_allows_start(ast, i)
            || pattern[i]
        {
            i += 1;
            continue;
        }
        if ast.text(i) == "(" && ast.matching(i).is_none() {
            i += 1;
            continue;
        }
        let Some(res) = eval_range_tracked(ast, i, ast.tokens.len()) else {
            i += 1;
            continue;
        };
        if !context_allows(ast, i, &res) {
            i += 1;
            continue;
        }
        let (value, end) = (res.value, res.end);
        if end <= i + 1 || end > ast.tokens.len() {
            i += 1;
            continue;
        }
        if ast.has_comment_between(i, end - 1) || !boundary_after(ast, end) {
            i += 1;
            continue;
        }
        if matches!(value, Const::Bool(_)) && conditions[i] {
            i += 1;
            continue;
        }
        let Some(tok) = const_token(&value, &ast.tokens[i]) else {
            i += 1;
            continue;
        };
        // `-1` 之类的一元字面量会被重新词法化成 `-` + `1`；若替换文本与
        // 原 token 拼接完全相同，折叠只是 print-invisible 的空转，跳过。
        let original: String = ast.tokens[i..end]
            .iter()
            .map(|t| t.text.as_ref())
            .collect();
        if tok.text.as_ref() == original {
            i = end;
            continue;
        }
        edits.push(Edit {
            start: i,
            end,
            tokens: vec![tok],
        });
        i = end;
    }
    apply_edits(ast, edits, "dart_const_fold", report)
}

fn const_token(value: &Const, at: &Token) -> Option<Token> {
    let (kind, text) = match value {
        Const::Int(n) => (TokKind::Number, n.to_string()),
        Const::Bool(b) => (TokKind::Ident, if *b { "true" } else { "false" }.to_string()),
        Const::Str(s) => (TokKind::Str, make_simple_string(s)?),
    };
    Some(Token::new(kind, text, at.line, at.col, at.newlines_before))
}

fn const_if(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let mut i = 0usize;
    while i < ast.tokens.len() {
        if ast.tokens[i].kind != TokKind::Ident || ast.text(i) != "if" {
            i += 1;
            continue;
        }
        let Some(open) = ast.next_sig(i) else {
            break;
        };
        if ast.text(open) != "(" {
            i += 1;
            continue;
        }
        let Some(close) = ast.matching(open) else {
            i += 1;
            continue;
        };
        let cond = eval_range(ast, open + 1, close);
        let Some((Const::Bool(cond), used)) = cond else {
            i += 1;
            continue;
        };
        if used != close {
            i += 1;
            continue;
        }
        let Some(tb) = ast.next_sig(close) else {
            i += 1;
            continue;
        };
        if ast.text(tb) != "{" || ast.brace_kind(tb) != Some(BraceKind::Block) {
            i += 1;
            continue;
        }
        let Some(tb_close) = ast.matching(tb) else {
            i += 1;
            continue;
        };
        let mut else_range = None;
        if let Some(e) = ast.next_sig(tb_close) {
            if ast.text(e) == "else" {
                let Some(eb) = ast.next_sig(e) else {
                    i += 1;
                    continue;
                };
                if ast.text(eb) != "{" || ast.brace_kind(eb) != Some(BraceKind::Block) {
                    i += 1;
                    continue;
                }
                let Some(eb_close) = ast.matching(eb) else {
                    i += 1;
                    continue;
                };
                else_range = Some((e, eb, eb_close));
            }
        }
        let end = else_range.map(|(_, _, c)| c).unwrap_or(tb_close) + 1;
        if ast.has_comment_between(i, end - 1) {
            i += 1;
            continue;
        }
        let then_sensitive = ast.tokens[tb + 1..tb_close]
            .iter()
            .any(|t| affects_flow_analysis(t.text.as_ref()));
        let else_sensitive = else_range.is_some_and(|(_, eb, ebc)| {
            ast.tokens[eb + 1..ebc]
                .iter()
                .any(|t| affects_flow_analysis(t.text.as_ref()))
        });
        if then_sensitive || else_sensitive {
            // 分支里的赋值/return/yield/for-in 会影响 late 变量流分析和返回
            // 类型推断；即使条件在常量域内可判，Dart 分析器也未必折叠它。
            i += 1;
            continue;
        }
        let (block, repl_end) = if cond {
            (tb, else_range.map(|(_, _, c)| c + 1).unwrap_or(tb_close + 1))
        } else if let Some((_, eb, eb_close)) = else_range {
            (eb, eb_close + 1)
        } else {
            i += 1;
            continue;
        };
        let Some(block_close) = ast.matching(block) else {
            i += 1;
            continue;
        };
        let mut tokens = ast.tokens[block..=block_close].to_vec();
        if let Some(first) = tokens.first_mut() {
            first.newlines_before = ast.tokens[i].newlines_before;
        }
        edits.push(Edit {
            start: i,
            end: repl_end,
            tokens,
        });
        i = repl_end;
    }
    apply_edits(ast, edits, "dart_const_if", report)
}

fn has_question(ast: &DartAst, range: (usize, usize)) -> bool {
    ast.tokens[range.0..range.1].iter().any(|t| t.text.as_ref() == "?")
}

fn push_range(out: &mut Vec<Token>, ast: &DartAst, range: (usize, usize)) {
    for (k, idx) in (range.0..range.1).enumerate() {
        let mut t = ast.tokens[idx].clone();
        if k == 0 {
            t.newlines_before = 0;
        }
        out.push(t);
    }
}

fn punct(text: &str) -> Token {
    Token::new(TokKind::Punct, text, 0, 0, 0)
}

/// `i` 处的 `if` 是否位于语句列表的起点，而不是无括号的 else/循环/if
/// 子语句或标签语句。无括号子语句若把 else 去掉，裸块会跑到外层语句之后
/// (`if (a) { x = 1; } else if (b) { throw ...; } else { x = 2; }`)；
/// 标签语句的 else 里可能有 `break label`，也不能把块移出标签。
fn direct_stmt_start(ast: &DartAst, i: usize) -> bool {
    match ast.prev_sig(i) {
        None => true,
        Some(p) => matches!(ast.text(p), "{" | ";" | "}"),
    }
}

/// 分支表达式是否同一种简单字面量（数值/字符串/bool/null 之外）。
/// 只有这种分支的三元表达式静态类型与原来的两个 return/赋值一致；
/// `Response` 与 `Future<Response>` 之类不同表达式类型的 LUB 可能退化为
/// `Object`，让原本能通过 `FutureOr<Response>` 返回类型检查的代码编译失败。
fn literal_kind(ast: &DartAst, range: (usize, usize)) -> Option<u8> {
    let mut s = range.0;
    let mut e = range.1;
    while s + 1 < e && ast.text(s) == "(" && ast.matching(s) == Some(e - 1) {
        s += 1;
        e -= 1;
    }
    if s >= e {
        return None;
    }
    let mut sign = false;
    if matches!(ast.text(s), "-" | "+") {
        sign = true;
        s += 1;
        if s >= e {
            return None;
        }
    }
    if e != s + 1 {
        return None;
    }
    let kind = match ast.tokens[s].kind {
        TokKind::Number => {
            let text = ast.text(s);
            let int_prefix = text.starts_with("0x")
                || text.starts_with("0X")
                || text.starts_with("0b")
                || text.starts_with("0B");
            if int_prefix || !(text.contains('.') || text.contains('e') || text.contains('E')) {
                1
            } else {
                4
            }
        }
        TokKind::Str => 2,
        TokKind::Ident if matches!(ast.text(s), "true" | "false") => 3,
        _ => return None,
    };
    if sign && kind != 1 {
        return None;
    }
    Some(kind)
}

fn if_return(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let mut i = 0usize;
    while i < ast.tokens.len() {
        if ast.tokens[i].kind != TokKind::Ident || ast.text(i) != "if" {
            i += 1;
            continue;
        }
        if !statement_context(ast, i) {
            i += 1;
            continue;
        }
        let Some(open) = ast.next_sig(i) else {
            break;
        };
        if ast.text(open) != "(" {
            i += 1;
            continue;
        }
        let Some(close) = ast.matching(open) else {
            i += 1;
            continue;
        };
        let cond = (open + 1, close);
        if cond.0 >= cond.1 || ast.tokens[cond.0..cond.1].iter().any(|t| t.text.as_ref() == "case") {
            // `if (value case Pattern)` 是模式匹配，不是 bool 条件，
            // 不能折成 `return condition ? ... : ...`。
            i += 1;
            continue;
        }
        let Some(then_start) = ast.next_sig(close) else {
            i += 1;
            continue;
        };
        let Some(then_info) = branch_return(ast, then_start, ast.tokens.len()) else {
            i += 1;
            continue;
        };
        let mut else_info = None;
        let mut repl_end = then_info.end;
        if let Some(e) = ast.next_sig(then_info.end - 1) {
            if ast.text(e) == "else" {
                let Some(else_start) = ast.next_sig(e) else {
                    i += 1;
                    continue;
                };
                let Some(info) = branch_return(ast, else_start, ast.tokens.len()) else {
                    i += 1;
                    continue;
                };
                repl_end = info.end;
                else_info = Some(info);
            }
        }
        let right = if let Some(info) = else_info {
            info.expr
        } else {
            if !direct_stmt_start(ast, i) {
                i += 1;
                continue;
            }
            let Some(n) = ast.next_sig(then_info.end - 1) else {
                i += 1;
                continue;
            };
            let Some(info) = branch_return(ast, n, ast.tokens.len()) else {
                i += 1;
                continue;
            };
            repl_end = info.end;
            info.expr
        };
        let Some(then_kind) = literal_kind(ast, then_info.expr) else {
            i += 1;
            continue;
        };
        let Some(right_kind) = literal_kind(ast, right) else {
            i += 1;
            continue;
        };
        if then_kind != right_kind {
            i += 1;
            continue;
        }
        if has_question(ast, then_info.expr) || has_question(ast, right) {
            // 已有条件表达式时不再嵌套，避免 40 层 if 链变成巨型三元。
            i += 1;
            continue;
        }
        if ast.has_comment_between(i, repl_end - 1) {
            i += 1;
            continue;
        }
        let mut toks = Vec::new();
        toks.push(Token::new(
            TokKind::Ident,
            "return",
            ast.tokens[i].line,
            ast.tokens[i].col,
            ast.tokens[i].newlines_before,
        ));
        toks.push(punct("("));
        push_range(&mut toks, ast, cond);
        toks.push(punct(")"));
        toks.push(punct("?"));
        toks.push(punct("("));
        push_range(&mut toks, ast, then_info.expr);
        toks.push(punct(")"));
        toks.push(punct(":"));
        toks.push(punct("("));
        push_range(&mut toks, ast, right);
        toks.push(punct(")"));
        toks.push(punct(";"));
        edits.push(Edit {
            start: i,
            end: repl_end,
            tokens: toks,
        });
        i = repl_end;
    }
    apply_edits(ast, edits, "dart_if_return", report)
}

fn if_assign_ternary(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let mut i = 0usize;
    while i < ast.tokens.len() {
        if ast.tokens[i].kind != TokKind::Ident
            || ast.text(i) != "if"
            || !statement_context(ast, i)
        {
            i += 1;
            continue;
        }
        let Some(open) = ast.next_sig(i) else {
            break;
        };
        if ast.text(open) != "(" {
            i += 1;
            continue;
        }
        let Some(close) = ast.matching(open) else {
            i += 1;
            continue;
        };
        let cond = (open + 1, close);
        if cond.0 >= cond.1 || ast.tokens[cond.0..cond.1].iter().any(|t| t.text.as_ref() == "case") {
            // `if (value case Pattern)` 是模式匹配，不是 bool 条件，
            // 不能折成 `return condition ? ... : ...`。
            i += 1;
            continue;
        }
        let Some(then_start) = ast.next_sig(close) else {
            i += 1;
            continue;
        };
        let Some(then_info) = branch_assign(ast, then_start, ast.tokens.len()) else {
            i += 1;
            continue;
        };
        let Some(e) = ast.next_sig(then_info.end - 1) else {
            i += 1;
            continue;
        };
        if ast.text(e) != "else" {
            i += 1;
            continue;
        }
        let Some(else_start) = ast.next_sig(e) else {
            i += 1;
            continue;
        };
        let Some(else_info) = branch_assign(ast, else_start, ast.tokens.len()) else {
            i += 1;
            continue;
        };
        if token_text(ast, then_info.target) != token_text(ast, else_info.target) {
            i += 1;
            continue;
        }
        let Some(then_kind) = literal_kind(ast, then_info.rhs) else {
            i += 1;
            continue;
        };
        let Some(else_kind) = literal_kind(ast, else_info.rhs) else {
            i += 1;
            continue;
        };
        if then_kind != else_kind {
            i += 1;
            continue;
        }
        if has_question(ast, then_info.rhs) || has_question(ast, else_info.rhs) {
            i += 1;
            continue;
        }
        let repl_end = else_info.end;
        if ast.has_comment_between(i, repl_end - 1) {
            i += 1;
            continue;
        }
        let mut toks = Vec::new();
        push_range(&mut toks, ast, then_info.target);
        toks.push(punct("="));
        toks.push(punct("("));
        push_range(&mut toks, ast, cond);
        toks.push(punct(")"));
        toks.push(punct("?"));
        toks.push(punct("("));
        push_range(&mut toks, ast, then_info.rhs);
        toks.push(punct(")"));
        toks.push(punct(":"));
        toks.push(punct("("));
        push_range(&mut toks, ast, else_info.rhs);
        toks.push(punct(")"));
        toks.push(punct(";"));
        if let Some(first) = toks.first_mut() {
            first.newlines_before = ast.tokens[i].newlines_before;
        }
        edits.push(Edit {
            start: i,
            end: repl_end,
            tokens: toks,
        });
        i = repl_end;
    }
    apply_edits(ast, edits, "dart_if_assign_ternary", report)
}

fn else_after_terminal(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let mut i = 0usize;
    while i < ast.tokens.len() {
        if ast.tokens[i].kind != TokKind::Ident
            || ast.text(i) != "if"
            || !statement_context(ast, i)
            || !direct_stmt_start(ast, i)
        {
            i += 1;
            continue;
        }
        let Some(open) = ast.next_sig(i) else {
            break;
        };
        if ast.text(open) != "(" {
            i += 1;
            continue;
        }
        let Some(close) = ast.matching(open) else {
            i += 1;
            continue;
        };
        let Some(then_start) = ast.next_sig(close) else {
            i += 1;
            continue;
        };
        let (then_end, terminal) = if ast.text(then_start) == "{"
            && ast.brace_kind(then_start) == Some(BraceKind::Block)
        {
            let Some(tc) = ast.matching(then_start) else {
                i += 1;
                continue;
            };
            let term = split_block(ast, then_start, tc)
                .and_then(|ss| ss.last().copied())
                .is_some_and(|s| is_terminal(ast, s));
            (tc + 1, term)
        } else {
            let Some(te) = stmt::stmt_end(ast, then_start, ast.tokens.len()) else {
                i += 1;
                continue;
            };
            (te, is_terminal(ast, stmt::Stmt { start: then_start, end: te }))
        };
        if !terminal {
            i += 1;
            continue;
        }
        let Some(e) = ast.next_sig(then_end - 1) else {
            i += 1;
            continue;
        };
        if ast.text(e) != "else" {
            i += 1;
            continue;
        }
        let Some(eb) = ast.next_sig(e) else {
            i += 1;
            continue;
        };
        if ast.text(eb) != "{" || ast.brace_kind(eb) != Some(BraceKind::Block) {
            i += 1;
            continue;
        }
        let Some(ec) = ast.matching(eb) else {
            i += 1;
            continue;
        };
        let else_end = ec + 1;
        let mut toks: Vec<Token> = ast.tokens[i..then_end].to_vec();
        if let Some(first) = toks.first_mut() {
            first.newlines_before = ast.tokens[i].newlines_before;
        }
        toks.extend_from_slice(&ast.tokens[e + 1..else_end]);
        edits.push(Edit {
            start: i,
            end: else_end,
            tokens: toks,
        });
        i = else_end;
    }
    apply_edits(ast, edits, "dart_else_after_terminal", report)
}

/// 不可达代码里的这些 token 会参与静态流分析（返回类型推断、late 变量
/// 的 definite assignment、for-in 对循环变量的赋值等），删掉可能让原本
/// 能编译的代码出现新错误。
fn affects_flow_analysis(t: &str) -> bool {
    matches!(
        t,
        "return"
            | "yield"
            | "="
            | "??="
            | "+="
            | "-="
            | "*="
            | "/="
            | "%="
            | "~/="
            | "&="
            | "|="
            | "^="
            | "<<="
            | ">>="
            | ">>>="
            | "++"
            | "--"
            | "in"
    )
}

fn unreachable(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    for i in 0..ast.tokens.len() {
        if ast.text(i) != "{" || ast.brace_kind(i) != Some(BraceKind::Block) {
            continue;
        }
        let Some(close) = ast.matching(i) else {
            continue;
        };
        let Some(stmts) = split_block(ast, i, close) else {
            continue;
        };
        if stmts.iter().any(|s| is_labeled(ast, *s)) {
            continue;
        }
        let Some(k) = stmts.iter().position(|s| is_terminal(ast, *s)) else {
            continue;
        };
        if k + 1 >= stmts.len() {
            continue;
        }
        let start = stmts[k].end;
        if start < close {
            // 不可达区间里若还有 return/yield、赋值或 for-in，删除可能改变静态
            // 流分析（返回类型推断、late 变量的 definite assignment），保守跳过。
            if ast.tokens[start..close]
                .iter()
                .any(|t| affects_flow_analysis(t.text.as_ref()))
            {
                continue;
            }
            edits.push(Edit {
                start,
                end: close,
                tokens: Vec::new(),
            });
        }
    }
    apply_edits(ast, edits, "dart_unreachable", report)
}

fn trailing_continue(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let mut i = 0usize;
    while i < ast.tokens.len() {
        let t = ast.text(i);
        let body = if t == "for" || t == "while" {
            let Some(open) = ast.next_sig(i) else {
                i += 1;
                continue;
            };
            if ast.text(open) != "(" {
                i += 1;
                continue;
            }
            let Some(close) = ast.matching(open) else {
                i += 1;
                continue;
            };
            ast.next_sig(close)
        } else if t == "do" {
            ast.next_sig(i)
        } else {
            i += 1;
            continue;
        };
        let Some(body) = body else {
            i += 1;
            continue;
        };
        if ast.text(body) != "{" || ast.brace_kind(body) != Some(BraceKind::Block) {
            i += 1;
            continue;
        }
        let Some(close) = ast.matching(body) else {
            i += 1;
            continue;
        };
        let Some(stmts) = split_block(ast, body, close) else {
            i += 1;
            continue;
        };
        let Some(last) = stmts.last().copied() else {
            i += 1;
            continue;
        };
        if ast.text(last.start) != "continue" || last.end != last.start + 2 {
            i += 1;
            continue;
        }
        if ast.text(last.start + 1) != ";" || ast.has_comment_between(last.start, last.end - 1) {
            i += 1;
            continue;
        }
        edits.push(Edit {
            start: last.start,
            end: last.end,
            tokens: Vec::new(),
        });
        i = close + 1;
    }
    apply_edits(ast, edits, "dart_trailing_continue", report)
}

fn empty_finally(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let mut i = 0usize;
    while i < ast.tokens.len() {
        if ast.tokens[i].kind != TokKind::Ident || ast.text(i) != "finally" {
            i += 1;
            continue;
        }
        let Some(open) = ast.next_sig(i) else {
            break;
        };
        if ast.text(open) != "{" || ast.brace_kind(open) != Some(BraceKind::Block) {
            i += 1;
            continue;
        }
        let Some(close) = ast.matching(open) else {
            i += 1;
            continue;
        };
        if ast.next_sig(open) != Some(close) || ast.has_comment_between(open, close) {
            i += 1;
            continue;
        }
        let mut brace = 0usize;
        let mut has_catch = false;
        let mut found_try = false;
        let mut j = i;
        while j > 0 {
            j -= 1;
            match ast.text(j) {
                "}" => brace += 1,
                "{" => {
                    if brace > 0 {
                        brace -= 1;
                    }
                }
                "catch" | "on" if brace == 0 => has_catch = true,
                "try" if brace == 0 => {
                    found_try = true;
                    break;
                }
                _ => {}
            }
        }
        if found_try && has_catch {
            edits.push(Edit {
                start: i,
                end: close + 1,
                tokens: Vec::new(),
            });
            i = close + 1;
        } else {
            i += 1;
        }
    }
    apply_edits(ast, edits, "dart_empty_finally", report)
}

fn adjacent_strings(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let mut i = 0usize;
    while i + 1 < ast.tokens.len() {
        if ast.tokens[i].kind != TokKind::Str || ast.tokens[i + 1].kind != TokKind::Str {
            i += 1;
            continue;
        }
        let (Some(a), Some(b)) = (
            cure_dart_ast::simple_string(&ast.tokens[i].text),
            cure_dart_ast::simple_string(&ast.tokens[i + 1].text),
        ) else {
            i += 1;
            continue;
        };
        let Some(tok) = make_simple_string(&format!("{a}{b}")).map(|text| {
            Token::new(
                TokKind::Str,
                text,
                ast.tokens[i].line,
                ast.tokens[i].col,
                ast.tokens[i].newlines_before,
            )
        }) else {
            i += 1;
            continue;
        };
        edits.push(Edit {
            start: i,
            end: i + 2,
            tokens: vec![tok],
        });
        i += 2;
    }
    apply_edits(ast, edits, "dart_adjacent_strings", report)
}

fn paren_removal(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let mut i = 0usize;
    while i < ast.tokens.len() {
        if ast.text(i) != "(" {
            i += 1;
            continue;
        }
        let Some(close) = ast.matching(i) else {
            i += 1;
            continue;
        };
        if close != i + 2 || ast.tokens[i + 1].is_comment() {
            i += 1;
            continue;
        }
        let inner = i + 1;
        if !matches!(
            ast.tokens[inner].kind,
            TokKind::Ident | TokKind::Number | TokKind::Str
        ) {
            i += 1;
            continue;
        }
        let Some(prev) = ast.prev_sig(i) else {
            i += 1;
            continue;
        };
        let p = ast.text(prev);
        if matches!(
            p,
            "if" | "for" | "while" | "switch" | "assert" | "catch"
        ) {
            i += 1;
            continue;
        }
        if !expr_intro(p)
            && (is_word(ast, prev)
                || matches!(
                    p,
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
                ))
        {
            i += 1;
            continue;
        }
        if ast
            .next_sig(close)
            .is_some_and(|n| matches!(ast.text(n), "=>" | "{" | "async" | "sync" | "*"))
        {
            i += 1;
            continue;
        }
        let mut tok = ast.tokens[inner].clone();
        tok.newlines_before = ast.tokens[i].newlines_before;
        edits.push(Edit {
            start: i,
            end: close + 1,
            tokens: vec![tok],
        });
        i = close + 1;
    }
    apply_edits(ast, edits, "dart_paren_removal", report)
}

fn empty_else(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let mut i = 0usize;
    while i < ast.tokens.len() {
        if ast.tokens[i].kind != TokKind::Ident || ast.text(i) != "else" {
            i += 1;
            continue;
        }
        let Some(prev) = ast.prev_sig(i) else {
            i += 1;
            continue;
        };
        if ast.text(prev) != "}" {
            i += 1;
            continue;
        }
        let Some(open) = ast.matching(prev) else {
            i += 1;
            continue;
        };
        if !matches!(
            ast.brace_kind(open),
            Some(BraceKind::Block) | Some(BraceKind::SwitchBlock)
        ) {
            i += 1;
            continue;
        }
        let Some(eb) = ast.next_sig(i) else {
            i += 1;
            continue;
        };
        if ast.text(eb) != "{" || ast.brace_kind(eb) != Some(BraceKind::Block) {
            i += 1;
            continue;
        }
        let Some(eb_close) = ast.matching(eb) else {
            i += 1;
            continue;
        };
        if ast.next_sig(eb).is_some_and(|n| n < eb_close) {
            i += 1;
            continue;
        }
        if ast.has_comment_between(i, eb_close) {
            i += 1;
            continue;
        }
        edits.push(Edit {
            start: i,
            end: eb_close + 1,
            tokens: Vec::new(),
        });
        i = eb_close + 1;
    }
    apply_edits(ast, edits, "dart_empty_else", report)
}

fn string_interp(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut changed = false;
    for t in &mut ast.tokens {
        if t.kind != TokKind::Str {
            continue;
        }
        if let Some(new) = rewrite_interp(&t.text) {
            t.text = new.into();
            report.hit("dart_string_interp");
            changed = true;
        }
    }
    changed
}

fn rewrite_interp(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 2 {
        return None;
    }
    let q = chars[0];
    if q != '\'' && q != '"' {
        return None;
    }
    if chars.len() >= 6 && chars[1] == q && chars[2] == q {
        return None;
    }
    let end = chars.len() - 1;
    if chars[end] != q {
        return None;
    }
    let content = &text[1..text.len() - 1];
    // 只处理不含任何引号字符的简单字符串：这样插值表达式里不可能再出现
    // 嵌套字符串，逐字符扫描不会误改 raw/嵌套字面量。
    if content.contains('\'') || content.contains('"') {
        return None;
    }
    let mut out = String::new();
    out.push(q);
    let mut i = 1usize;
    let mut changed = false;
    while i < end {
        let c = chars[i];
        if c == '\\' {
            out.push(c);
            if i + 1 < end {
                out.push(chars[i + 1]);
            }
            i += 2;
            continue;
        }
        if c == '$' && i + 1 < end && chars[i + 1] == '{' && !(i > 1 && chars[i - 1] == '$') {
            let mut j = i + 2;
            while j < end && (chars[j].is_alphanumeric() || chars[j] == '_') {
                j += 1;
            }
            if j > i + 2 && j < end && chars[j] == '}' {
                let ident: String = chars[i + 2..j].iter().collect();
                let after_ok = j + 1 >= end || {
                    // 任何非 ASCII 字符都可能是 ID_Continue（组合符号等），
                    // 压缩后会被吸进标识符，保守跳过。
                    let c = chars[j + 1];
                    c.is_ascii() && !(c.is_ascii_alphanumeric() || c == '_' || c == '$')
                };
                if after_ok && simple_interp_ident(&ident) {
                    out.push('$');
                    out.push_str(&ident);
                    i = j + 1;
                    changed = true;
                    continue;
                }
            }
        }
        out.push(c);
        i += 1;
    }
    if !changed {
        return None;
    }
    out.push(q);
    Some(out)
}

fn ternary_const(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let conditions = control_condition_mask(ast);
    let mut i = 0usize;
    while i < ast.tokens.len() {
        if ast.text(i) != "?" {
            i += 1;
            continue;
        }
        let Some(colon) = ternary_colon(ast, i) else {
            i += 1;
            continue;
        };
        let Some(cs) = expr_start(ast, i) else {
            i += 1;
            continue;
        };
        if cs >= i || conditions.get(cs).copied().unwrap_or(false) {
            i += 1;
            continue;
        }
        let Some((Const::Bool(pick), used)) = eval_range(ast, cs, i) else {
            i += 1;
            continue;
        };
        if used != i {
            i += 1;
            continue;
        }
        let Some(end) = ternary_else_end(ast, colon + 1) else {
            i += 1;
            continue;
        };
        if end <= colon + 1 {
            i += 1;
            continue;
        }
        if ast.has_comment_between(cs, end.saturating_sub(1)) {
            i += 1;
            continue;
        }
        let (s, e) = if pick {
            (i + 1, colon)
        } else {
            (colon + 1, end)
        };
        if s >= e {
            i += 1;
            continue;
        }
        // 只有两分支是同类字面量才选分支：这样三元静态类型不变，也不会
        // 丢掉含赋值/return 的分支（影响 late 变量流分析/返回类型推断）。
        let then_kind = literal_kind(ast, (i + 1, colon));
        let else_kind = literal_kind(ast, (colon + 1, end));
        let Some(kind) = then_kind else {
            i += 1;
            continue;
        };
        if else_kind != Some(kind) {
            i += 1;
            continue;
        }
        let mut toks = ast.tokens[s..e].to_vec();
        if let Some(first) = toks.first_mut() {
            first.newlines_before = ast.tokens[cs].newlines_before;
        }
        edits.push(Edit {
            start: cs,
            end,
            tokens: toks,
        });
        i = end;
    }
    apply_edits(ast, edits, "dart_ternary_const", report)
}

fn ternary_bool(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let conditions = control_condition_mask(ast);
    let mut i = 0usize;
    while i < ast.tokens.len() {
        if ast.text(i) != "?" {
            i += 1;
            continue;
        }
        let Some(colon) = ternary_colon(ast, i) else {
            i += 1;
            continue;
        };
        let Some(then_i) = ast.next_sig(i) else {
            i += 1;
            continue;
        };
        let Some((then_v, then_end)) = bool_atom(ast, then_i, colon) else {
            i += 1;
            continue;
        };
        if ast.next_sig(then_end.saturating_sub(1)) != Some(colon) {
            i += 1;
            continue;
        }
        let Some(else_i) = ast.next_sig(colon) else {
            i += 1;
            continue;
        };
        let Some((else_v, else_end)) = bool_atom(ast, else_i, ast.tokens.len()) else {
            i += 1;
            continue;
        };
        if ast.next_sig(else_end.saturating_sub(1)).is_some_and(|n| {
            !matches!(
                ast.text(n),
                ";" | "," | ")" | "]" | "}" | ":" | "=>" | "when"
            )
        }) {
            // else 分支必须整个就是字面量，否则删掉 `? :` 会改变结合方式。
            i += 1;
            continue;
        }
        if then_v == else_v {
            i += 1;
            continue;
        }
        let Some(cs) = expr_start(ast, i) else {
            i += 1;
            continue;
        };
        if cs >= i
            || conditions.get(cs).copied().unwrap_or(false)
            || ast.has_comment_between(cs, else_end.saturating_sub(1))
        {
            i += 1;
            continue;
        }
        if !then_v && else_v {
            let mut toks = vec![punct("!")];
            if single_operand(ast, cs, i) {
                push_range(&mut toks, ast, (cs, i));
            } else {
                toks.push(punct("("));
                push_range(&mut toks, ast, (cs, i));
                toks.push(punct(")"));
            }
            if let Some(first) = toks.first_mut() {
                first.newlines_before = ast.tokens[cs].newlines_before;
            }
            edits.push(Edit {
                start: cs,
                end: else_end,
                tokens: toks,
            });
        } else {
            // `c ? true : false` -> `c` 只在 c 静态必为 bool 时安全；
            // 动态值原本会在条件处抛类型错误。
            if !is_bool_expr(ast, cs, i) {
                i += 1;
                continue;
            }
            edits.push(Edit {
                start: i,
                end: else_end,
                tokens: Vec::new(),
            });
        }
        i = else_end;
    }
    apply_edits(ast, edits, "dart_ternary_bool", report)
}

fn ternary_not(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let conditions = control_condition_mask(ast);
    let mut i = 0usize;
    while i < ast.tokens.len() {
        if ast.text(i) != "?" {
            i += 1;
            continue;
        }
        let Some(colon) = ternary_colon(ast, i) else {
            i += 1;
            continue;
        };
        let Some(cs) = expr_start(ast, i) else {
            i += 1;
            continue;
        };
        if cs >= i
            || conditions.get(cs).copied().unwrap_or(false)
            || ast.text(cs) != "!"
        {
            i += 1;
            continue;
        }
        let rest = cs + 1;
        if rest >= i || operand_end(ast, rest, i + 1) != Some(i) {
            i += 1;
            continue;
        }
        let Some(end) = ternary_else_end(ast, colon + 1) else {
            i += 1;
            continue;
        };
        if end <= colon + 1 || ast.has_comment_between(cs, end.saturating_sub(1)) {
            i += 1;
            continue;
        }
        let mut toks = ast.tokens[rest..i].to_vec();
        toks.push(punct("?"));
        push_range(&mut toks, ast, (colon + 1, end));
        toks.push(punct(":"));
        push_range(&mut toks, ast, (i + 1, colon));
        if let Some(first) = toks.first_mut() {
            first.newlines_before = ast.tokens[cs].newlines_before;
        }
        edits.push(Edit {
            start: cs,
            end,
            tokens: toks,
        });
        i = end;
    }
    apply_edits(ast, edits, "dart_ternary_not", report)
}

fn bool_literal_op(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let pattern = pattern_mask(ast);
    let conditions = condition_mask(ast);
    let mut i = 0usize;
    while i < ast.tokens.len() {
        let lit = match ast.text(i) {
            "true" => true,
            "false" => false,
            _ => {
                i += 1;
                continue;
            }
        };
        if pattern[i] || conditions.get(i).copied().unwrap_or(false) {
            i += 1;
            continue;
        }
        if logical_operand_start(ast, i) {
            if let Some(op_i) = ast.next_sig(i) {
                let op = ast.text(op_i);
                if op == "&&" || op == "||" {
                    if let Some(rhs) = ast.next_sig(op_i) {
                        if rhs < ast.tokens.len() && !ast.has_comment_between(i, rhs - 1) {
                            match (lit, op) {
                                (true, "&&") | (false, "||") => {
                                    let Some(end) = logic_rhs_end(ast, rhs, op) else {
                                        i += 1;
                                        continue;
                                    };
                                    // 原式会用 &&/|| 检查 X 是 bool；替换成 X 后
                                    // 动态非 bool 值不再抛错，必须 X 本身可证 bool。
                                    if end <= rhs || !is_bool_expr(ast, rhs, end) {
                                        i += 1;
                                        continue;
                                    }
                                    edits.push(Edit {
                                        start: i,
                                        end: rhs,
                                        tokens: Vec::new(),
                                    });
                                    i = rhs;
                                    continue;
                                }
                                (false, "&&") | (true, "||") => {
                                    if let Some(end) = logic_rhs_end(ast, rhs, op) {
                                        if end > rhs && !ast.has_comment_between(i, end - 1) {
                                            edits.push(Edit {
                                                start: i,
                                                end,
                                                tokens: vec![ast.tokens[i].clone()],
                                            });
                                            i = end;
                                            continue;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        // 反向形式：`X && true` / `X || false` -> `X`。
        if let Some(op_i) = ast.prev_sig(i) {
            let op = ast.text(op_i);
            let keep = (op == "&&" && lit) || (op == "||" && !lit);
            if keep {
                if let Some(last) = ast.prev_sig(op_i) {
                    let Some(start) = logic_lhs_start(ast, op_i) else {
                        i += 1;
                        continue;
                    };
                    if logic_boundary(ast, i + 1, op)
                        && !ast.has_comment_between(last, i)
                        && is_bool_expr(ast, start, op_i)
                    {
                        edits.push(Edit {
                            start: last,
                            end: i + 1,
                            tokens: vec![ast.tokens[last].clone()],
                        });
                        i += 1;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    apply_edits(ast, edits, "dart_bool_literal_op", report)
}

fn double_negation(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let mut i = 0usize;
    while i + 1 < ast.tokens.len() {
        if ast.text(i) != "!" {
            i += 1;
            continue;
        }
        if !bang_is_prefix(ast, i) {
            i += 1;
            continue;
        }
        if ast.prev_sig(i).is_some_and(|p| ast.text(p) == "is") {
            i += 1;
            continue;
        }
        let Some(second) = ast.next_sig(i) else {
            break;
        };
        if ast.text(second) != "!" {
            i += 1;
            continue;
        }
        let Some(end) = operand_end(ast, second + 1, ast.tokens.len()) else {
            i += 1;
            continue;
        };
        if end <= second + 1
            || ast.has_comment_between(i, end - 1)
            || !is_bool_expr(ast, second + 1, end)
        {
            i += 1;
            continue;
        }
        let mut toks = ast.tokens[second + 1..end].to_vec();
        if let Some(first) = toks.first_mut() {
            first.newlines_before = ast.tokens[i].newlines_before;
        }
        edits.push(Edit {
            start: i,
            end,
            tokens: toks,
        });
        i = end;
    }
    apply_edits(ast, edits, "dart_double_negation", report)
}

fn negated_eq(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let mut i = 0usize;
    while i < ast.tokens.len() {
        if ast.text(i) != "!" {
            i += 1;
            continue;
        }
        if !bang_is_prefix(ast, i) {
            i += 1;
            continue;
        }
        let Some(open) = ast.next_sig(i) else {
            break;
        };
        if ast.text(open) != "(" {
            i += 1;
            continue;
        }
        let Some(close) = ast.matching(open) else {
            i += 1;
            continue;
        };
        if ast.has_comment_between(i, close) {
            i += 1;
            continue;
        }
        // 去掉内层多余括号：`!((a == b))`。
        let mut s = open + 1;
        let mut e = close;
        while s + 1 < e
            && ast.text(s) == "("
            && ast.matching(s).is_some_and(|m| m + 1 == e)
        {
            s += 1;
            e -= 1;
        }
        if s >= e {
            i += 1;
            continue;
        }
        let mut op: Option<(usize, &str)> = None;
        let mut bad = false;
        let mut j = s;
        while j < e {
            let t = ast.text(j);
            if matches!(t, "(" | "[" | "{") {
                let Some(m) = ast.matching(j) else {
                    bad = true;
                    break;
                };
                if m >= e {
                    bad = true;
                    break;
                }
                j = m + 1;
                continue;
            }
            match t {
                "==" | "!=" | "is" => {
                    if op.is_some() {
                        bad = true;
                        break;
                    }
                    op = Some((j, t));
                }
                "&&" | "||" | "?" | "??" | "=>" | "=" | "??=" | "+=" | "-=" | "*=" | "/="
                | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=" | ">>>=" | "," | ";" => {
                    bad = true;
                    break;
                }
                _ => {}
            }
            j += 1;
        }
        if bad {
            i += 1;
            continue;
        }
        let Some((op_i, kind)) = op else {
            i += 1;
            continue;
        };
        if op_i == s || op_i + 1 >= e {
            i += 1;
            continue;
        }
        let left = (s, op_i);
        let mut toks = ast.tokens[left.0..left.1].to_vec();
        match kind {
            "==" | "!=" => {
                toks.push(punct(if kind == "==" { "!=" } else { "==" }));
                toks.extend_from_slice(&ast.tokens[op_i + 1..e]);
            }
            "is" => {
                let mut ti = op_i + 1;
                let mut negated = false;
                if ti < e && ast.text(ti) == "!" {
                    negated = true;
                    ti += 1;
                }
                if ti + 1 != e || !is_word(ast, ti) {
                    // 只处理 `is Type` / `is! Type` 的单 token 类型。
                    i += 1;
                    continue;
                }
                toks.push(ast.tokens[op_i].clone());
                if !negated {
                    toks.push(punct("!"));
                }
                toks.push(ast.tokens[ti].clone());
            }
            _ => {
                i += 1;
                continue;
            }
        }
        if needs_parens_in_context(ast, i, close + 1) {
            let mut wrapped = vec![punct("(")];
            wrapped.append(&mut toks);
            wrapped.push(punct(")"));
            toks = wrapped;
        }
        if let Some(first) = toks.first_mut() {
            first.newlines_before = ast.tokens[i].newlines_before;
        }
        edits.push(Edit {
            start: i,
            end: close + 1,
            tokens: toks,
        });
        i = close + 1;
    }
    apply_edits(ast, edits, "dart_negated_eq", report)
}

/// 二元运算符优先级（数值越大绑定越紧），与 Dart 一致：
/// 位运算比相等/关系更紧。
fn binary_prec(t: &str) -> Option<u8> {
    Some(match t {
        "??" => 1,
        "||" => 2,
        "&&" => 3,
        "==" | "!=" => 4,
        "<" | ">" | "<=" | ">=" => 5,
        "|" => 6,
        "^" => 7,
        "&" => 8,
        "<<" | ">>" | ">>>" => 9,
        "+" | "-" => 10,
        "*" | "/" | "%" | "~/" => 11,
        _ => return None,
    })
}

/// `x = x op RHS` → `x op= RHS` 只在 `RHS` 顶层没有比 `op` 更松/同级
/// 的运算符时成立。`x = x * 10 + digit` 里 `+` 比 `*` 松，折成
/// `x *= 10 + digit` 会变成 `x * (10 + digit)`。
fn rhs_has_looser(ast: &DartAst, start: usize, end: usize, prec: u8) -> bool {
    let mut j = start;
    while j < end {
        let t = ast.text(j);
        if matches!(t, "(" | "[" | "{") {
            let Some(m) = ast.matching(j) else {
                return true;
            };
            j = m + 1;
            continue;
        }
        if matches!(
            t,
            "?" | ":"
                | "="
                | "=>"
                | "??"
                | "??="
                | "+="
                | "-="
                | "*="
                | "/="
                | "%="
                | "~/="
                | "&="
                | "|="
                | "^="
                | "<<="
                | ">>="
                | ">>>="
                | ","
                | ".."
                | "?.."
                | "is"
                | "as"
        ) {
            return true;
        }
        if binary_prec(t).is_some_and(|p| p <= prec) {
            return true;
        }
        j += 1;
    }
    false
}

fn assign_compound(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    for b in 0..ast.tokens.len() {
        if ast.text(b) != "{" || ast.brace_kind(b) != Some(BraceKind::Block) {
            continue;
        }
        let Some(close) = ast.matching(b) else {
            continue;
        };
        let Some(stmts) = split_block(ast, b, close) else {
            continue;
        };
        for st in stmts {
            let semi = st.end - 1;
            if st.end <= st.start + 3 || ast.text(semi) != ";" {
                continue;
            }
            if ast.tokens[st.start].kind != TokKind::Ident {
                continue;
            }
            if matches!(
                ast.text(st.start),
                "var" | "final" | "const" | "late" | "this" | "super" | "return"
            ) {
                continue;
            }
            let Some(eq) = ast.next_sig(st.start) else {
                continue;
            };
            if eq != st.start + 1 || ast.text(eq) != "=" {
                continue;
            }
            let Some(lhs2) = ast.next_sig(eq) else {
                continue;
            };
            if ast.text(lhs2) != ast.text(st.start) {
                continue;
            }
            let Some(op_i) = ast.next_sig(lhs2) else {
                continue;
            };
            let op = ast.text(op_i);
            if !matches!(
                op,
                "+" | "-"
                    | "*"
                    | "/"
                    | "~/"
                    | "%"
                    | "&"
                    | "|"
                    | "^"
                    | "<<"
                    | ">>"
                    | ">>>"
            ) {
                continue;
            }
            let Some(rhs) = ast.next_sig(op_i) else {
                continue;
            };
            let Some(prec) = binary_prec(op) else {
                continue;
            };
            if rhs >= semi
                || ast.has_comment_between(st.start, semi)
                || rhs_has_looser(ast, rhs, semi, prec)
            {
                continue;
            }
            let mut toks = vec![ast.tokens[st.start].clone()];
            toks.push(punct(&format!("{op}=")));
            toks.extend_from_slice(&ast.tokens[rhs..semi]);
            toks.push(ast.tokens[semi].clone());
            edits.push(Edit {
                start: st.start,
                end: st.end,
                tokens: toks,
            });
        }
    }
    apply_edits(ast, edits, "dart_assign_compound", report)
}

fn try_catch_body(ast: &DartAst, start: usize) -> Option<(usize, usize)> {
    let mut j = ast.next_sig(start)?;
    loop {
        match ast.text(j) {
            "{" => break,
            "catch" => {
                let n = ast.next_sig(j)?;
                j = if ast.text(n) == "(" {
                    ast.matching(n)? + 1
                } else {
                    n
                };
            }
            "(" | "[" => {
                j = ast.matching(j)? + 1;
            }
            ";" | ")" | "]" | "}" => return None,
            _ => j += 1,
        }
        if j >= ast.tokens.len() {
            return None;
        }
        j = ast.next_sig(j.saturating_sub(1))?;
    }
    if ast.brace_kind(j) != Some(BraceKind::Block) {
        return None;
    }
    let close = ast.matching(j)?;
    Some((j, close))
}

fn body_is_rethrow(ast: &DartAst, open: usize, close: usize) -> bool {
    if ast.has_comment_between(open, close) {
        return false;
    }
    let Some(r) = ast.next_sig(open) else {
        return false;
    };
    if ast.text(r) != "rethrow" {
        return false;
    }
    let Some(semi) = ast.next_sig(r) else {
        return false;
    };
    semi < close
        && ast.text(semi) == ";"
        && ast.next_sig(semi).map_or(true, |n| n >= close)
}

fn try_unwrap(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let mut i = 0usize;
    while i < ast.tokens.len() {
        if ast.text(i) != "try" {
            i += 1;
            continue;
        }
        let Some(open) = ast.next_sig(i) else {
            break;
        };
        if ast.text(open) != "{" || ast.brace_kind(open) != Some(BraceKind::Block) {
            i += 1;
            continue;
        }
        let Some(close) = ast.matching(open) else {
            i += 1;
            continue;
        };
        let mut cats: Vec<(usize, usize, usize)> = Vec::new();
        let mut fin: Option<(usize, usize)> = None;
        let mut j = close + 1;
        let shape = loop {
            let Some(n) = ast.next_sig(j.saturating_sub(1)) else {
                break Some(());
            };
            match ast.text(n) {
                "on" | "catch" => {
                    let Some((bo, bc)) = try_catch_body(ast, n) else {
                        break None;
                    };
                    cats.push((n, bo, bc));
                    j = bc + 1;
                }
                "finally" => {
                    let Some(bo) = ast.next_sig(n) else {
                        break None;
                    };
                    if ast.text(bo) != "{" || ast.brace_kind(bo) != Some(BraceKind::Block) {
                        break None;
                    }
                    let Some(bc) = ast.matching(bo) else {
                        break None;
                    };
                    fin = Some((bo, bc));
                    break Some(());
                }
                _ => break Some(()),
            }
        };
        if shape.is_none() {
            i += 1;
            continue;
        }
        let try_end = fin
            .map(|(_, bc)| bc + 1)
            .or_else(|| cats.last().map(|(_, _, bc)| bc + 1))
            .unwrap_or(close + 1);
        let removable: Vec<bool> = cats
            .iter()
            .map(|(n, bo, bc)| body_is_rethrow(ast, *bo, *bc) && !ast.has_comment_between(*n, *bc))
            .collect();
        let all_removed = !cats.is_empty() && removable.iter().all(|r| *r);
        let empty_finally = fin.is_some_and(|(bo, bc)| {
            ast.next_sig(bo) == Some(bc) && !ast.has_comment_between(bo, bc)
        });
        let unwrap = if cats.is_empty() {
            empty_finally
        } else {
            all_removed && (fin.is_none() || empty_finally)
        };
        if unwrap {
            if !ast.has_comment_between(close, try_end.saturating_sub(1)) {
                let mut toks = ast.tokens[open..=close].to_vec();
                if let Some(first) = toks.first_mut() {
                    first.newlines_before = ast.tokens[i].newlines_before;
                }
                edits.push(Edit {
                    start: i,
                    end: try_end,
                    tokens: toks,
                });
                i = try_end;
                continue;
            }
        }
        for ((n, _, bc), rem) in cats.iter().zip(removable.iter()) {
            if *rem {
                edits.push(Edit {
                    start: *n,
                    end: bc + 1,
                    tokens: Vec::new(),
                });
            }
        }
        i = try_end;
    }
    apply_edits(ast, edits, "dart_try_unwrap", report)
}

fn nested_if(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let mut i = 0usize;
    while i < ast.tokens.len() {
        if ast.text(i) != "if" || !statement_context(ast, i) {
            i += 1;
            continue;
        }
        let Some(open) = ast.next_sig(i) else {
            break;
        };
        if ast.text(open) != "(" {
            i += 1;
            continue;
        }
        let Some(close) = ast.matching(open) else {
            i += 1;
            continue;
        };
        if ast.tokens[open + 1..close]
            .iter()
            .any(|t| t.text.as_ref() == "case")
        {
            i += 1;
            continue;
        }
        let Some(then_open) = ast.next_sig(close) else {
            i += 1;
            continue;
        };
        if ast.text(then_open) != "{" || ast.brace_kind(then_open) != Some(BraceKind::Block) {
            i += 1;
            continue;
        }
        let Some(then_close) = ast.matching(then_open) else {
            i += 1;
            continue;
        };
        let Some(stmts) = split_block(ast, then_open, then_close) else {
            i += 1;
            continue;
        };
        if stmts.len() != 1 || ast.text(stmts[0].start) != "if" {
            i += 1;
            continue;
        }
        let inner = stmts[0];
        if !statement_context(ast, inner.start)
            || ast.has_comment_between(then_open, inner.start)
            || ast.has_comment_between(inner.end.saturating_sub(1), then_close)
        {
            i += 1;
            continue;
        }
        let Some(io) = ast.next_sig(inner.start) else {
            i += 1;
            continue;
        };
        if ast.text(io) != "(" {
            i += 1;
            continue;
        }
        let Some(ic) = ast.matching(io) else {
            i += 1;
            continue;
        };
        if ast.tokens[io + 1..ic]
            .iter()
            .any(|t| t.text.as_ref() == "case")
        {
            i += 1;
            continue;
        }
        let Some(ithen) = ast.next_sig(ic) else {
            i += 1;
            continue;
        };
        let Some(ithen_end) = stmt::stmt_end(ast, ithen, inner.end) else {
            i += 1;
            continue;
        };
        if ast.next_sig(ithen_end.saturating_sub(1))
            .is_some_and(|n| ast.text(n) == "else" && n < inner.end)
        {
            i += 1;
            continue;
        }
        if ast.next_sig(then_close)
            .is_some_and(|n| ast.text(n) == "else")
        {
            i += 1;
            continue;
        }
        let mut toks = vec![ast.tokens[i].clone()];
        toks.push(punct("("));
        toks.push(punct("("));
        push_range(&mut toks, ast, (open + 1, close));
        toks.push(punct(")"));
        toks.push(punct("&&"));
        toks.push(punct("("));
        push_range(&mut toks, ast, (io + 1, ic));
        toks.push(punct(")"));
        toks.push(punct(")"));
        toks.extend_from_slice(&ast.tokens[ithen..ithen_end]);
        edits.push(Edit {
            start: i,
            end: then_close + 1,
            tokens: toks,
        });
        i = then_close + 1;
    }
    apply_edits(ast, edits, "dart_nested_if", report)
}

fn empty_then(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    let mut i = 0usize;
    while i < ast.tokens.len() {
        if ast.text(i) != "if" || !statement_context(ast, i) {
            i += 1;
            continue;
        }
        let Some(open) = ast.next_sig(i) else {
            break;
        };
        if ast.text(open) != "(" {
            i += 1;
            continue;
        }
        let Some(close) = ast.matching(open) else {
            i += 1;
            continue;
        };
        if ast.tokens[open + 1..close]
            .iter()
            .any(|t| t.text.as_ref() == "case")
        {
            i += 1;
            continue;
        }
        let Some(then_start) = ast.next_sig(close) else {
            i += 1;
            continue;
        };
        let then_end = if ast.text(then_start) == "{" {
            if ast.brace_kind(then_start) != Some(BraceKind::Block) {
                i += 1;
                continue;
            }
            let Some(tc) = ast.matching(then_start) else {
                i += 1;
                continue;
            };
            if ast.has_comment_between(then_start, tc) || ast.next_sig(then_start) != Some(tc) {
                i += 1;
                continue;
            }
            tc + 1
        } else if ast.text(then_start) == ";" {
            then_start + 1
        } else {
            i += 1;
            continue;
        };
        let Some(e) = ast.next_sig(then_end - 1) else {
            i += 1;
            continue;
        };
        if ast.text(e) != "else" {
            i += 1;
            continue;
        }
        let Some(else_start) = ast.next_sig(e) else {
            i += 1;
            continue;
        };
        let Some(else_end) = stmt::stmt_end(ast, else_start, ast.tokens.len()) else {
            i += 1;
            continue;
        };
        // 空 else 交给 empty_else：先删 else 再谈反转。
        if ast.text(else_start) == "{"
            && ast.matching(else_start)
                .is_some_and(|ec| ast.next_sig(else_start) == Some(ec))
        {
            i += 1;
            continue;
        }
        if ast.has_comment_between(i, else_end - 1) {
            i += 1;
            continue;
        }
        let mut toks = vec![ast.tokens[i].clone()];
        toks.push(punct("("));
        toks.push(punct("!"));
        if single_operand(ast, open + 1, close) {
            push_range(&mut toks, ast, (open + 1, close));
        } else {
            toks.push(punct("("));
            push_range(&mut toks, ast, (open + 1, close));
            toks.push(punct(")"));
        }
        toks.push(punct(")"));
        toks.extend_from_slice(&ast.tokens[else_start..else_end]);
        edits.push(Edit {
            start: i,
            end: else_end,
            tokens: toks,
        });
        i = else_end;
    }
    apply_edits(ast, edits, "dart_empty_then", report)
}

fn is_function_body(ast: &DartAst, b: usize) -> bool {
    if ast.text(b) != "{" || ast.brace_kind(b) != Some(BraceKind::Block) {
        return false;
    }
    let Some(mut p) = ast.prev_sig(b) else {
        return false;
    };
    if ast.text(p) == "*" {
        let Some(q) = ast.prev_sig(p) else {
            return false;
        };
        p = q;
    }
    if matches!(ast.text(p), "async" | "sync") {
        let Some(q) = ast.prev_sig(p) else {
            return false;
        };
        p = q;
        if ast.text(p) == "*" {
            let Some(q) = ast.prev_sig(p) else {
                return false;
            };
            p = q;
        }
    }
    if ast.text(p) != ")" {
        return false;
    }
    let Some(open) = ast.matching(p) else {
        return false;
    };
    if ast.text(open) != "(" {
        return false;
    }
    let Some(before) = ast.prev_sig(open) else {
        return true;
    };
    !matches!(
        ast.text(before),
        "if" | "for" | "while" | "switch" | "catch" | "assert"
    )
}

fn trailing_return(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    for b in 0..ast.tokens.len() {
        if !is_function_body(ast, b) {
            continue;
        }
        let Some(close) = ast.matching(b) else {
            continue;
        };
        // 注释可能说明该 return 是刻意保留的（例如 dart2js 内联屏障），
        // 也可能是一条 trailing `// ignore:`；有注释时一律保留。
        if ast.has_comment_between(b, close) {
            continue;
        }
        let Some(stmts) = split_block(ast, b, close) else {
            continue;
        };
        let Some(last) = stmts.last().copied() else {
            continue;
        };
        if ast.text(last.start) != "return"
            || last.end != last.start + 2
            || ast.text(last.start + 1) != ";"
            || ast.has_comment_between(last.start, last.end - 1)
        {
            continue;
        }
        edits.push(Edit {
            start: last.start,
            end: last.end,
            tokens: Vec::new(),
        });
    }
    apply_edits(ast, edits, "dart_trailing_return", report)
}

/// `final y = <expr>; return y;` → `return <expr>;`（同一块内、y 只出现
/// 这两处）。`var` 同样处理；`const`/`late`/带类型注解/多声明器跳过。
///
/// 初始值限制为**上下文不敏感**表达式（无调用/构造/集合字面量/条件），
/// 避免 `return <expr>` 的返回类型上下文改变泛型推断（`List<dynamic>`
/// 与 `List<int>` 的运行期类型差异可被调用方观察到）。
fn trailing_assign_return(ast: &mut DartAst, report: &mut Report) -> bool {
    let mut edits = Vec::new();
    for open in 0..ast.tokens.len() {
        if ast.text(open) != "{" || ast.brace_kind(open) != Some(BraceKind::Block) {
            continue;
        }
        let Some(close) = ast.matching(open) else {
            continue;
        };
        let Some(stmts) = split_block(ast, open, close) else {
            continue;
        };
        for pair in stmts.windows(2) {
            let (decl, ret) = (pair[0], pair[1]);
            let d = &ast.tokens[decl.start..decl.end];
            if d.len() < 5 {
                continue;
            }
            if ast.tokens[decl.start].kind != TokKind::Ident
                || !matches!(ast.text(decl.start), "final" | "var")
            {
                continue;
            }
            let Some(name_i) = ast.next_sig(decl.start) else {
                continue;
            };
            if ast.tokens[name_i].kind != TokKind::Ident || name_i >= decl.end {
                continue;
            }
            let Some(eq_i) = ast.next_sig(name_i) else {
                continue;
            };
            if ast.text(eq_i) != "=" {
                continue;
            }
            let Some(expr_start) = ast.next_sig(eq_i) else {
                continue;
            };
            // 末尾必须是 `;`，且初始值非空、无注释。
            if ast.text(decl.end - 1) != ";" || expr_start + 1 >= decl.end {
                continue;
            }
            if ast.has_comment_between(decl.start, ret.end - 1) {
                continue;
            }
            let expr_end = decl.end - 1; // 不含 `;`
            let name = ast.text(name_i).to_string();
            // 返回语句：恰好 `return <name> ;`（3 个 token）。
            if ast.text(ret.start) != "return" || ret.end != ret.start + 3 {
                continue;
            }
            let rn = ret.start + 1;
            if ast.tokens[rn].kind != TokKind::Ident
                || ast.text(rn) != name
                || ast.text(ret.start + 2) != ";"
            {
                continue;
            }
            // 名字在块内只出现两次（声明目标 + return）。字符串插值不
            // 计入 Ident，保守拒绝初始值里的 `$`。
            let occurrences = ast.tokens[open + 1..close]
                .iter()
                .filter(|t| t.kind == TokKind::Ident && t.text.as_ref() == name)
                .count();
            if occurrences != 2 {
                continue;
            }
            if !context_insensitive_init(ast, expr_start, expr_end) {
                continue;
            }
            let first = &ast.tokens[decl.start];
            let (line, col) = (first.line, first.col);
            let mut repl = Vec::with_capacity(expr_end - expr_start + 2);
            repl.push(Token::new(TokKind::Ident, "return", line, col, 0));
            repl.extend_from_slice(&ast.tokens[expr_start..expr_end]);
            repl.push(Token::new(TokKind::Punct, ";", line, col, 0));
            edits.push(Edit {
                start: decl.start,
                end: ret.end,
                tokens: repl,
            });
        }
    }
    apply_edits(ast, edits, "dart_trailing_assign_return", report)
}

/// 初始值只含标识符/字面量/运算符：无 `(`、`[`、`{`、调用泛型 `<`、
/// `new`、`=>`、`?`、`,`、字符串插值。这些构造的结果类型不会被返回
/// 位置上下文改变。
fn context_insensitive_init(ast: &DartAst, start: usize, end: usize) -> bool {
    for i in start..end {
        let t = &ast.tokens[i];
        match t.kind {
            TokKind::Str => {
                if t.text.contains('$') {
                    return false;
                }
            }
            TokKind::Number => {}
            TokKind::Ident => {
                if matches!(
                    t.text.as_ref(),
                    "new" | "const" | "throw" | "switch" | "await" | "yield"
                ) {
                    return false;
                }
            }
            TokKind::Punct => {
                if matches!(
                    t.text.as_ref(),
                    "(" | ")" | "[" | "]" | "{" | "}" | "<" | ">" | ">>" | ">>>" | "=>"
                        | "?" | ":" | "," | "is" | "as"
                ) {
                    return false;
                }
            }
            _ => return false,
        }
    }
    true
}

fn apply_edits(
    ast: &mut DartAst,
    mut edits: Vec<Edit>,
    rule: &'static str,
    report: &mut Report,
) -> bool {
    if edits.is_empty() {
        return false;
    }
    edits.sort_by_key(|e| e.start);
    let mut kept: Vec<Edit> = Vec::new();
    let mut last = 0usize;
    for e in edits {
        if e.start >= last && e.end <= ast.tokens.len() && e.start < e.end {
            last = e.end;
            kept.push(e);
        }
    }
    if kept.is_empty() {
        return false;
    }
    for e in kept.iter().rev() {
        let first_nl = ast.tokens[e.start].newlines_before;
        let mut repl = e.tokens.clone();
        if let Some(t) = repl.first_mut() {
            if t.newlines_before == 0 {
                t.newlines_before = first_nl;
            }
        } else if e.end < ast.tokens.len() && ast.tokens[e.end].newlines_before == 0 {
            ast.tokens[e.end].newlines_before = first_nl;
        }
        ast.tokens.splice(e.start..e.end, repl);
        report.hit(rule);
    }
    ast.rebuild();
    true
}