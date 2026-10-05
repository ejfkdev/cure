//! 容错式 Java 词法器。
//!
//! 设计目标：**永不失败**。非法输入产出 `Tok::Error` 并继续（如未终止字符串
//! 吞到行尾），保证上层语法解析总能拿到完整 token 流做恢复。

/// token 种类（携带原文）。
#[derive(Clone, PartialEq, Debug)]
pub enum Tok {
    Ident(String),
    /// 运算符/标点（规范形态，如 ">>" "=="; 单字符皆为一字节）。
    Punct(&'static str),
    /// 数值字面量原文（含前后缀，如 "0x1FL"、"1.5e2f"）。
    Num(String),
    Char(String),
    Str(String),
    /// 文本块内容（已去掉两侧 `"""` 与首行换行）。
    TextBlock(String),
    /// 词法错误（原文片段）。
    Error(String),
    Eof,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    pub tok: Tok,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn is_punct(&self, s: &str) -> bool {
        matches!(&self.tok, Tok::Punct(p) if *p == s)
    }
    pub fn is_ident(&self, s: &str) -> bool {
        matches!(&self.tok, Tok::Ident(i) if i == s)
    }
    pub fn text(&self) -> String {
        match &self.tok {
            Tok::Ident(i) => i.clone(),
            Tok::Punct(p) => (*p).to_string(),
            Tok::Num(n) => n.clone(),
            Tok::Char(c) => c.clone(),
            Tok::Str(s) => s.clone(),
            Tok::TextBlock(_) => String::new(),
            Tok::Error(e) => e.clone(),
            Tok::Eof => String::new(),
        }
    }
}

/// 词法错误（位置 1-based）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LexError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

pub fn lex(src: &str) -> (Vec<Token>, Vec<LexError>) {
    let mut lx = Lexer {
        b: src.as_bytes(),
        s: src,
        pos: 0,
        line: 1,
        col: 1,
        toks: Vec::new(),
        errs: Vec::new(),
    };
    lx.run();
    (lx.toks, lx.errs)
}

struct Lexer<'a> {
    b: &'a [u8],
    s: &'a str,
    pos: usize,
    line: usize,
    col: usize,
    toks: Vec<Token>,
    errs: Vec<LexError>,
}

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c == b'$' || c >= 0x80
}
fn is_ident_part(c: u8) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

impl<'a> Lexer<'a> {
    fn peek(&self, off: usize) -> u8 {
        *self.b.get(self.pos + off).unwrap_or(&0)
    }
    fn err(&mut self, start: (usize, usize), msg: &str) {
        self.errs.push(LexError {
            line: start.0,
            col: start.1,
            message: msg.into(),
        });
    }

    fn bump(&mut self) -> u8 {
        let c = self.peek(0);
        if c == 0 {
            return 0;
        }
        self.pos += 1;
        if c == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        c
    }

    fn run(&mut self) {
        loop {
            let (line, col) = (self.line, self.col);
            let start = self.pos;
            let Some(&c) = self.b.get(self.pos) else { break };
            // 空白
            if c.is_ascii_whitespace() {
                self.bump();
                continue;
            }
            // 注释
            if c == b'/' && self.peek(1) == b'/' {
                while let Some(&c2) = self.b.get(self.pos) {
                    if c2 == b'\n' {
                        break;
                    }
                    self.bump();
                }
                continue;
            }
            if c == b'/' && self.peek(1) == b'*' {
                self.bump();
                self.bump();
                let mut closed = false;
                while let Some(&c2) = self.b.get(self.pos) {
                    if c2 == b'*' && self.peek(1) == b'/' {
                        self.bump();
                        self.bump();
                        closed = true;
                        break;
                    }
                    self.bump();
                }
                if !closed {
                    self.err((line, col), "unterminated block comment");
                }
                continue;
            }
            // 标识符 / 关键字
            if is_ident_start(c) {
                while let Some(&c2) = self.b.get(self.pos) {
                    if is_ident_part(c2) {
                        self.bump();
                    } else {
                        break;
                    }
                }
                self.toks.push(Token {
                    tok: Tok::Ident(self.s[start..self.pos].to_string()),
                    start,
                    end: self.pos,
                    line,
                    col,
                });
                continue;
            }
            // 数字（含 .5 形式：仅当 '.' 后是数字且前一个 token 不是可接成员访问的东西）
            if c.is_ascii_digit()
                || (c == b'.' && self.peek(1).is_ascii_digit() && !self.prev_ends_primary())
            {
                self.number(start, line, col);
                continue;
            }
            // 字符
            if c == b'\'' {
                self.char_lit(start, line, col);
                continue;
            }
            // 文本块 / 字符串
            if c == b'"' {
                if self.peek(1) == b'"' && self.peek(2) == b'"' {
                    self.text_block(start, line, col);
                } else {
                    self.string_lit(start, line, col);
                }
                continue;
            }
            // 运算符（最长匹配）
            if let Some(p) = self.punct() {
                self.toks.push(Token {
                    tok: Tok::Punct(p),
                    start,
                    end: self.pos,
                    line,
                    col,
                });
                continue;
            }
            // 未知字符 → Error token
            self.err((line, col), &format!("unexpected character {:?}", c as char));
            self.bump();
            self.toks.push(Token {
                tok: Tok::Error(self.s[start..self.pos].to_string()),
                start,
                end: self.pos,
                line,
                col,
            });
        }
        let (line, col) = (self.line, self.col);
        self.toks.push(Token {
            tok: Tok::Eof,
            start: self.pos,
            end: self.pos,
            line,
            col,
        });
    }

