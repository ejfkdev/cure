//! Python 词法器：缩进/隐式续行/字符串（含 f/t-string 替换域）扫描。
//!
//! 容错：未知字符产出单字符 Op 由解析器回退原文；词法错误只记录诊断，
//! 永不中止。缩进用「空格 + tab×8」宽度比较（CPython TabError 不在
//! 本工具职责内）。

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokKind {
    Name,
    Number,
    Str,
    Op,
    Newline,
    Indent,
    Dedent,
    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokKind,
    pub text: String,
    pub line: u32,
    /// 1-based column.
    pub col: u32,
    pub start: u32,
    pub end: u32,
    /// 该 token 前是否有换行（语句边界/ASI 无关，供 raw 捕获）。
    pub newline_before: bool,
}

#[derive(Clone, Debug)]
pub struct LexError {
    pub line: u32,
    pub col: u32,
    pub message: String,
}

const OPS: &[&str] = &[
    "**=", "//=", ">>=", "<<=", "...", "==", "!=", "<>", "<=", ">=", ":=", "->", "**", "//", "<<",
    ">>", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "@=", "(", ")", "[", "]", "{", "}", ":",
    ",", ";", ".", "+", "-", "*", "/", "%", "@", "&", "|", "^", "~", "<", ">", "=", "`",
];

pub fn tokenize(src: &str) -> (Vec<Token>, Vec<LexError>) {
    Lexer::new(src).run()
}

