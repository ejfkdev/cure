//! Dart 词法层：覆盖 Dart 1/2/3 的全部 token 形态。
//!
//! 关键点：
//! - 字符串是单个 token（含 `r`/三引号/插值），插值里的嵌套字符串、
//!   注释与 `${...}` 花括号都会被平衡扫描；字符串原文永不重写。
//! - 块注释可嵌套（Dart 与 C 不同）。
//! - 数字支持十进制/十六进制/二进制、小数、指数、`_` 分隔符。
//! - 所有多字符运算符按最长匹配，`>>`/`>>>` 保留原样（泛型嵌套由
//!   打印器/简化器按上下文处理）。

use cure_dart_ast::{TokKind, Token};

#[derive(Clone, Debug)]
pub struct LexError {
    pub line: u32,
    pub col: u32,
    pub message: String,
}

pub fn tokenize(src: &str) -> (Vec<Token>, Vec<LexError>) {
    let mut lx = Lexer {
        src,
        i: 0,
        line: 1,
        col: 1,
        newlines: 0,
        toks: Vec::new(),
        errors: Vec::new(),
    };
    if lx.src.starts_with('\u{feff}') {
        lx.bump();
    }
    if lx.src[lx.i..].starts_with("#!") {
        let start = lx.i;
        while let Some(c) = lx.peek() {
            if c == '\n' {
                break;
            }
            lx.bump();
        }
        let text = lx.slice(start);
        lx.push(TokKind::LineComment, text, 1, 1);
    }
    while lx.i < lx.src.len() {
        let c = lx.peek().unwrap();
        if c == '\n' {
            lx.newlines += 1;
            lx.bump();
            continue;
        }
        if c == ' ' || c == '\t' || c == '\r' {
            lx.bump();
            continue;
        }
        let start = lx.i;
        let line = lx.line;
        let col = lx.col;
        if c == '/' && lx.peek_at(1) == Some('/') {
            lx.bump();
            lx.bump();
            while let Some(ch) = lx.peek() {
                if ch == '\n' {
                    break;
                }
                lx.bump();
            }
            let text = lx.slice(start);
            lx.push(TokKind::LineComment, text, line, col);
            continue;
        }
        if c == '/' && lx.peek_at(1) == Some('*') {
            lx.bump();
            lx.bump();
            let mut depth = 1usize;
            while depth > 0 {
                match (lx.peek(), lx.peek_at(1)) {
                    (Some('/'), Some('*')) => {
                        lx.bump();
                        lx.bump();
                        depth += 1;
                    }
                    (Some('*'), Some('/')) => {
                        lx.bump();
                        lx.bump();
                        depth -= 1;
                    }
                    (Some(_), _) => {
                        lx.bump();
                    }
                    (None, _) => {
                        lx.error(line, col, "unterminated block comment");
                        break;
                    }
                }
            }
            let text = lx.slice(start);
            lx.push(TokKind::BlockComment, text, line, col);
            continue;
        }
        if (c == 'r' || c == 'R') && lx.peek_at(1).is_some_and(|q| q == '\'' || q == '"') {
            lx.bump();
            if !lx.scan_string(true) {
                lx.error(line, col, "unterminated raw string literal");
            }
            let text = lx.slice(start);
            lx.push(TokKind::Str, text, line, col);
            continue;
        }
        if c == '\'' || c == '"' {
            if !lx.scan_string(false) {
                lx.error(line, col, "unterminated string literal");
            }
            let text = lx.slice(start);
            lx.push(TokKind::Str, text, line, col);
            continue;
        }
        if c.is_ascii_digit() || (c == '.' && lx.peek_at(1).is_some_and(|d| d.is_ascii_digit())) {
            lx.scan_number();
            let text = lx.slice(start);
            lx.push(TokKind::Number, text, line, col);
            continue;
        }
        if is_ident_start(c) {
            lx.bump();
            while lx.peek().is_some_and(is_ident_continue) {
                lx.bump();
            }
            let text = lx.slice(start);
            lx.push(TokKind::Ident, text, line, col);
            continue;
        }
        let ops = [
            "...?", "?..", ">>>=", ">>=", "<<=", "~/=", "??=", "...", ">>>", ">>", "<<", "<=",
            ">=", "==", "!=", "&&", "||", "??", "++", "--", "+=", "-=", "*=", "/=", "%=", "&=",
            "|=", "^=", "=>", "?.", "..", "~/",
        ];
        let mut matched = None;
        for op in ops {
            let n = op.chars().count();
            if (0..n).all(|k| lx.peek_at(k) == op.chars().nth(k)) {
                matched = Some(op);
                break;
            }
        }
        if let Some(op) = matched {
            for _ in 0..op.chars().count() {
                lx.bump();
            }
            lx.push(TokKind::Punct, op.to_string(), line, col);
            continue;
        }
        if "(){}[];,:?@#~!+-*/%&|^.<>=".contains(c) {
            lx.bump();
            lx.push(TokKind::Punct, c.to_string(), line, col);
            continue;
        }
        lx.error(line, col, &format!("unexpected character `{c}`"));
        lx.bump();
    }
    (lx.toks, lx.errors)
}

