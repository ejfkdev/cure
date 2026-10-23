//! TS/JS 词法分析器。
//!
//! JS 词法的三个雷区：
//! - **正则 vs 除法**：`/` 的双重身份。按「前一有效 token」启发式判
//!   （表达式结尾符之后是除法；`}` 按 lexer 的花括号来源追踪——
//!   块式 `{`（前驱是 `)`/`else`/`do`/`try`/`=>`/`;` 等）闭合后允许
//!   正则，对象字面量式 `{`（前驱是 `=`/`(`/`,`/值）闭合后是除法）。
//! - **模板字面量嵌套**：`` `a${ `b${x}` }c` `` —— 模板模式栈 +
//!   插值内花括号深度追踪。`TemplateQuasi` token 承载 quasi 原文
//!   （`is_end` 区分 `${` 续接与 `` ` `` 终结）。
//! - **自动分号插入（ASI）**：token 记录 `newline_before`，语法层的
//!   restricted production（return/throw/break/continue 换行）与
//!   「换行即隐式分号」判定由 parser 依据该标志执行。
//!
//! 容错：坏 token 记错误跳过继续；未终结字符串/模板/正则在边界截断。

/// 词法 token。
#[derive(Clone, PartialEq, Debug)]
pub enum TokKind {
    /// 标识符 / 关键字（上下文关键字由 parser 判定——
    /// `of`/`as`/`satisfies`/`async` 在不同位置身份不同）。
    Ident,
    /// 标点/运算符原文（`=>` `?.` `??=` `>>>=` `...` 等）。
    Punct,
    /// 数值字面量（原文保留：0x1F / 1_000 / 1.5e2 / 12n）。
    Num,
    /// 字符串字面量（原文含引号）。
    Str,
    /// 正则字面量（原文含 flags）。
    Regex,
    /// 模板 quasi。is_end=false：`${` 已消费、后随插值表达式；
    /// is_end=true：终结。cont=true 表示这是**外围模板的续接段**（由
    /// 插值终结 `}` 产出——绝不可能是标签模板的开始）；cont=false
    /// 是新模板开始（`tag`…` 的 tag 判定位）。
    TemplateQuasi { is_end: bool, cont: bool },
    Eof,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    pub kind: TokKind,
    /// 原文（Num/Str/Regex 含定界符与全部原文；Ident/Punct 为自身）。
    pub text: String,
    pub line: usize,
    pub col: usize,
    /// token 前是否有换行（ASI 判定）。
    pub newline_before: bool,
    /// 源码字节偏移（原文区间捕获：Raw/JSX/模式/签名）。
    pub start: usize,
    pub end: usize,
}

/// 词法错误。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LexError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

/// 前一 token 的形态归纳（正则/花括号启发式）。
#[derive(Clone, Copy, PartialEq, Debug)]
enum Prev {
    Start,
    Ident,
    Num,
    Str,
    Regex,
    Template,
    CloseParen,
    CloseBracket,
    CloseBrace,
    /// 关键字（regex 引导词 + 语句引导词——prev_text 携带原文）。
    Keyword,
    Punct,
}

/// 正则引导关键字（后随表达式位置——`/re/` 可能）。
const REGEX_LEAD: &[&str] = &[
    "return", "typeof", "instanceof", "in", "of", "new", "delete", "void",
    "case", "do", "else", "yield", "await", "throw", "if", "while", "for",
    "switch", "try", "finally",
];

/// `{` 块式（vs 对象字面量式）判定的语句关键字。
const BLOCK_KW: &[&str] = &["else", "do", "try", "finally"];