struct Lexer<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
    line: u32,
    col: u32,
    toks: Vec<Token>,
    errors: Vec<LexError>,
    indents: Vec<usize>,
    at_line_start: bool,
    line_has_code: bool,
    paren_depth: i32,
    /// 本逻辑行开始处的缩进原文（Parser raw 捕获用）。
    indent_prefix: String,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
            toks: Vec::new(),
            errors: Vec::new(),
            indents: vec![0],
            at_line_start: true,
            line_has_code: false,
            paren_depth: 0,
            indent_prefix: String::new(),
        }
    }

    fn err(&mut self, msg: &str) {
        self.errors.push(LexError {
            line: self.line,
            col: self.col,
            message: msg.into(),
        });
    }

    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn peek_at(&self, off: usize) -> Option<char> {
        self.src[self.pos..].chars().nth(off)
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn push(&mut self, kind: TokKind, text: String, start: usize, line: u32, col: u32, nl: bool) {
        self.toks.push(Token {
            kind,
            text,
            line,
            col,
            start: start as u32,
            end: self.pos as u32,
            newline_before: nl,
        });
    }

    fn run(mut self) -> (Vec<Token>, Vec<LexError>) {
        let mut nl_before = false;
        loop {
            if self.at_line_start && self.paren_depth == 0 {
                self.at_line_start = false;
                if !self.consume_indentation() {
                    // blank / comment-only line：下一物理行仍要按行首处理
                    nl_before = true;
                    self.at_line_start = true;
                    if self.pos >= self.bytes.len() {
                        break;
                    }
                    continue;
                }
            }
            let Some(c) = self.peek() else { break };
            if c == '\n' {
                self.bump();
                if self.paren_depth <= 0 && self.line_has_code {
                    let start = self.pos - 1;
                    self.push(
                        TokKind::Newline,
                        "\n".into(),
                        start,
                        self.line - 1,
                        1,
                        nl_before,
                    );
                    nl_before = false;
                    self.line_has_code = false;
                    self.at_line_start = true;
                } else if self.paren_depth <= 0 {
                    self.at_line_start = true;
                }
                continue;
            }
            if c == '\r' {
                self.bump();
                if self.peek() == Some('\n') {
                    continue;
                }
                continue;
            }
            if c == '\\' && (self.peek_at(1) == Some('\n') || self.peek_at(1) == Some('\r')) {
                self.bump();
                if self.peek() == Some('\r') {
                    self.bump();
                }
                if self.peek() == Some('\n') {
                    self.bump();
                }
                self.at_line_start = false; // 续行：不产生缩进/换行
                nl_before = true;
                continue;
            }
            if c == '#' {
                while let Some(c) = self.peek() {
                    if c == '\n' || c == '\r' {
                        break;
                    }
                    self.bump();
                }
                continue;
            }
            if c.is_whitespace() {
                self.bump();
                continue;
            }
            let start = self.pos;
            let (line, col) = (self.line, self.col);
            if c.is_ascii_digit()
                || (c == '.' && self.peek_at(1).is_some_and(|d| d.is_ascii_digit()))
            {
                self.scan_number();
                self.push(
                    TokKind::Number,
                    self.src[start..self.pos].into(),
                    start,
                    line,
                    col,
                    nl_before,
                );
                nl_before = false;
                self.line_has_code = true;
                continue;
            }
            if let Some(prefix_len) = self.string_prefix_len() {
                self.scan_string(prefix_len);
                self.push(
                    TokKind::Str,
                    self.src[start..self.pos].into(),
                    start,
                    line,
                    col,
                    nl_before,
                );
                nl_before = false;
                self.line_has_code = true;
                continue;
            }
            if is_ident_start(c) {
                self.bump();
                while self.peek().is_some_and(is_ident_continue) {
                    self.bump();
                }
                self.push(
                    TokKind::Name,
                    self.src[start..self.pos].into(),
                    start,
                    line,
                    col,
                    nl_before,
                );
                nl_before = false;
                self.line_has_code = true;
                continue;
            }
            // operators (longest match)
            let rest = &self.src[self.pos..];
            let mut matched = None;
            for op in OPS {
                if rest.starts_with(op) {
                    matched = Some(*op);
                    break;
                }
            }
            match matched {
                Some(op) => {
                    for _ in op.chars() {
                        self.bump();
                    }
                    if matches!(op, "(" | "[" | "{") {
                        self.paren_depth += 1;
                    } else if matches!(op, ")" | "]" | "}") {
                        self.paren_depth -= 1;
                        if self.paren_depth < 0 {
                            self.paren_depth = 0;
                        }
                    }
                    self.push(TokKind::Op, op.into(), start, line, col, nl_before);
                    nl_before = false;
                    self.line_has_code = true;
                }
                None => {
                    let c = self.bump().unwrap_or('\0');
                    self.err(&format!("unknown character {c:?}"));
                    self.push(TokKind::Op, c.to_string(), start, line, col, nl_before);
                    nl_before = false;
                    self.line_has_code = true;
                }
            }
        }
        // 收尾：最后一个逻辑行没有换行
        if self.line_has_code {
            self.push(
                TokKind::Newline,
                "\n".into(),
                self.pos,
                self.line,
                self.col,
                nl_before,
            );
        }
        while self.indents.len() > 1 {
            self.indents.pop();
            self.push(
                TokKind::Dedent,
                String::new(),
                self.pos,
                self.line,
                self.col,
                false,
            );
        }
        self.push(
            TokKind::Eof,
            String::new(),
            self.pos,
            self.line,
            self.col,
            false,
        );
        (self.toks, self.errors)
    }

    /// 处理行首缩进；返回 false 表示本行是空行/纯注释（已消费）。
    fn consume_indentation(&mut self) -> bool {
        let start = self.pos;
        let mut width = 0usize;
        while let Some(c) = self.peek() {
            match c {
                ' ' => {
                    width += 1;
                    self.bump();
                }
                '\t' => {
                    width = (width / 8 + 1) * 8;
                    self.bump();
                }
                '\x0c' => {
                    width = 0;
                    self.bump();
                }
                _ => break,
            }
        }
        // 空行或注释行：不产生 INDENT/DEDENT/NEWLINE
        if self.peek().is_none()
            || self.peek() == Some('\n')
            || self.peek() == Some('\r')
            || self.peek() == Some('#')
        {
            while let Some(c) = self.peek() {
                if c == '\n' {
                    self.bump();
                    break;
                }
                if c == '\r' {
                    self.bump();
                    if self.peek() == Some('\n') {
                        self.bump();
                    }
                    break;
                }
                self.bump();
            }
            return false;
        }
        self.indent_prefix = self.src[start..self.pos].to_string();
        let cur = *self.indents.last().unwrap_or(&0);
        if width > cur {
            self.indents.push(width);
            self.push(
                TokKind::Indent,
                self.indent_prefix.clone(),
                start,
                self.line,
                self.col,
                false,
            );
        } else if width < cur {
            while self.indents.len() > 1 && *self.indents.last().unwrap() > width {
                self.indents.pop();
                self.push(
                    TokKind::Dedent,
                    String::new(),
                    start,
                    self.line,
                    self.col,
                    false,
                );
            }
            if *self.indents.last().unwrap_or(&0) != width {
                self.err("inconsistent indentation");
            }
        }
        true
    }

    /// 当前是否处于字符串前缀 + 引号；返回前缀字符数。
    fn string_prefix_len(&self) -> Option<usize> {
        let mut i = 0;
        let mut n = 0;
        while i < 3 {
            match self.peek_at(i) {
                Some(c)
                    if matches!(c, 'r' | 'R' | 'b' | 'B' | 'u' | 'U' | 'f' | 'F' | 't' | 'T') =>
                {
                    n += 1;
                    i += 1;
                }
                _ => break,
            }
        }
        match self.peek_at(i) {
            Some('\'') | Some('"') => Some(n),
            _ => None,
        }
    }

    fn scan_number(&mut self) {
        if self.peek() == Some('0')
            && matches!(self.peek_at(1), Some('x' | 'X' | 'o' | 'O' | 'b' | 'B'))
        {
            let base = self.peek_at(1).unwrap_or('x').to_ascii_lowercase();
            self.bump();
            self.bump();
            while let Some(c) = self.peek() {
                let ok = match base {
                    'x' => c.is_ascii_hexdigit() || c == '_',
                    'o' => matches!(c, '0'..='7' | '_'),
                    'b' => matches!(c, '0' | '1' | '_'),
                    _ => false,
                };
                if ok {
                    self.bump();
                } else {
                    break;
                }
            }
            if matches!(self.peek(), Some('l' | 'L')) {
                self.bump();
            } else if matches!(self.peek(), Some('j' | 'J')) {
                self.bump();
            }
            return;
        }
        let mut seen_dot = false;
        let mut seen_exp = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '_' {
                self.bump();
            } else if c == '.' && !seen_dot && !seen_exp {
                seen_dot = true;
                self.bump();
            } else if (c == 'e' || c == 'E') && !seen_exp {
                // 指数必须至少跟一位数字：`1else` 要切成 `1` + `else`
                // （CPython 的 tokenizer 同样规则）。
                let mut k = 1;
                if matches!(self.peek_at(k), Some('+' | '-')) {
                    k += 1;
                }
                if !self.peek_at(k).is_some_and(|d| d.is_ascii_digit()) {
                    break;
                }
                seen_exp = true;
                self.bump();
                if matches!(self.peek(), Some('+' | '-')) {
                    self.bump();
                }
            } else {
                break;
            }
        }
        if matches!(self.peek(), Some('l' | 'L')) {
            self.bump();
        } else if matches!(self.peek(), Some('j' | 'J')) {
            self.bump();
        }
    }

    /// 从前缀后（引号处）扫描整个字符串 token。
    fn scan_string(&mut self, prefix_len: usize) {
        for _ in 0..prefix_len {
            self.bump();
        }
        let prefix = &self.src[self.pos.saturating_sub(prefix_len)..self.pos];
        let is_f = prefix.contains(['f', 'F', 't', 'T']);
        let quote = self.peek().unwrap_or('"');
        let triple = self.peek_at(1) == Some(quote) && self.peek_at(2) == Some(quote);
        let qn = if triple { 3 } else { 1 };
        for _ in 0..qn {
            self.bump();
        }
        loop {
            let Some(c) = self.peek() else {
                self.err("unterminated string literal");
                return;
            };
            if c == '\\' {
                // f-string 里花括号仍按语法解析：`\{{` 是反斜杠 + 转义
                // 花括号（`{{`），不能把 `{` 当成被反斜杠吃掉的字符。
                if is_f && matches!(self.peek_at(1), Some('{' | '}')) {
                    self.bump();
                    continue;
                }
                // raw 字符串里反斜杠仍阻止引号终止
                self.bump();
                if self.peek().is_some() {
                    self.bump();
                }
                continue;
            }
            if c == quote {
                if !triple {
                    self.bump();
                    return;
                }
                if self.peek_at(1) == Some(quote) && self.peek_at(2) == Some(quote) {
                    self.bump();
                    self.bump();
                    self.bump();
                    return;
                }
                self.bump();
                continue;
            }
            if is_f && c == '{' {
                if self.peek_at(1) == Some('{') {
                    self.bump();
                    self.bump();
                    continue;
                }
                self.bump();
                self.skip_interp_field();
                continue;
            }
            if is_f && c == '}' && self.peek_at(1) == Some('}') {
                self.bump();
                self.bump();
                continue;
            }
            self.bump();
        }
    }

    /// 跳过 f-string 的 `{...}` 替换域（含嵌套字符串/注释/花括号）。
    /// 深度 0 的 `:` 之后是 **format spec**：`#`/引号都是字面文本，
    /// 只有嵌套 `{...}` 是替换域（`f"{x:#x}"` 曾把 `#` 当注释吞到 EOF）。
    fn skip_interp_field(&mut self) {
        let mut nest = 0i32;
        let mut in_format = false;
        loop {
            let Some(c) = self.peek() else { return };
            if in_format {
                match c {
                    '{' => {
                        self.bump();
                        self.skip_interp_field();
                    }
                    '}' => {
                        self.bump();
                        return;
                    }
                    _ => {
                        self.bump();
                    }
                }
                continue;
            }
            match c {
                '(' | '[' | '{' => {
                    nest += 1;
                    self.bump();
                }
                ')' | ']' => {
                    if nest > 0 {
                        nest -= 1;
                    }
                    self.bump();
                }
                '}' => {
                    if nest > 0 {
                        nest -= 1;
                        self.bump();
                    } else {
                        self.bump();
                        return;
                    }
                }
                ':' if nest == 0 => {
                    in_format = true;
                    self.bump();
                }
                '#' => {
                    while let Some(c) = self.peek() {
                        if c == '\n' || c == '\r' {
                            break;
                        }
                        self.bump();
                    }
                }
                '\'' | '"' => {
                    self.scan_string_plain();
                }
                'r' | 'R' | 'b' | 'B' | 'u' | 'U' | 'f' | 'F' | 't' | 'T' => {
                    if self.string_prefix_len().is_some() {
                        let n = self.string_prefix_len().unwrap_or(0);
                        self.scan_string(n);
                    } else {
                        self.bump();
                    }
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    /// 替换域内的裸字符串（无前缀）扫描，避免把内部引号当域结束。
    fn scan_string_plain(&mut self) {
        let quote = self.peek().unwrap_or('"');
        let triple = self.peek_at(1) == Some(quote) && self.peek_at(2) == Some(quote);
        let qn = if triple { 3 } else { 1 };
        for _ in 0..qn {
            self.bump();
        }
        loop {
            let Some(c) = self.peek() else { return };
            if c == '\\' {
                self.bump();
                if self.peek().is_some() {
                    self.bump();
                }
                continue;
            }
            if c == quote {
                if !triple {
                    self.bump();
                    return;
                }
                if self.peek_at(1) == Some(quote) && self.peek_at(2) == Some(quote) {
                    self.bump();
                    self.bump();
                    self.bump();
                    return;
                }
            }
            self.bump();
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic() || (!c.is_ascii() && !c.is_whitespace())
}

fn is_ident_continue(c: char) -> bool {
    c == '_' || c.is_alphanumeric() || (!c.is_ascii() && !c.is_whitespace())
}