struct Lexer<'a> {
    src: &'a str,
    i: usize,
    line: u32,
    col: u32,
    newlines: u32,
    toks: Vec<Token>,
    errors: Vec<LexError>,
}

impl<'a> Lexer<'a> {
    fn peek(&self) -> Option<char> {
        self.src[self.i..].chars().next()
    }

    fn peek_at(&self, n: usize) -> Option<char> {
        self.src[self.i..].chars().nth(n)
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.i += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn slice(&self, start: usize) -> String {
        self.src[start..self.i].to_string()
    }

    fn push(&mut self, kind: TokKind, text: String, line: u32, col: u32) {
        self.toks.push(Token::new(
            kind,
            text,
            line,
            col,
            std::mem::take(&mut self.newlines),
        ));
    }

    fn error(&mut self, line: u32, col: u32, message: &str) {
        self.errors.push(LexError {
            line,
            col,
            message: message.to_string(),
        });
    }

    /// 当前位置在引号上；raw 前缀已由调用方消费。
    fn scan_string(&mut self, raw: bool) -> bool {
        let Some(q) = self.peek() else {
            return false;
        };
        if q != '\'' && q != '"' {
            return false;
        }
        self.bump();
        let triple = self.peek() == Some(q) && self.peek_at(1) == Some(q);
        if triple {
            self.bump();
            self.bump();
        }
        loop {
            let Some(c) = self.peek() else {
                return false;
            };
            if triple {
                if c == q && self.peek_at(1) == Some(q) && self.peek_at(2) == Some(q) {
                    self.bump();
                    self.bump();
                    self.bump();
                    return true;
                }
            } else if c == q {
                self.bump();
                return true;
            } else if c == '\n' {
                // 单行字符串里的裸换行：报错由调用方处理，保留到行尾。
                return false;
            }
            if !raw && c == '\\' {
                self.bump();
                self.bump();
                continue;
            }
            if !raw && c == '$' {
                match self.peek_at(1) {
                    Some('{') => {
                        self.bump();
                        if !self.scan_interp() {
                            return false;
                        }
                        continue;
                    }
                    Some(ch) if is_ident_start(ch) => {
                        self.bump();
                        while self.peek().is_some_and(is_interp_ident_continue) {
                            self.bump();
                        }
                        continue;
                    }
                    _ => {}
                }
            }
            self.bump();
        }
    }

    /// 当前位置在 `${` 的 `{` 上。
    fn scan_interp(&mut self) -> bool {
        if self.peek() != Some('{') {
            return false;
        }
        self.bump();
        let mut depth = 1usize;
        loop {
            let Some(c) = self.peek() else {
                return false;
            };
            match c {
                '{' => {
                    self.bump();
                    depth += 1;
                }
                '}' => {
                    self.bump();
                    depth -= 1;
                    if depth == 0 {
                        return true;
                    }
                }
                '\'' | '"' => {
                    if !self.scan_string(false) {
                        return false;
                    }
                }
                'r' | 'R'
                    if self.peek_at(1).is_some_and(|q| q == '\'' || q == '"') =>
                {
                    self.bump();
                    if !self.scan_string(true) {
                        return false;
                    }
                }
                '/' if self.peek_at(1) == Some('/') => {
                    while let Some(ch) = self.peek() {
                        if ch == '\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                '/' if self.peek_at(1) == Some('*') => {
                    self.bump();
                    self.bump();
                    let mut d = 1usize;
                    while d > 0 {
                        match (self.peek(), self.peek_at(1)) {
                            (Some('/'), Some('*')) => {
                                self.bump();
                                self.bump();
                                d += 1;
                            }
                            (Some('*'), Some('/')) => {
                                self.bump();
                                self.bump();
                                d -= 1;
                            }
                            (Some(_), _) => {
                                self.bump();
                            }
                            (None, _) => return false,
                        }
                    }
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    fn scan_number(&mut self) {
        if self.peek() == Some('.') {
            self.bump();
            self.scan_digits(10);
        } else {
            self.scan_digits(10);
            if self.peek() == Some('x') || self.peek() == Some('X') {
                self.bump();
                self.scan_digits(16);
                return;
            }
            if self.peek() == Some('b') || self.peek() == Some('B') {
                self.bump();
                self.scan_digits(2);
                return;
            }
            if self.peek() == Some('.') && self.peek_at(1).is_some_and(|c| c.is_ascii_digit()) {
                self.bump();
                self.scan_digits(10);
            }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            let save = self.i;
            let save_line = self.line;
            let save_col = self.col;
            self.bump();
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.bump();
            }
            if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.scan_digits(10);
            } else {
                self.i = save;
                self.line = save_line;
                self.col = save_col;
            }
        }
    }

    fn scan_digits(&mut self, radix: u32) {
        while let Some(c) = self.peek() {
            if c == '_' || c.is_digit(radix) {
                self.bump();
            } else {
                break;
            }
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c == '$' || c.is_alphabetic()
}

fn is_ident_continue(c: char) -> bool {
    c == '_' || c == '$' || c.is_alphanumeric()
}

/// `$identifier` 插值里的标识符不能含 `$`，否则 `'$_body${'...'}'` 会把
/// 第二个 `$` 吞进标识符、导致 `${` 不再被识别。
fn is_interp_ident_continue(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}