pub struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    line: usize,
    line_start: usize,
    prev: Prev,
    prev_text: String,
    /// 最近被闭合的 `{` 是否块式（正则判定的 `}` 分支）。
    last_brace_block: bool,
    /// 模板插值模式栈：每层的花括号深度（0 = 该层插值的 `}` 即终结）。
    template_stack: Vec<i32>,
    /// `{` 块式栈（正则启发的 `}` 分支）。
    brace_stack: Vec<bool>,
    errors: Vec<LexError>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Lexer {
            src: src.as_bytes(),
            pos: 0,
            line: 1,
            line_start: 0,
            prev: Prev::Start,
            prev_text: String::new(),
            last_brace_block: true,
            template_stack: Vec::new(),
            brace_stack: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// 全量词法（容错）。**零进展防御**：非 Eof token 必须消费 ≥1 字节
    /// ——违反即跳过该字节并记错（异常转义/未来词法 bug 的保险丝；
    /// test262 曾因空 ident 零消费进入无限 token 累积 → 系统级 OOM）。
    pub fn tokenize(mut self) -> (Vec<Token>, Vec<LexError>) {
        let mut toks = Vec::new();
        loop {
            let before = self.pos;
            let t = self.next_token();
            let is_eof = t.kind == TokKind::Eof;
            toks.push(t);
            if is_eof {
                break;
            }
            if self.pos == before {
                self.errors.push(LexError {
                    line: self.line,
                    col: self.pos - self.line_start + 1,
                    message: format!("zero-progress token at byte {before} skipped"),
                });
                // 按整 UTF-8 字符跳（单字节会落在多字节字符中间——
                // 后续 token 起点非字符边界，raw 切片 panic）
                self.pos += utf8_len(self.peek()).max(1);
            }
        }
        (toks, self.errors)
    }

    fn err(&mut self, msg: &str) {
        self.errors.push(LexError {
            line: self.line,
            col: self.pos - self.line_start + 1,
            message: msg.to_string(),
        });
    }

    fn peek(&self) -> u8 {
        *self.src.get(self.pos).unwrap_or(&0)
    }
    fn peek2(&self) -> u8 {
        *self.src.get(self.pos + 1).unwrap_or(&0)
    }
    fn peek3(&self) -> u8 {
        *self.src.get(self.pos + 2).unwrap_or(&0)
    }
    fn bump(&mut self) -> u8 {
        let c = self.peek();
        self.pos += 1;
        if c == b'\n' {
            self.line += 1;
            self.line_start = self.pos;
        }
        c
    }
    fn at_end(&self) -> bool {
        self.pos >= self.src.len()
    }

    /// 跳过空白与注释；返回是否遇到换行。
    fn skip_trivia(&mut self) -> bool {
        let mut newline = false;
        loop {
            if self.at_end() {
                return newline;
            }
            match self.peek() {
                b' ' | b'\t' | b'\r' | 0x0b | 0x0c => {
                    self.pos += 1;
                }
                c2 if c2 >= 0x80 => {
                    // Unicode 空白（NBSP/U+2028/U+FEFF 等）：解码判定
                    let l = utf8_len(c2);
                    let end = (self.pos + l).min(self.src.len());
                    let ch = String::from_utf8_lossy(&self.src[self.pos..end])
                        .chars()
                        .next()
                        .unwrap_or('\u{fffd}');
                    if ch.is_whitespace() || ch == '\u{feff}' {
                        self.pos = end;
                    } else {
                        return newline;
                    }
                }
                b'\n' => {
                    newline = true;
                    self.bump();
                }
                b'\\' if self.peek2() == b'\n' => {
                    // 行续接（反斜杠换行）——不产生换行语义
                    self.pos += 2;
                    self.line += 1;
                    self.line_start = self.pos;
                }
                b'/' if self.peek2() == b'/' => {
                    while !self.at_end() && self.peek() != b'\n' {
                        self.pos += 1;
                    }
                }
                b'/' if self.peek2() == b'*' => {
                    self.pos += 2;
                    while !self.at_end() {
                        if self.peek() == b'*' && self.peek2() == b'/' {
                            self.pos += 2;
                            break;
                        }
                        if self.peek() == b'\n' {
                            newline = true;
                        }
                        self.bump();
                    }
                }
                _ => return newline,
            }
        }
    }

    fn next_token(&mut self) -> Token {
        let newline = self.skip_trivia();
        let line = self.line;
        let col = self.pos - self.line_start + 1;
        let byte_start = self.pos;
        if self.at_end() {
            return Token {
                kind: TokKind::Eof,
                text: String::new(),
                line,
                col,
                newline_before: newline,
                start: byte_start,
                end: byte_start,
            };
        }
        let c = self.peek();

        // ---- 模板字面量通道 ----
        // `` ` ``：开模板（嵌套模板在插值内再入一层）
        if c == b'`' {
            self.bump();
            let (q, is_end) = self.scan_quasi();
            if !is_end {
                self.template_stack.push(0);
            }
            self.set_prev(Prev::Template, "");
            let end = self.pos;
            return Token {
                kind: TokKind::TemplateQuasi { is_end, cont: false },
                text: q,
                line,
                col,
                newline_before: newline,
                start: byte_start,
                end,
            };
        }
        // 插值内深度 0 的 `}`：终结插值，扫下一 quasi（cont=true——
        // 外围模板的续接段）
        if c == b'}' && self.template_stack.last() == Some(&0) {
            self.bump();
            let (q, is_end) = self.scan_quasi();
            if is_end {
                self.template_stack.pop();
            } else {
                *self.template_stack.last_mut().unwrap() = 0;
            }
            self.set_prev(Prev::Template, "");
            let end = self.pos;
            return Token {
                kind: TokKind::TemplateQuasi { is_end, cont: true },
                text: q,
                line,
                col,
                newline_before: newline,
                start: byte_start,
                end,
            };
        }
        // 插值内的 `{`/`}`：深度维护
        if c == b'{' && !self.template_stack.is_empty() {
            *self.template_stack.last_mut().unwrap() += 1;
            self.bump();
            self.set_prev(Prev::Punct, "{");
            return self.mk(TokKind::Punct, "{", line, col, newline, byte_start);
        }
        if c == b'}' && !self.template_stack.is_empty() {
            *self.template_stack.last_mut().unwrap() -= 1;
            self.bump();
            self.set_prev(Prev::CloseBrace, "}");
            return self.mk(TokKind::Punct, "}", line, col, newline, byte_start);
        }

        let kind;
        let text: String;
        match c {
            b'{' => {
                let block_like = self.brace_block_like();
                self.last_brace_block_push(block_like);
                self.bump();
                kind = TokKind::Punct;
                text = "{".to_string();
                self.set_prev(Prev::Punct, "{");
            }
            b'}' => {
                self.last_brace_block = self.pop_brace_kind();
                self.bump();
                kind = TokKind::Punct;
                text = "}".to_string();
                self.set_prev(Prev::CloseBrace, "}");
            }
            b'/' => {
                if self.regex_allowed() {
                    let r = self.scan_regex();
                    self.set_prev(Prev::Regex, "");
                    let end = self.pos;
                    return Token {
                        kind: TokKind::Regex,
                        text: r,
                        line,
                        col,
                        newline_before: newline,
                        start: byte_start,
                        end,
                    };
                }
                self.bump();
                let t = if self.peek() == b'=' && self.peek2() != b'=' {
                    self.bump();
                    "/="
                } else {
                    "/"
                };
                kind = TokKind::Punct;
                text = t.to_string();
                self.set_prev(Prev::Punct, t);
            }
            b'"' | b'\'' => {
                let s = self.scan_string(c);
                self.set_prev(Prev::Str, "");
                kind = TokKind::Str;
                text = s;
            }
            b'0'..=b'9' => {
                let n = self.scan_number();
                self.set_prev(Prev::Num, "");
                kind = TokKind::Num;
                text = n;
            }
            b'.' if self.peek2().is_ascii_digit() => {
                let n = self.scan_number();
                self.set_prev(Prev::Num, "");
                kind = TokKind::Num;
                text = n;
            }
            _ if is_ident_start(c) || (c == b'\\' && self.peek2() == b'u') => {
                // 标识符可含 unicode 转义（\uXXXX / \u{...}——test262
                // 大量使用）：解码进名字（打印输出解码形态——语义恒等，
                // 幂等成立）。高字节按 UTF-8 解码字符判定（NBSP 等非
                // 标识字符终止——test262 whitespace 家族）。
                let mut word = String::new();
                while self.pos < self.src.len() {
                    let c = self.peek();
                    if c == b'\\' && self.peek2() == b'u' {
                        match decode_unicode_escape(self.src, self.pos + 1) {
                            Some((ch, len)) => {
                                // 白名单与高字节分支**严格一致**（含 ZWJ/
                                // ZWNJ 续接符——`‌` 曾因两分支白名单
                                // 不一致被本分支 break 且零消费——空 ident
                                // token 无限累积 → 100GB OOM，test262
                                // unicode-property-names 抓获）
                                if !(ch.is_alphanumeric()
                                    || ch == '_'
                                    || ch == '$'
                                    || ch == '\u{200c}'
                                    || ch == '\u{200d}')
                                {
                                    break;
                                }
                                word.push(ch);
                                self.pos += len + 1;
                            }
                            None => break,
                        }
                    } else if c < 0x80 && is_ident_part(c) {
                        word.push(c as char);
                        self.pos += 1;
                    } else if c >= 0x80 {
                        // 高字节字符：**宽容接受**为标识符（Unicode ID 表
                        // 无法零依赖内置——test262 的 ℘/℮ 等 Other_ID_Start
                        // 字符远多于误判代价；唯一终止条件是空白。
                        // 真正的 Unicode 标点误并入标识符最多损失一个
                        // 错误报告——原文保真不受影响）
                        let l = utf8_len(c);
                        let end = (self.pos + l).min(self.src.len());
                        let ch_str = String::from_utf8_lossy(&self.src[self.pos..end]).to_string();
                        let ch = ch_str.chars().next().unwrap_or('\u{fffd}');
                        if ch.is_whitespace() {
                            break;
                        }
                        word.push_str(&ch_str);
                        self.pos = end;
                    } else {
                        break;
                    }
                }
                let word = word;
                self.pos = self.pos;
                kind = TokKind::Ident;
                text = word.clone();
                self.set_prev(
                    if REGEX_LEAD.contains(&word.as_str()) {
                        Prev::Keyword
                    } else {
                        Prev::Ident
                    },
                    &word,
                );
            }
            _ => {
                let p = self.scan_punct();
                kind = TokKind::Punct;
                self.set_prev(
                    match p.as_str() {
                        ")" => Prev::CloseParen,
                        "]" => Prev::CloseBracket,
                        _ => Prev::Punct,
                    },
                    &p,
                );
                text = p;
            }
        }
        Token {
            kind,
            text,
            line,
            col,
            newline_before: newline,
            start: byte_start,
            end: self.pos,
        }
    }

    fn mk(
        &self,
        kind: TokKind,
        text: &str,
        line: usize,
        col: usize,
        newline: bool,
        start: usize,
    ) -> Token {
        Token {
            kind,
            text: text.to_string(),
            line,
            col,
            newline_before: newline,
            start,
            end: self.pos,
        }
    }

    fn set_prev(&mut self, p: Prev, text: &str) {
        self.prev = p;
        if !text.is_empty() || matches!(p, Prev::Keyword) {
            self.prev_text = text.to_string();
        }
    }

    /// `{` 的块式判定（正则启发式的 `}` 分支 + 对象字面量区分）。
    fn brace_block_like(&self) -> bool {
        match self.prev {
            Prev::CloseParen | Prev::CloseBrace | Prev::Start => true,
            Prev::Keyword => BLOCK_KW.contains(&self.prev_text.as_str()),
            Prev::Punct => matches!(
                self.prev_text.as_str(),
                ";" | "{" | "=>" | ":" | "@" | "#" | "case"
            ),
            _ => false,
        }
    }

    /// 块式栈（`{` 压入、`}` 弹出）。
    fn last_brace_block_push(&mut self, block_like: bool) {
        self.brace_stack.push(block_like);
    }
    fn pop_brace_kind(&mut self) -> bool {
        self.brace_stack.pop().unwrap_or(true)
    }

    /// 正则启发式。
    fn regex_allowed(&self) -> bool {
        match self.prev {
            Prev::Start | Prev::Keyword | Prev::Template => true,
            Prev::CloseBrace => self.last_brace_block,
            // 后缀 ++/-- 是表达式结尾 → 除法；`<` 后是 JSX 闭标签/
            // 泛型（不可能是除法左操作数）→ 拒绝正则；其余标点开
            // 表达式 → 正则
            Prev::Punct => !matches!(self.prev_text.as_str(), "++" | "--" | "<"),
            // `)` 后多数是表达式结尾（除 if/for 头——罕见正则位置，
            // 保守按除法；坏除法由 parser 容错恢复兜底）
            Prev::Ident | Prev::Num | Prev::Str | Prev::Regex | Prev::CloseParen
            | Prev::CloseBracket => false,
        }
    }

    /// 扫描 quasi：直到 `` ` ``（is_end=true）或 `${`（is_end=false）。
    fn scan_quasi(&mut self) -> (String, bool) {
        let mut q = String::new();
        loop {
            if self.at_end() {
                self.err("unterminated template literal");
                return (q, true);
            }
            let c = self.peek();
            match c {
                b'`' => {
                    self.bump();
                    return (q, true);
                }
                b'$' if self.peek2() == b'{' => {
                    self.pos += 2;
                    return (q, false);
                }
                b'\\' => {
                    self.bump();
                    if !self.at_end() {
                        let e = self.bump();
                        q.push('\\');
                        q.push(if e == b'\r' { '\r' } else { e as char });
                        // CR LF 保持原文（打印机保真）
                    } else {
                        q.push('\\');
                    }
                }
                b'\r' => {
                    self.bump();
                    if self.peek() == b'\n' {
                        self.bump();
                    }
                    q.push('\n');
                }
                _ => {
                    let ch_len = utf8_len(c);
                    let end = (self.pos + ch_len).min(self.src.len());
                    q.push_str(&String::from_utf8_lossy(&self.src[self.pos..end]));
                    for _ in self.pos..end {
                        self.bump();
                    }
                }
            }
        }
    }

    fn scan_string(&mut self, quote: u8) -> String {
        let start = self.pos;
        self.bump(); // 引号
        loop {
            if self.at_end() {
                self.err("unterminated string");
                break;
            }
            let c = self.peek();
            if c == b'\\' {
                self.bump();
                if !self.at_end() {
                    self.bump();
                }
                continue;
            }
            if c == quote {
                self.bump();
                break;
            }
            if c == b'\n' {
                self.err("unterminated string (newline)");
                break;
            }
            self.bump();
        }
        String::from_utf8_lossy(&self.src[start..self.pos]).to_string()
    }

    fn scan_number(&mut self) -> String {
        let start = self.pos;
        // 0x / 0o / 0b
        if self.peek() == b'0' && matches!(self.peek2(), b'x' | b'X' | b'o' | b'O' | b'b' | b'B') {
            self.pos += 2;
            while self.peek().is_ascii_alphanumeric() || self.peek() == b'_' {
                self.pos += 1;
            }
            return self.text_from(start);
        }
        // 十进制整数部分
        while self.peek().is_ascii_digit() || self.peek() == b'_' {
            self.pos += 1;
        }
        // BigInt（12n）——小数点前才可能是
        if self.peek() == b'n' {
            self.pos += 1;
            return self.text_from(start);
        }
        // 小数部分（1. / 1.5）
        if self.peek() == b'.' {
            self.pos += 1;
            while self.peek().is_ascii_digit() || self.peek() == b'_' {
                self.pos += 1;
            }
        }
        // 指数
        if (self.peek() == b'e' || self.peek() == b'E')
            && (self.peek2().is_ascii_digit()
                || ((self.peek2() == b'+' || self.peek2() == b'-') && self.peek3().is_ascii_digit()))
        {
            self.pos += 1;
            if self.peek() == b'+' || self.peek() == b'-' {
                self.pos += 1;
            }
            while self.peek().is_ascii_digit() || self.peek() == b'_' {
                self.pos += 1;
            }
        }
        self.text_from(start)
    }

    fn text_from(&self, start: usize) -> String {
        String::from_utf8_lossy(&self.src[start..self.pos]).to_string()
    }

    fn scan_regex(&mut self) -> String {
        let start = self.pos;
        self.bump(); // '/'
        let mut in_class = false;
        loop {
            if self.at_end() {
                self.err("unterminated regex");
                break;
            }
            let c = self.bump();
            match c {
                b'\\' => {
                    if !self.at_end() {
                        let e = self.bump();
                        if e == b'\n' {
                            self.err("regex escape newline");
                            break;
                        }
                    }
                }
                b'[' => in_class = true,
                b']' => in_class = false,
                b'/' if !in_class => break,
                b'\n' => {
                    self.err("unterminated regex (newline)");
                    break;
                }
                _ => {}
            }
        }
        // flags
        while is_ident_part(self.peek()) {
            self.pos += 1;
        }
        self.text_from(start)
    }

    /// 标点最长匹配（长在前）。
    fn scan_punct(&mut self) -> String {
        const PUNCTS: &[&str] = &[
            ">>>=", "...", "===", "!==", "**=", "<<=", ">>=", ">>>", "&&=", "||=", "??=", "=>",
            "==", "!=", "<=", ">=", "&&", "||", "??", "?.", "++", "--", "+=", "-=", "*=", "/=", "%=",
            "&=", "|=", "^=", "**", "<<", ">>", "{", "}", "(", ")", "[", "]", ";", ",", "<",
            ">", "+", "-", "*", "/", "%", "&", "|", "^", "!", "~", "?", ":", "=", ".", "@", "#",
        ];
        for p in PUNCTS {
            let b = p.as_bytes();
            if self.src.len() >= self.pos + b.len() && &self.src[self.pos..self.pos + b.len()] == b {
                // `?.` 后随数字是三元 `x ? .5 : …`——回退为 `?`
                if *p == "?." && self.peek3().is_ascii_digit() {
                    break;
                }
                self.pos += b.len();
                return p.to_string();
            }
        }
        // 未知标点：单字节
        let c = self.bump();
        (c as char).to_string()
    }
}