    /// 前一个 token 是否以 primary 结尾（决定 `.` 后接数字的归类）。
    fn prev_ends_primary(&self) -> bool {
        match self.toks.last().map(|t| &t.tok) {
            Some(Tok::Ident(_)) | Some(Tok::Str(_)) | Some(Tok::Char(_)) => true,
            Some(Tok::Num(_)) => true,
            Some(Tok::Punct(p)) => matches!(*p, ")" | "]" | "}"),
            _ => false,
        }
    }

    fn punct(&mut self) -> Option<&'static str> {
        const THREE: [&str; 4] = [">>>=", "<<=", ">>=", "..."];
        const TWO: [&str; 18] = [
            "==", "!=", "<=", ">=", "&&", "||", "++", "--", "+=", "-=", "*=", "/=", "%=", "&=",
            "|=", "^=", ">>", "<<",
        ];
        const ONE: &str = "+-*/%=<>!~&|^?:;,.(){}[]";
        let rest = &self.s[self.pos..];
        for p in THREE {
            if rest.starts_with(p) {
                for _ in 0..p.len() {
                    self.bump();
                }
                return Some(p);
            }
        }
        if rest.starts_with("::") {
            self.bump();
            self.bump();
            return Some("::");
        }
        if rest.starts_with("->") {
            self.bump();
            self.bump();
            return Some("->");
        }
        for p in TWO {
            if rest.starts_with(p) {
                self.bump();
                self.bump();
                return Some(p);
            }
        }
        let c = self.peek(0);
        if ONE.as_bytes().contains(&c) {
            self.bump();
            return match c {
                b'+' => Some("+"),
                b'-' => Some("-"),
                b'*' => Some("*"),
                b'/' => Some("/"),
                b'%' => Some("%"),
                b'=' => Some("="),
                b'<' => Some("<"),
                b'>' => Some(">"),
                b'!' => Some("!"),
                b'~' => Some("~"),
                b'&' => Some("&"),
                b'|' => Some("|"),
                b'^' => Some("^"),
                b'?' => Some("?"),
                b':' => Some(":"),
                b';' => Some(";"),
                b',' => Some(","),
                b'.' => Some("."),
                b'(' => Some("("),
                b')' => Some(")"),
                b'{' => Some("{"),
                b'}' => Some("}"),
                b'[' => Some("["),
                b']' => Some("]"),
                _ => Some("@"),
            };
        }
        if c == b'@' {
            self.bump();
            return Some("@");
        }
        None
    }

    fn number(&mut self, start: usize, line: usize, col: usize) {
        if self.peek(0) == b'0' && (self.peek(1) | 0x20) == b'x' {
            // 十六进制整数或浮点
            self.bump();
            self.bump();
            while let Some(&c2) = self.b.get(self.pos) {
                if c2.is_ascii_hexdigit() || c2 == b'_' {
                    self.bump();
                } else if c2 == b'.' {
                    self.bump();
                } else if (c2 | 0x20) == b'p' {
                    self.bump();
                } else if (c2 == b'+' || c2 == b'-') && (self.s.as_bytes()[self.pos - 1] | 0x20) == b'p' {
                    self.bump();
                } else {
                    break;
                }
            }
            self.suffix();
            let text = self.s[start..self.pos].to_string();
            self.toks.push(Token {
                tok: Tok::Num(text),
                start,
                end: self.pos,
                line,
                col,
            });
            return;
        }
        if self.peek(0) == b'0' && (self.peek(1) | 0x20) == b'b' {
            self.bump();
            self.bump();
            while let Some(&c2) = self.b.get(self.pos) {
                if c2 == b'0' || c2 == b'1' || c2 == b'_' {
                    self.bump();
                } else {
                    break;
                }
            }
            self.suffix();
            self.toks.push(Token {
                tok: Tok::Num(self.s[start..self.pos].to_string()),
                start,
                end: self.pos,
                line,
                col,
            });
            return;
        }
        // 十进制整数/浮点
        while let Some(&c2) = self.b.get(self.pos) {
            if c2.is_ascii_digit() || c2 == b'_' {
                self.bump();
            } else {
                break;
            }
        }
        let mut is_float = false;
        if self.peek(0) == b'.' && self.peek(1).is_ascii_digit() {
            is_float = true;
            self.bump();
            while let Some(&c2) = self.b.get(self.pos) {
                if c2.is_ascii_digit() || c2 == b'_' {
                    self.bump();
                } else {
                    break;
                }
            }
        }
        let _ = &is_float;
        if (self.peek(0) | 0x20) == b'e' && self.peek(1).is_ascii_digit() {
            is_float = true;
            self.bump();
            if self.peek(0) == b'+' || self.peek(0) == b'-' {
                self.bump();
            }
            while let Some(&c2) = self.b.get(self.pos) {
                if c2.is_ascii_digit() || c2 == b'_' {
                    self.bump();
                } else {
                    break;
                }
            }
        }
        if is_float {
            self.suffix();
        } else if let Some(&c2) = self.b.get(self.pos) {
            // 整数后缀 L/l；f/F/d/D 也可能出现在整数上（视为浮点）
            if c2 == b'L' || c2 == b'l' {
                self.bump();
            } else if (c2 | 0x20) == b'f' || (c2 | 0x20) == b'd' {
                is_float = true;
                let _ = is_float;
                self.suffix();
            }
        }
        self.toks.push(Token {
            tok: Tok::Num(self.s[start..self.pos].to_string()),
            start,
            end: self.pos,
            line,
            col,
        });
    }

    fn suffix(&mut self) {
        if let Some(&c2) = self.b.get(self.pos) {
            if c2 == b'L' || c2 == b'l' || (c2 | 0x20) == b'f' || (c2 | 0x20) == b'd' {
                self.bump();
            }
        }
    }

    fn char_lit(&mut self, start: usize, line: usize, col: usize) {
        self.bump(); // '
        let mut closed = false;
        while let Some(&c2) = self.b.get(self.pos) {
            if c2 == b'\n' {
                break;
            }
            if c2 == b'\\' {
                self.bump();
                self.bump();
                continue;
            }
            if c2 == b'\'' {
                self.bump();
                closed = true;
                break;
            }
            self.bump();
        }
        if !closed {
            self.err((line, col), "unterminated char literal");
            while let Some(&c2) = self.b.get(self.pos) {
                if c2 == b'\n' {
                    break;
                }
                self.bump();
            }
        }
        self.toks.push(Token {
            tok: Tok::Char(self.s[start..self.pos].to_string()),
            start,
            end: self.pos,
            line,
            col,
        });
    }

    fn string_lit(&mut self, start: usize, line: usize, col: usize) {
        self.bump(); // "
        let mut closed = false;
        while let Some(&c2) = self.b.get(self.pos) {
            if c2 == b'\n' {
                break;
            }
            if c2 == b'\\' {
                self.bump();
                if self.peek(0) == 0 {
                    break;
                }
                self.bump();
                continue;
            }
            if c2 == b'"' {
                self.bump();
                closed = true;
                break;
            }
            self.bump();
        }
        if !closed {
            self.err((line, col), "unterminated string literal");
            while let Some(&c2) = self.b.get(self.pos) {
                if c2 == b'\n' {
                    break;
                }
                self.bump();
            }
        }
        self.toks.push(Token {
            tok: Tok::Str(self.s[start..self.pos].to_string()),
            start,
            end: self.pos,
            line,
            col,
        });
    }

    fn text_block(&mut self, start: usize, line: usize, col: usize) {
        self.bump();
        self.bump();
        self.bump(); // """
        // 吃掉紧跟的换行
        if self.peek(0) == b'\r' {
            self.bump();
        }
        if self.peek(0) == b'\n' {
            self.bump();
        }
        let content_start = self.pos;
        let mut closed = false;
        while self.pos < self.b.len() {
            if self.peek(0) == b'"' && self.peek(1) == b'"' && self.peek(2) == b'"' {
                closed = true;
                break;
            }
            self.bump();
        }
        let content = if closed {
            self.s[content_start..self.pos].to_string()
        } else {
            self.err((line, col), "unterminated text block");
            self.s[content_start..self.pos].to_string()
        };
        if closed {
            self.bump();
            self.bump();
            self.bump();
        }
        self.toks.push(Token {
            tok: Tok::TextBlock(content),
            start,
            end: self.pos,
            line,
            col,
        });
    }
}
