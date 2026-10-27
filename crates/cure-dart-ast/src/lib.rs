//! Lossless Dart token document.
//!
//! Dart 1/2/3 语法差异很大（可选 `new`、null safety、records/patterns、
//! extension types、macro/augment 等），本后端不试图为每种版本建独立
//! 语法树：词法层完整保留所有 token 与注释，结构层只建立括号配对和
//! “块/集合”花括号分类。未建模写法天然原样保留，因此同一份文档模型
//! 可以接受所有 Dart 版本。

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokKind {
    Ident,
    Number,
    Str,
    LineComment,
    BlockComment,
    Punct,
}

impl TokKind {
    pub fn is_comment(self) -> bool {
        matches!(self, TokKind::LineComment | TokKind::BlockComment)
    }

    pub fn is_word(self) -> bool {
        matches!(self, TokKind::Ident | TokKind::Number | TokKind::Str)
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokKind,
    /// 紧凑字符串：`String` 的 8 字节容量字段对每个 token 都是纯浪费。
    pub text: Box<str>,
    pub line: u32,
    pub col: u32,
    /// 该 token 之前被跳过的换行数（0 = 与上一 token 同行）。
    pub newlines_before: u32,
}

impl Token {
    pub fn new(
        kind: TokKind,
        text: impl Into<Box<str>>,
        line: u32,
        col: u32,
        newlines_before: u32,
    ) -> Self {
        Token {
            kind,
            text: text.into(),
            line,
            col,
            newlines_before,
        }
    }