pub fn is_ident_start(c: u8) -> bool {
    c == b'_' || c == b'$' || c.is_ascii_alphabetic() || c >= 0x80
}

/// 解码标识符转义（`\uXXXX` / `\u{...}`）：返回解码字符与消耗长度。
/// 无效转义返回 None（调用方按原文兜底）。
fn decode_unicode_escape(src: &[u8], pos: usize) -> Option<(char, usize)> {
    // pos 指向 'u'
    if src.get(pos) != Some(&b'u') {
        return None;
    }
    if src.get(pos + 1) == Some(&b'{') {
        let mut end = pos + 2;
        while end < src.len() && src[end] != b'}' {
            end += 1;
        }
        if end >= src.len() {
            return None;
        }
        let hex = std::str::from_utf8(&src[pos + 2..end]).ok()?;
        let v = u32::from_str_radix(hex, 16).ok()?;
        let c = char::from_u32(v)?;
        Some((c, end - pos + 1))
    } else {
        if pos + 4 >= src.len() {
            return None;
        }
        let hex = std::str::from_utf8(&src[pos + 1..pos + 5]).ok()?;
        let v = u32::from_str_radix(hex, 16).ok()?;
        let c = char::from_u32(v)?;
        Some((c, 5))
    }
}

pub fn is_ident_part(c: u8) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

