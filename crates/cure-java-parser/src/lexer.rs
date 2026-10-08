//! 容错式 Java 词法器。
//!
//! 设计目标：**永不失败**。非法输入产出 `Tok::Error` 并继续（如未终止字符串
//! 吞到行尾），保证上层语法解析总能拿到完整 token 流做恢复。

/// token 种类（携带原文）。
#[derive(Clone, PartialEq, Debug)]
pub enum Tok<'a> {
    /// 标识符（借用源——词法期零分配；37 万文件 × 每文件数千 token
    /// 曾是 malloc 大头）
    Ident(&'a str),
    /// 运算符/标点（规范形态，如 ">>" "=="; 单字符皆为一字节）。
    Punct(&'static str),
    /// 数值字面量原文（含前后缀，如 "0x1FL"、"1.5e2f"；借用源）。
    Num(&'a str),
    Char(&'a str),
    Str(&'a str),
    /// 文本块内容（已去掉两侧 `"""` 与首行换行；需加工，保持 String）。
    TextBlock(String),
    /// 词法错误（原文片段；罕见）。
    Error(String),
    Eof,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Token<'a> {
    pub tok: Tok<'a>,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

impl Token<'_> {
    pub fn is_punct(&self, s: &str) -> bool {
        matches!(&self.tok, Tok::Punct(p) if *p == s)
    }
    pub fn is_ident(&self, s: &str) -> bool {
        matches!(&self.tok, Tok::Ident(i) if *i == s)
    }
    pub fn text(&self) -> String {
        match &self.tok {
            Tok::Ident(i) => i.to_string(),
            Tok::Punct(p) => (*p).to_string(),
            Tok::Num(n) => n.to_string(),
            Tok::Char(c) => c.to_string(),
            Tok::Str(s) => s.to_string(),
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

pub fn lex(src: &str) -> (Vec<Token<'_>>, Vec<LexError>) {
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
    toks: Vec<Token<'a>>,
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
        // 真实 EOF 才停：输入里的 NUL 字节是普通字符，必须推进
        // （否则字符串/注释循环在 NUL 上死循环）
        let Some(&c) = self.b.get(self.pos) else { return 0 };
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
                // 行终止符 = LF / CR / CRLF（JLS 3.4）——CR-only 文件的
                // 行注释曾只认 LF，从头吞到 EOF：整文件内容静默丢失
                //（Ops_cr.java 304,893 字节 → 1 字节，无告警）
                while let Some(&c2) = self.b.get(self.pos) {
                    if c2 == b'\n' || c2 == b'\r' {
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
                    tok: Tok::Ident(&self.s[start..self.pos]),
                    start,
                    end: self.pos,
                    line,
                    col,
                });
                continue;
            }
            // 数字（含 .5 形式：'.' 后是数字即浮点字面量开头——x.5 本就
            // 非法，无需 prev_ends_primary 守卫；return .5f——jdk-sources
            // GroupLayout 抓获：return 是 Ident 曾误判成员访问）
            if c.is_ascii_digit()
                || (c == b'.' && self.peek(1).is_ascii_digit())
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

    fn punct(&mut self) -> Option<&'static str> {
        const THREE: [&str; 5] = [">>>=", ">>>", "<<=", ">>=", "..."];
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
            self.toks.push(Token {
                tok: Tok::Num(&self.s[start..self.pos]),
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
                tok: Tok::Num(&self.s[start..self.pos]),
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
        } else if self.peek(0) == b'.'
            && matches!(
                self.peek(1),
                0 | b'f' | b'F' | b'd' | b'D' | b';' | b',' | b')' | b']' | b' ' | b'\n' | b'\t'
            )
        {
            // 尾点浮点（1. / 2.f——JLS 一个 double 字面量 token；后面是
            // 终结符/后缀，不是成员访问——jdk-sources StackMoveTest）
            is_float = true;
            self.bump();
        } else if self.peek(0) == b'.'
            && matches!(
                self.peek(1),
                b'+' | b'-' | b'*' | b'/' | b'%' | b'}' | b'=' | b'<' | b'>' | b'&' | b'|' | b'^'
            )
        {
            // 尾点浮点后跟运算符/闭花括号（`3./2` / `{…, 3.}`——SymmLQTest
            // 抓获：白名单外曾读成 int 3 → 3./2 折成整数除法 1，
            // before 1.5 → after 1 值错）
            is_float = true;
            self.bump();
        } else if self.peek(0) == b'.'
            && (self.peek(1) | 0x20) == b'e'
            && (self.peek(2).is_ascii_digit()
                || ((self.peek(2) == b'+' || self.peek(2) == b'-')
                    && self.peek(3).is_ascii_digit()))
        {
            // 整数.指数（2.e-7 / 1.E+3——commons-math 抓获：尾点分支
            // 白名单不含 e/E → 2 与 .e-7 撕裂成 2.e - 7 二元减法，
            // 值错 + 触发 EigenDecompositionTest 2M 错误风暴）
            is_float = true;
            self.bump();
        }
        let _ = &is_float;
        // 指数：e 后跟数字，或符号后跟数字（1.5e-3 / 1E+10）
        if (self.peek(0) | 0x20) == b'e'
            && (self.peek(1).is_ascii_digit()
                || ((self.peek(1) == b'+' || self.peek(1) == b'-')
                    && self.peek(2).is_ascii_digit()))
        {
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
            tok: Tok::Num(&self.s[start..self.pos]),
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
            tok: Tok::Char(&self.s[start..self.pos]),
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
            tok: Tok::Str(&self.s[start..self.pos]),
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
            // 转义对优先于闭合判定（javac 语义，GJF StringWrapperTest 实证）：
            // \" 中的引号不计入 """ 闭合 run——\""" 是 转义引号+两个内容引号，
            // 不是闭合。否则提前闭合会把块剩余内容当代码再解析（错误风暴）。
            if self.peek(0) == b'\\' {
                self.bump(); // \
                self.bump(); // 被转义字符（EOF 处 bump 自行无害）
                continue;
            }
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