    pub fn is_comment(&self) -> bool {
        self.kind.is_comment()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BraceKind {
    /// 语句/声明块，格式化时换行缩进。
    Block,
    /// switch 语句或 switch 表达式的分支体。
    SwitchBlock,
    /// map/set/命名参数等集合字面量，格式化时保持行内。
    Collection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BracketError {
    pub index: usize,
    pub message: String,
}

const PAIR_NONE: u32 = u32::MAX;
const BRACE_NONE: u8 = 0;
const BRACE_BLOCK: u8 = 1;
const BRACE_SWITCH: u8 = 2;
const BRACE_COLLECTION: u8 = 3;

#[derive(Clone, Debug)]
pub struct DartAst {
    pub tokens: Vec<Token>,
    /// 匹配括号下标（4 字节/节点，`Option<usize>` 是 16 字节）。
    pairs: Vec<u32>,
    /// 花括号分类（1 字节/节点）。
    braces: Vec<u8>,
}

impl DartAst {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut ast = DartAst {
            pairs: vec![PAIR_NONE; tokens.len()],
            braces: vec![BRACE_NONE; tokens.len()],
            tokens,
        };
        let _ = ast.rebuild();
        ast
    }

    /// 解析器用：只构建一次括号索引，并返回诊断。
    pub fn from_tokens(tokens: Vec<Token>) -> (Self, Vec<BracketError>) {
        let mut ast = DartAst {
            pairs: vec![PAIR_NONE; tokens.len()],
            braces: vec![BRACE_NONE; tokens.len()],
            tokens,
        };
        let errors = ast.rebuild();
        (ast, errors)
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn text(&self, i: usize) -> &str {
        &self.tokens[i].text
    }

    pub fn matching(&self, i: usize) -> Option<usize> {
        match self.pairs.get(i).copied() {
            Some(PAIR_NONE) | None => None,
            Some(v) => Some(v as usize),
        }
    }

    pub fn brace_kind(&self, i: usize) -> Option<BraceKind> {
        match self.braces.get(i).copied() {
            Some(BRACE_BLOCK) => Some(BraceKind::Block),
            Some(BRACE_SWITCH) => Some(BraceKind::SwitchBlock),
            Some(BRACE_COLLECTION) => Some(BraceKind::Collection),
            _ => None,
        }
    }

    pub fn prev_sig(&self, i: usize) -> Option<usize> {
        let mut j = i;
        while j > 0 {
            j -= 1;
            if !self.tokens[j].is_comment() {
                return Some(j);
            }
        }
        None
    }

    pub fn next_sig(&self, i: usize) -> Option<usize> {
        let mut j = i + 1;
        while j < self.tokens.len() {
            if !self.tokens[j].is_comment() {
                return Some(j);
            }
            j += 1;
        }
        None
    }

    pub fn prev_sig_text(&self, i: usize) -> Option<&str> {
        self.prev_sig(i).map(|j| self.text(j))
    }

    pub fn next_sig_text(&self, i: usize) -> Option<&str> {
        self.next_sig(i).map(|j| self.text(j))
    }

    pub fn has_comment_between(&self, a: usize, b: usize) -> bool {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        self.tokens[lo..=hi].iter().any(|t| t.is_comment())
    }

    /// 重建括号配对和花括号分类。简化器每次修改 token 后调用。
    pub fn rebuild(&mut self) -> Vec<BracketError> {
        let n = self.tokens.len();
        self.pairs.clear();
        self.pairs.resize(n, PAIR_NONE);
        self.braces.clear();
        self.braces.resize(n, BRACE_NONE);
        let mut errors = Vec::new();
        let mut stack: Vec<usize> = Vec::new();

        for i in 0..n {
            let t = &self.tokens[i];
            if t.kind != TokKind::Punct {
                continue;
            }
            match t.text.as_ref() {
                "(" | "[" | "{" => stack.push(i),
                ")" | "]" | "}" => {
                    let want = match t.text.as_ref() {
                        ")" => "(",
                        "]" => "[",
                        _ => "{",
                    };
                    match stack.last() {
                        Some(&o) if self.tokens[o].text.as_ref() == want => {
                            stack.pop();
                            self.pairs[o] = i as u32;
                            self.pairs[i] = o as u32;
                        }
                        Some(&o) => {
                            errors.push(BracketError {
                                index: i,
                                message: format!(
                                    "bracket mismatch: `{}` at line {} closes `{}` opened at line {}",
                                    t.text,
                                    t.line,
                                    self.tokens[o].text,
                                    self.tokens[o].line
                                ),
                            });
                        }
                        None => errors.push(BracketError {
                            index: i,
                            message: format!("unmatched closing `{}` at line {}", t.text, t.line),
                        }),
                    }
                }
                _ => {}
            }
        }
        for o in stack {
            errors.push(BracketError {
                index: o,
                message: format!(
                    "unclosed `{}` opened at line {}",
                    self.tokens[o].text, self.tokens[o].line
                ),
            });
        }

        self.classify_braces();
        errors
    }

    fn classify_braces(&mut self) {
        let n = self.tokens.len();
        let mut open: Vec<usize> = Vec::new();
        let mut decl_header = false;
        for i in 0..n {
            if self.tokens[i].kind == TokKind::Punct {
                match self.tokens[i].text.as_ref() {
                    "{" => {
                        let kind = self.classify_brace(i, &open, decl_header);
                        self.braces[i] = match kind {
                            BraceKind::Block => BRACE_BLOCK,
                            BraceKind::SwitchBlock => BRACE_SWITCH,
                            BraceKind::Collection => BRACE_COLLECTION,
                        };
                        open.push(i);
                        decl_header = false;
                    }
                    "}" => {
                        open.pop();
                    }
                    ";" => decl_header = false,
                    _ => {}
                }
            } else if self.tokens[i].kind == TokKind::Ident {
                if matches!(
                    self.tokens[i].text.as_ref(),
                    "class" | "mixin" | "enum" | "extension"
                ) {
                    decl_header = true;
                }
            }
        }
    }

    fn classify_brace(&self, i: usize, open: &[usize], decl_header: bool) -> BraceKind {
        let Some(prev) = self.prev_sig(i) else {
            return BraceKind::Block;
        };
        let p = self.text(prev);
        let top = open
            .last()
            .and_then(|&o| self.brace_kind(o))
            .unwrap_or(BraceKind::Block);
        let block_after_modifier = p == "async"
            || p == "sync"
            || (p == "*"
                && self
                    .prev_sig(prev)
                    .is_some_and(|q| matches!(self.text(q), "async" | "sync")));
        if p == ")" {
            // `switch (...) {` 的分支体需要 case 缩进；switch 表达式也走
            // 同一分支（其中不会出现 case 关键字，仅影响花括号换行）。
            if let Some(open_paren) = self.matching(prev) {
                if self
                    .prev_sig(open_paren)
                    .is_some_and(|q| self.text(q) == "switch")
                {
                    return BraceKind::SwitchBlock;
                }
            }
            return BraceKind::Block;
        }
        if matches!(p, "else" | "do" | "try" | "finally") || block_after_modifier {
            return BraceKind::Block;
        }
        if p == "=>" {
            return BraceKind::Collection;
        }
        // 命名参数/记录/集合里的 `{` 即使处在 class 头部也必须当集合，
        // 否则 `class User({required var String _name});` 会被拆成块。
        if matches!(p, "(" | "[" | "," | "=") {
            return BraceKind::Collection;
        }
        if p == ":" {
            return if matches!(top, BraceKind::Block | BraceKind::SwitchBlock) {
                BraceKind::Block
            } else {
                BraceKind::Collection
            };
        }
        if matches!(p, ";" | "{" | "}") {
            return if top == BraceKind::Collection {
                BraceKind::Collection
            } else {
                BraceKind::Block
            };
        }
        if decl_header
            || self.is_initializer_body(i)
            || (p == "get" || self.prev_sig(prev).is_some_and(|q| self.text(q) == "get"))
        {
            return BraceKind::Block;
        }
        BraceKind::Collection
    }

    /// 构造函数初始化列表后的函数体：`A() : x = 1, y = 2 { ... }`。
    /// 此时 `{` 前是初始化表达式而不是 `)`，需要向前找到构造参数表的
    /// `)` 后的初始化冒号，否则会被误判成集合字面量。
    fn is_initializer_body(&self, i: usize) -> bool {
        let mut j = i;
        let mut paren = 0usize;
        let mut bracket = 0usize;
        while j > 0 {
            j -= 1;
            match self.text(j) {
                ")" => paren += 1,
                "(" => {
                    if paren == 0 {
                        return false;
                    }
                    paren -= 1;
                }
                "]" => bracket += 1,
                "[" => {
                    if bracket == 0 {
                        return false;
                    }
                    bracket -= 1;
                }
                ";" | "{" | "}" if paren == 0 && bracket == 0 => return false,
                ":" if paren == 0 && bracket == 0 => {
                    return self
                        .prev_sig(j)
                        .is_some_and(|p| self.text(p) == ")");
                }
                _ => {}
            }
        }
        false
    }
}

pub fn unit_metrics(ast: &DartAst) -> (u64, u64, u32) {
    let nodes = ast.tokens.len() as u64;
    let mut decisions = 0u64;
    let mut depth = 0u32;
    let mut cur = 0u32;
    for t in &ast.tokens {
        if t.kind == TokKind::Punct {
            match t.text.as_ref() {
                "(" | "[" | "{" => {
                    cur += 1;
                    depth = depth.max(cur);
                }
                ")" | "]" | "}" => cur = cur.saturating_sub(1),
                "?" | ":" | "=>" | "&&" | "||" | "??" => decisions += 1,
                _ => {}
            }
        } else if t.kind == TokKind::Ident
            && matches!(
                t.text.as_ref(),
                "if" | "for" | "while" | "do" | "switch" | "case" | "&&" | "||" | "??"
            )
        {
            decisions += 1;
        }
    }
    (nodes, decisions, depth)
}

/// 简单、无前缀、无转义、无插值的单引号/双引号字符串 → 内容。
/// 只用于常量折叠；三引号/raw/带 `$`/反斜杠的字符串一律返回 None。
pub fn simple_string(text: &str) -> Option<String> {
    let b = text.as_bytes();
    if b.len() < 2 {
        return None;
    }
    let q = b[0];
    if (q != b'\'' && q != b'"') || b[b.len() - 1] != q {
        return None;
    }
    if b.len() >= 6 && b[1] == q && b[2] == q {
        return None;
    }
    let inner = &text[1..text.len() - 1];
    if inner
        .chars()
        .any(|c| c == '\\' || c == '$' || c == '\n' || c == '\r' || c == q as char)
    {
        return None;
    }
    Some(inner.to_string())
}

pub fn make_simple_string(s: &str) -> Option<String> {
    if s.chars().any(|c| {
        c == '\\'
            || c == '$'
            || c == '\''
            || c == '"'
            || c == '\n'
            || c == '\r'
            || c.is_control()
    }) {
        return None;
    }
    Some(format!("'{s}'"))
}

pub fn tok_ident_start(c: char) -> bool {
    c == '_' || c == '$' || c.is_alphabetic()
}

pub fn tok_ident_continue(c: char) -> bool {
    c == '_' || c == '$' || c.is_alphanumeric()
}