fn utf8_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else if b >> 3 == 0b11110 {
        4
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<(TokKind, String)> {
        let (toks, errs) = Lexer::new(src).tokenize();
        assert!(errs.is_empty(), "{errs:?}");
        toks.into_iter()
            .filter(|t| t.kind != TokKind::Eof)
            .map(|t| (t.kind.clone(), t.text.clone()))
            .collect()
    }

    #[test]
    fn punct_maximal_munch() {
        let k = kinds("a >>>= b ??= c => d ... e ?. f");
        assert!(k.contains(&(TokKind::Punct, ">>>=".into())));
        assert!(k.contains(&(TokKind::Punct, "??=".into())));
        assert!(k.contains(&(TokKind::Punct, "=>".into())));
        assert!(k.contains(&(TokKind::Punct, "...".into())));
        assert!(k.contains(&(TokKind::Punct, "?.".into())));
    }

    #[test]
    fn regex_vs_division() {
        // 值后是除法
        let k = kinds("a / b");
        assert!(k.contains(&(TokKind::Punct, "/".into())));
        // 关键字后是正则
        let k = kinds("return /re/.test(x)");
        assert!(k.contains(&(TokKind::Regex, "/re/".into())));
        // 块式 `}` 后是正则
        let k = kinds("function f() { x } /re/.test(y)");
        assert!(k.contains(&(TokKind::Regex, "/re/".into())));
        // 对象字面量 `}` 后是除法
        let k = kinds("var a = {} / 2");
        assert!(k.contains(&(TokKind::Punct, "/".into())));
        // 语义上 ({} / 2) 是除法；lexer 判定依赖块式追踪
    }

    #[test]
    fn template_nesting() {
        let k = kinds("`a${ `b${x}` }c`");
        // 三段 quasi + 嵌套模板的 quasi
        let quasis: Vec<_> = k
            .iter()
            .filter(|(kind, _)| matches!(kind, TokKind::TemplateQuasi { .. }))
            .map(|(_, t)| t.clone())
            .collect();
        assert_eq!(quasis, vec!["a", "b", "", "c"]);
    }

    #[test]
    fn numbers() {
        let k = kinds("0x1F 1_000 1.5e2 12n 1. .5");
        let nums: Vec<_> = k
            .iter()
            .filter(|(kind, _)| *kind == TokKind::Num)
            .map(|(_, t)| t.clone())
            .collect();
        assert_eq!(nums, vec!["0x1F", "1_000", "1.5e2", "12n", "1.", ".5"]);
    }

    #[test]
    fn newline_flag() {
        let (toks, _) = Lexer::new("a\nb").tokenize();
        let b = toks.iter().find(|t| t.text == "b").unwrap();
        assert!(b.newline_before);
        let a = toks.iter().find(|t| t.text == "a").unwrap();
        assert!(!a.newline_before);
    }
}
