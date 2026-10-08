//! # cure-java-parser
//!
//! 容错式 Java 源码解析器：源码文本 → [`cure_java_ast::CompilationUnit`] + arena 节点。
//!
//! **容错语义**（本 crate 的核心承诺）：
//! - 永不 panic、永不放弃：无论输入多坏都返回"尽力而为"的解析结果；
//! - 哪里坏跳哪里：出错的语句 → `Raw` 语句节点（原文保真），出错的方法 →
//!   `Member::Raw`，出错的顶层区域 → `unit.raws`；其余部分照常解析、照常可被简化；
//! - [`parse`] 同时返回所有 [`ParseError`]（1-based 行列）。
//!
//! 覆盖范围：package/import、类/接口/枚举（成员、字段、方法、构造器、初始化块、
//! 嵌套类型）、完整语句集（if/while/do/for/for-each/try/catch/finally/
//! try-with-resources/switch 经典+箭头/synchronized/labeled/assert）、
//! 完整表达式（赋值、三元、短路、instanceof、lambda、方法引用、数组创建、
//! 泛型钻石、匿名类体原文保真）。签名泛型/注解/throws 以原文保真。

pub mod lexer;

use cure_java_ast::*;

use lexer::{lex, Token, Tok};

/// 解析结果：arena + 编译单元 + 错误列表。
#[derive(Debug)]
pub struct ParseOutcome {
    pub ast: JavaAst,
    pub unit: CompilationUnit,
    pub errors: Vec<ParseError>,
}

/// JLS 3.3 预处理：解码合法的 `\uXXXX`（含 `\uu+` 多 u 形态；`\` 前有
/// 偶数个反斜杠才合法）。必须发生在词法**之前**——`'\u005c''` 只有先解码
/// 成 `'\''` 才能正确切词（javaparser EscapeSequences 抓获：逐 token 处理
/// 会把尾随 `'` 切进垃圾字符字面量 → 风暴）。解码语义与 javac 一致。
/// 字节级扫描（`\`/`u`/hex 均 ASCII，UTF-8 多字节 ≥0x80 永不误匹配）；
/// 无合法逃逸时零拷贝借用（绝大多数文件）——曾每文件 Vec<char>(4B/char)
/// + String 双分配，语料 5596 文件全量多付出 ~GB 级瞬时流量。
fn preprocess_unicode_escapes(src: &str) -> std::borrow::Cow<'_, str> {
    use std::borrow::Cow;
    let b = src.as_bytes();
    let mut i = 0usize;
    // 阶段 1：是否存在合法逃逸（快路径扫描，无分配）
    while i < b.len() {
        if b[i] != b'\\' {
            i += 1;
            continue;
        }
        let run_start = i;
        let mut j = i;
        while j < b.len() && b[j] == b'\\' {
            j += 1;
        }
        let run = j - run_start;
        let mut k = j;
        while k < b.len() && b[k] == b'u' {
            k += 1;
        }
        let u_count = k - j;
        let eligible = run % 2 == 1
            && u_count >= 1
            && k + 4 <= b.len()
            && b[k..k + 4].iter().all(|c| c.is_ascii_hexdigit());
        if eligible {
            return Cow::Owned(build_decoded(src, b, run_start));
        }
        if u_count > 0 && run % 2 == 1 {
            i = k;
        } else {
            i = j;
        }
    }
    Cow::Borrowed(src)
}

/// 从首个逃逸位置重建解码文本（逃逸均为 ASCII 边界，区间切片合法 UTF-8）。
/// 拷贝游标 copy：逃逸之间的普通文本靠 push_str(src[copy..逃逸起点]) 补
/// ——曾漏掉该区段（只有逃逸自身进 out → 尾部全丢，formfeed 回归测试
/// 抓获：`" \u000C"` 解码后连收尾引号都消失 → unterminated string）。
fn build_decoded(src: &str, b: &[u8], first: usize) -> String {
    let mut out = String::with_capacity(src.len());
    let mut copy = 0usize;
    let mut i = first;
    while i < b.len() {
        if b[i] != b'\\' {
            i += 1;
            continue;
        }
        let run_start = i;
        let mut j = i;
        while j < b.len() && b[j] == b'\\' {
            j += 1;
        }
        let run = j - run_start;
        let mut k = j;
        while k < b.len() && b[k] == b'u' {
            k += 1;
        }
        let u_count = k - j;
        let eligible = run % 2 == 1
            && u_count >= 1
            && k + 4 <= b.len()
            && b[k..k + 4].iter().all(|c| c.is_ascii_hexdigit());
        if eligible {
            let hex = &src[k..k + 4];
            let v = u32::from_str_radix(hex, 16).unwrap_or(0xfffd);
            // lone surrogate（U+D800–DFFF）：Rust char 不可表示。映射到
            // 15 号专用区哨兵（U+F0000 + 偏移，实践中永不冲突），打印机
            // 侧还原为 \uXXXX 转义——往返保真（此前 unwrap_or(FFFD)
            // 静默损坏，InputAvoidEscapedUnicodeCharacters 抓获：
            // 2048 个 surrogate 转义折成 FFFD）
            let decoded = if (0xd800..=0xdfff).contains(&v) {
                char::from_u32(0xf0000 + (v - 0xd800)).unwrap_or('\u{fffd}')
            } else {
                char::from_u32(v).unwrap_or('\u{fffd}')
            };
            out.push_str(&src[copy..run_start]); // 逃逸前的原文
            // 前 run-1 个 \ 原样保留（\ 对 = 转义反斜杠），\uXXXX → 解码字符
            for _ in 0..run - 1 {
                out.push('\\');
            }
            out.push(decoded);
            copy = k + 4;
            i = k + 4;
        } else {
            // 不合法（或非 \u 形态）：原样区段（由下一逃逸或尾部 push 补）
            i = if u_count > 0 && run % 2 == 1 { k } else { j };
        }
    }
    out.push_str(&src[copy..]);
    out
}

/// 解析 Java 源码（容错，永不失败）。
pub fn parse(src: &str) -> ParseOutcome {
    // JLS 3.3：\uXXXX 预解码（词法之前）——token 偏移与 text_of 均基于
    // 解码后的文本
    let decoded = preprocess_unicode_escapes(src);
    // 借用 decode 结果（无逃逸文件零拷贝；有逃逸借用局部 owned——
    // 生命周期包含于本函数）
    let decoded_ref: &str = match &decoded {
        std::borrow::Cow::Borrowed(b) => *b,
        std::borrow::Cow::Owned(o) => o.as_str(),
    };
    let (tokens, lex_errs) = lex(decoded_ref);
    let mut p = Parser {
        t: tokens,
        pos: 0,
        src: decoded_ref,
        errs: Vec::new(),
        ast: JavaAst::new(),
        in_case_label: false,
    };
    for e in lex_errs {
        p.errs.push(ParseError {
            line: e.line,
            col: e.col,
            message: format!("lex: {}", e.message),
        });
    }
    let unit = p.compilation_unit();
    ParseOutcome {
        ast: p.ast,
        unit,
        errors: p.errs,
    }
}

// ---------------------------------------------------------------------------
// 解析器
// ---------------------------------------------------------------------------

struct Parser<'src> {
    t: Vec<Token<'src>>,
    pos: usize,
    /// 借用源（无 \uXXXX 逃逸的文件零拷贝——曾 into_owned() 每文件
    /// 整源克隆，37 万文件 = 4.6GB memcpy；有逃逸时借用 decode 后的
    /// 局部 owned String，生命周期完全包含在 parse() 内）
    src: &'src str,
    errs: Vec<ParseError>,
    ast: JavaAst,
    /// case 标签解析中：`AOSP ->` 不是单参 lambda
    in_case_label: bool,
}

/// 解析深度上限（防御恶意/超长输入导致的失控）。
const STEP_GUARD: usize = 2_000_000;

impl<'src> Parser<'src> {
    // ---- 基础 ----

    fn tok(&self) -> &Token<'src> {
        &self.t[self.pos.min(self.t.len() - 1)]
    }
    fn peek(&self, off: usize) -> &Token<'src> {
        &self.t[(self.pos + off).min(self.t.len() - 1)]
    }
    fn at(&self, s: &str) -> bool {
        self.tok().text() == s
    }
    fn at_punct(&self, s: &str) -> bool {
        self.tok().is_punct(s)
    }
    /// final 与注解的任意交错前缀（`@A final @B C c`——JSR 308 + 模式修饰）。
    /// 注解/修饰不进节点：类型注解无运行时语义；声明位置另有各自建模。
    /// 同 skip_mods_annotations，但报告是否跳过了 `final`。
    fn skip_mods_annotations_final(&mut self) -> bool {
        let mut saw_final = false;
        loop {
            if self.at_kw("final") {
                saw_final = true;
                self.bump();
                continue;
            }
            if self.at_punct("@") {
                self.bump();
                self.bump();
                while self.at_punct(".")
                    && matches!(self.peek(1).tok, Tok::Ident(_))
                {
                    self.bump();
                    self.bump();
                }
                if self.at_punct("(") {
                    self.skip_balanced("(", ")");
                }
                continue;
            }
            break;
        }
        saw_final
    }

    fn skip_mods_annotations(&mut self) {
        loop {
            if self.at_kw("final") {
                self.bump();
                continue;
            }
            if self.at_punct("@") {
                self.bump();
                self.bump();
                // 限定名（@a.b.C）
                while self.at_punct(".")
                    && matches!(self.peek(1).tok, Tok::Ident(_))
                {
                    self.bump();
                    self.bump();
                }
                if self.at_punct("(") {
                    self.skip_balanced("(", ")");
                }
                continue;
            }
            break;
        }
    }

    /// case 标签的 when 守卫（case … when cond ->，Java 21）——消费并入
    /// 原文（标签区间由调用方闭合）。`when cond ->` 中 when 后是箭头时
    /// 不是守卫（是名为 when 的标签？保守不消费）。
    fn consume_when_guard(&mut self) {
        if self.at_kw("when") && !self.peek(1).is_punct("->") {
            self.bump();
            // 守卫表达式内 λ 合法（`case Integer i when list.stream()
            // .anyMatch(x -> x < i)`——ES/WG 复现：in_case_label=true
            // 使主表达式解析把 `x ->` 当 switch 箭头拒绝 → 解析失败风暴
            // → 恢复期重组毁文件）。临时解除标记。
            // 但守卫后的 `->`（case 体）不能再被当 λ 箭头误吃——
            // CP5 复现：`(o instanceof String) -> {}` 被解析为 λ。
            // 对策：λ 允许，但完成后若停在 `->` 之前的表达式边界，
            // 由收尾判定接管；被括号包裹的守卫走平衡跳过（原文保真，
            // 语义不触碰——raw 标签由上层 text_of 拼回）
            let save = self.pos;
            if self.at_punct("(") {
                // 括号化守卫：平衡扫描（含嵌套 λ 的箭头/花括号）——
                // 保守整段原文
                let _ = self.skip_balanced("(", ")");
            } else {
                // 守卫表达式：**前哨扫描**到本 case 的收尾 `->` / `:`
                //（不在嵌套 ()/<>/{}/[] 内）——不区分裸标识符/二元/λ
                //（DeconstructionDesugaring 的 `((int) o1) == 0 && …` /
                // GuardsErrors 的 `i == check` / WG 的 λ 形态全覆盖；
                // 原文保真——语义不触碰，标签 raw 由上层 text_of 拼回）
                let mut depth = 0i32;
                let mut guard = 0usize;
                while !self.at_eof() && guard < 100_000 {
                    guard += 1;
                    let t = self.tok().clone();
                    match &t.tok {
                        Tok::Punct(p) => match *p {
                            // 只追踪 ()/{}——< > 在守卫里几乎总是比较
                            // 运算符（`x < i` 曾虚假加深度使收尾箭头
                            // 永不停——WG 复现）
                            "(" | "{" | "[" => depth += 1,
                            ")" | "}" | "]" => {
                                if depth == 0 {
                                    break; // 守卫意外闭合——上层回退
                                }
                                depth -= 1;
                            }
                            "->" | ":" if depth == 0 => break,
                            _ => {}
                        },
                        Tok::Ident(k) if depth == 0 && (*k == "case" || *k == "default") => {
                            break; // 下一标签——守卫已空/失败
                        }
                        _ => {}
                    }
                    self.bump();
                }
            }
            let _ = save;
        }
    }

    /// 值类（JDK 28 预览 JEP draft：`value class` / 内部亦可能 `value` + 其他
    /// 声明）。两 token 前瞻判定——不进 is_modifier_kw（value 是常见标识符，
    /// 会误吃变量名/字段名）
    fn at_value_decl(&self) -> bool {
        self.at_kw("value") && matches!(&self.peek(1).tok, Tok::Ident(c) if *c == "class")
    }

    fn at_kw(&self, s: &str) -> bool {
        matches!(self.tok().tok, Tok::Ident(i) if i == s)
    }
    /// `@interface` 注解类型声明（`@` 是独立 punct，词法层不成单 token）。
    fn at_annotation_decl(&self) -> bool {
        self.at_punct("@") && matches!(&self.peek(1).tok, Tok::Ident(i) if *i == "interface")
    }
    fn bump(&mut self) -> Token<'_> {
        let t = self.t[self.pos.min(self.t.len() - 1)].clone();
        if self.pos < self.t.len() - 1 {
            self.pos += 1;
        }
        t
    }
    fn eat(&mut self, s: &str) -> bool {
        if self.at(s) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn at_eof(&self) -> bool {
        matches!(self.tok().tok, Tok::Eof)
    }
    fn err_at(&mut self, msg: &str) {
        let (line, col) = (self.tok().line, self.tok().col);
        self.errs.push(ParseError {
            line,
            col,
            message: msg.into(),
        });
    }
    fn expect(&mut self, s: &str) -> bool {
        if self.eat(s) {
            true
        } else {
            self.err_at(&format!("expected `{s}`, found `{}`", self.tok().text()));
            false
        }
    }
    fn text_of(&self, a: usize, b: usize) -> String {
        let (a, b) = (a.min(self.src.len()), b.min(self.src.len()));
        if a >= b {
            String::new()
        } else {
            self.src[a..b].to_string()
        }
    }
    /// 当前 token 起始的字节偏移。
    fn cur_start(&self) -> usize {
        self.tok().start
    }

    // ---- 恢复 ----

    /// 从 `byte_start` 起做语句级恢复，返回覆盖整条残缺语句的原文。
    fn raw_from(&mut self, byte_start: usize) -> JavaId {
        let _ = self.sync_stmt();
        let end = self.t[self.pos.min(self.t.len() - 1)].start;
        let text = self.text_of(byte_start, end).trim().to_string();
        if text.is_empty() {
            self.ast.empty()
        } else {
            self.ast.raw(&text)
        }
    }

    /// 语句级恢复：跳到 `;`（消费）或语句起始关键字 / `}` / EOF（不消费）。
    /// 返回被跳过的原文。
    fn sync_stmt(&mut self) -> String {
        let start = self.cur_start();
        let mut depth = 0i32;
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > STEP_GUARD || self.at_eof() {
                break;
            }
            let t = self.tok().clone();
            if depth == 0 {
                if t.is_punct(";") {
                    self.bump();
                    break;
                }
                if t.is_punct("}") || t.is_punct("{") {
                    break;
                }
                if matches!(&t.tok, Tok::Ident(i) if matches!(
                    *i,
                    "if" | "for" | "while" | "do" | "try" | "return" | "throw" | "break"
                        | "continue" | "switch" | "synchronized" | "assert" | "final" | "class"
                ) || is_primitive_kw(i))
                {
                    break;
                }
            }
            // 闭括号在 depth==0：停在不消费位置（防止把后续代码吞进 Raw）
            if depth == 0
                && matches!(&t.tok, Tok::Punct("}") | Tok::Punct(")") | Tok::Punct("]"))
            {
                break;
            }
            match &t.tok {
                Tok::Punct("{") | Tok::Punct("(") | Tok::Punct("[") => depth += 1,
                Tok::Punct("}") | Tok::Punct(")") | Tok::Punct("]") => depth -= 1,
                _ => {}
            }
            self.bump();
        }
        let text = self.text_of(start, self.t[self.pos.min(self.t.len() - 1)].start);
        text.trim().to_string()
    }

    /// 成员级恢复：跳过平衡区域直到 `;`（消费）或回到成员边界 `}`（不消费）。
    /// 吞到当前 switch 块的收尾 `}`（不消耗 `}` 本身；由调用方的
    /// 外层循环统一收）。风暴兜底用。
    fn sync_to_block_end(&mut self) -> String {
        let start = self.cur_start();
        let mut depth = 0i32;
        let mut guard = 0usize;
        while !self.at_eof() && guard < STEP_GUARD {
            guard += 1;
            let t = self.tok().clone();
            match &t.tok {
                Tok::Punct("{") => depth += 1,
                Tok::Punct("}") => {
                    if depth <= 0 {
                        break;
                    }
                    depth -= 1;
                }
                _ => {}
            }
            self.bump();
        }
        self.text_of(start, self.t[self.pos.saturating_sub(1)].end)
    }

    /// 局部类型声明探测（不动 self.pos）：跳过 abstract/final/注解后
    /// 若是 class/interface/enum/record 的**声明形态**返回关键字的 pos。
    /// record 的上下文关键字判定（后随 Ident + (/</{）与主路径一致。
    fn find_local_type_decl(&self) -> Option<usize> {
        let mut p = self.pos;
        // 跳过修饰符
        loop {
            match &self.t.get(p).map(|t| &t.tok) {
                Some(Tok::Ident(i)) if *i == "abstract" || *i == "final" => {
                    p += 1;
                }
                Some(Tok::Punct(q)) if *q == "@" => {
                    // @Anno / @Anno(...) / @a.b.C(...)
                    p += 1;
                    if let Some(Tok::Ident(_)) = &self.t.get(p).map(|t| &t.tok) {
                        p += 1;
                    }
                    while let Some(Tok::Punct(q)) = &self.t.get(p).map(|t| &t.tok) {
                        if *q != "." {
                            break;
                        }
                        p += 1;
                        if let Some(Tok::Ident(_)) = &self.t.get(p).map(|t| &t.tok) {
                            p += 1;
                        } else {
                            return None;
                        }
                    }
                    if let Some(Tok::Punct("(")) = &self.t.get(p).map(|t| &t.tok) {
                        // 平衡扫描
                        let mut depth = 0i32;
                        loop {
                            match &self.t.get(p).map(|t| &t.tok) {
                                Some(Tok::Punct(q)) if *q == "(" => depth += 1,
                                Some(Tok::Punct(q)) if *q == ")" => {
                                    depth -= 1;
                                    if depth == 0 {
                                        p += 1;
                                        break;
                                    }
                                }
                                Some(Tok::Eof) | None => return None,
                                _ => {}
                            }
                            p += 1;
                        }
                    }
                }
                _ => break,
            }
        }
        // 类型关键字判定
        match &self.t.get(p).map(|t| &t.tok) {
            Some(Tok::Ident(k))
                if *k == "class" || *k == "interface" || *k == "enum" =>
            {
                // 声明形态：后随 Ident（`enum e = ...` 的变量用法后随 = ;  排除）
                if let Some(Tok::Ident(_)) = &self.t.get(p + 1).map(|t| &t.tok) {
                    if !matches!(
                        &self.t.get(p + 2).map(|t| &t.tok),
                        Some(Tok::Punct(q)) if *q == "=" || *q == ";" || *q == ","
                    ) {
                        Some(p)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Some(Tok::Ident(k)) if *k == "record" => {
                // record 是上下文关键字：Ident 后随 ( / < / {
                if let Some(Tok::Ident(_)) = &self.t.get(p + 1).map(|t| &t.tok) {
                    if matches!(
                        &self.t.get(p + 2).map(|t| &t.tok),
                        Some(Tok::Punct(q)) if *q == "(" || *q == "<" || *q == "{"
                    ) {
                        Some(p)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn sync_member(&mut self) -> String {
        let start = self.cur_start();
        let mut depth = 0i32;
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > STEP_GUARD || self.at_eof() {
                break;
            }
            let t = self.tok().clone();
            if depth == 0 && t.is_punct(";") {
                self.bump();
                break;
            }
            if depth <= 0 && t.is_punct("}") {
                break;
            }
            match &t.tok {
                Tok::Punct("{") => depth += 1,
                Tok::Punct("}") => depth -= 1,
                _ => {}
            }
            self.bump();
        }
        let text = self.text_of(start, self.t[self.pos.min(self.t.len() - 1)].start);
        text.trim().to_string()
    }

    /// 跳过一个平衡的 `{...}`（要求当前指向 `{`），返回原文。
    fn skip_balanced_braces(&mut self) -> Option<String> {
        if !self.at_punct("{") {
            return None;
        }
        let start = self.cur_start();
        let mut depth = 0i32;
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > STEP_GUARD || self.at_eof() {
                return None;
            }
            match &self.tok().tok {
                Tok::Punct("{") => depth += 1,
                Tok::Punct("}") => {
                    depth -= 1;
                    if depth == 0 {
                        self.bump();
                        return Some(self.text_of(start, self.t[self.pos - 1].end));
                    }
                }
                _ => {}
            }
            self.bump();
        }
    }

    /// 跳过一个平衡的括号组（要求当前指向 `(` 或 `<` 等开括号），返回原文。
    fn skip_balanced(&mut self, open: &str, close: &str) -> Option<String> {
        if !self.at_punct(open) {
            return None;
        }
        let start = self.cur_start();
        let mut depth = 0i32;
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > STEP_GUARD || self.at_eof() {
                return None;
            }
            if self.at_punct(open) {
                depth += 1;
            } else {
                // close 方的分裂计数：泛型嵌套的收尾被词法器并成单 token
                // （Comparable<T>> 的 ">>"）——按 2/3 个 close 计，否则永不
                // 配平 → 静默吞到文件尾（对抗波 2 抓获：泛型方法整体消失）。
                // type_args_raw 同此语义。
                let n = if self.at_punct(close) {
                    1
                } else if close == ">" {
                    if self.at_punct(">>") {
                        2
                    } else if self.at_punct(">>>") {
                        3
                    } else {
                        0
                    }
                } else {
                    0
                };
                if n > 0 {
                    depth -= n;
                    if depth <= 0 {
                        self.bump();
                        return Some(self.text_of(start, self.t[self.pos - 1].end));
                    }
                }
            }
            self.bump();
        }
    }

    // ---- 编译单元 ----

    fn compilation_unit(&mut self) -> CompilationUnit {
        let mut unit = CompilationUnit::default();
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > STEP_GUARD || self.at_eof() {
                break;
            }
            if self.at_kw("package") {
                // 容错：package 语句缺分号（Bar.java 形态）——扫到分号或
                // 行尾即止（token 有 line 信息；不设界会吞掉整个 class 体
                // → 4M 错误风暴）。**点悬挂续行**：上一 token 是 `.` 时
                // 标识符可在新行（多行 package 是合法 Java——差分审查
                // 抓获：checkstyle NoWhitespaceAfter 的注释穿插多行
                // package 曾被从 `.` 后截断，`tools.` 变孤儿语句）。
                let pkg_line = self.tok().line;
                self.bump(); // 跳过 package 关键字本身
                let mut parts: String = String::new();
                let mut prev_dot = false;
                while !self.at_eof() && !self.at_punct(";") {
                    if self.tok().line != pkg_line && !prev_dot && !self.at_punct(".") {
                        break; // 换行且非点连接
                    }
                    // 包名按 token 归一化（标识符 + 点拼接）：原文里的
                    // 注释/空白/换行剥除——带尾注释的 raw 文本回打会把
                    // `;` 吞进注释（`package a.b // c;` 非法）
                    match &self.tok().tok {
                        Tok::Ident(name) => parts.push_str(name),
                        Tok::Punct(p) if *p == "." => parts.push('.'),
                        _ => {}
                    }
                    prev_dot = self.at_punct(".");
                    self.bump();
                }
                let pkg = parts;
                unit.package = Some(pkg);
                // 换行退出后仍可能跟悬空分号
                self.eat(";");
                continue;
            }
            if self.at_kw("import") {
                let start = self.cur_start();
                while !self.at_eof() && !self.at_punct(";") {
                    self.bump();
                }
                let after = self.text_of(start, self.tok().start);
                let imp = after.trim().trim_start_matches("import").trim().to_string();
                unit.imports.push(imp);
                self.eat(";");
                continue;
            }
            if self.at_punct(";") {
                self.bump();
                continue;
            }
            // 类型声明（带注解/修饰符）
            let mods_start = self.cur_start();
            let mods = self.modifiers();
            // package-info：注解（如 @NullMarked）后跟 package——注解
            // 吞掉后落回循环顶由 package 分支处理，注解原文挂到
            // unit.package_annotations（149 文件曾静默丢弃）
            if self.at_kw("package") {
                if !mods.is_empty() {
                    let ann = mods.trim().to_string();
                    unit.package_annotations = ann;
                }
                continue;
            }
            // 值类（JDK 28 预览）：public final value class X —— value 并入
            // mods 文本，class 照常解析
            if self.at_value_decl() {
                self.bump();
            }
            // module-info（Java 9）：[open] module name { requires/exports/
            // opens/provides/uses…; }——无专用节点，整文件 RAW 保真
            //（jdk-sources 每模块一个 module-info.java，87 失败中占 70）
            if self.at_kw("module")
                || (self.at_kw("open") && matches!(&self.peek(1).tok, Tok::Ident(m) if *m == "module"))
            {
                let text = self.text_of(mods_start, self.src.len());
                unit.raws.push(text.trim().to_string());
                return unit;
            }
            if self.at_kw("class")
                || self.at_kw("interface")
                || self.at_kw("enum")
                || self.at_kw("record")
                || self.at_annotation_decl()
            {
                match self.type_decl_body(&mods, mods_start) {
                    Some(t) => unit.types.push(t),
                    None => {
                        let pos_before = self.pos;
                        let text = self.sync_member();
                        if !text.is_empty() {
                            unit.raws.push(text);
                        }
                        // 停滞守卫（A8mem：sync_member 在 depth-0 `}` 不消费
                        // 即 break——本循环原位自旋 2M 轮/147MB）
                        if self.pos == pos_before && !self.at_eof() {
                            let ts = self.cur_start();
                            let te = self.tok().end;
                            let t = self.text_of(ts, te).trim().to_string();
                            if !t.is_empty() {
                                unit.raws.push(t);
                            }
                            self.bump();
                        }
                    }
                }
                continue;
            }
            // 顶层无法识别 → 原文保真
            self.err_at("expected type declaration");
            let pos_before = self.pos;
            let text = self.sync_member();
            if !text.is_empty() {
                unit.raws.push(text);
            }
            // 停滞守卫（同上——单 token raw 保真 + 强制推进）
            if self.pos == pos_before && !self.at_eof() {
                let ts = self.cur_start();
                let te = self.tok().end;
                let t = self.text_of(ts, te).trim().to_string();
                if !t.is_empty() {
                    unit.raws.push(t);
                }
                self.bump();
            }
        }
        unit
    }

    /// 修饰符与注解原文（不含尾随空格）。
    fn modifiers(&mut self) -> String {
        let start = self.cur_start();
        let mut last_end = start;
        let mut guard = 0usize;
        'mods: loop {
            guard += 1;
            if guard > 10_000 {
                break;
            }
            if self.at_annotation_decl() {
                // `@interface` 是注解类型声明关键字，不是注解——留给 type_decl_body
                break 'mods;
            }
            // non-sealed：三 token 序列合并（对抗波 3：单独消费 non 会留下
            // `-sealed` 残体产出非法输出）
            if matches!(&self.tok().tok, Tok::Ident(n) if *n == "non")
                && matches!(&self.peek(1).tok, Tok::Punct("-"))
                && matches!(&self.peek(2).tok, Tok::Ident(s) if *s == "sealed")
            {
                self.bump();
                self.bump();
                self.bump();
                last_end = self.t[self.pos - 1].end;
                continue;
            }
            if self.at_punct("@") {
                self.bump();
                self.bump(); // 注解名
                // 限定名注解（@jdk.internal.ValueBased——JDK 现代源码
                // 2119 文件主簇：类级注解大量限定名）
                while self.at_punct(".")
                    && matches!(self.peek(1).tok, Tok::Ident(_))
                {
                    self.bump();
                    self.bump();
                }
                // 注解参数（可能多层嵌套）
                if self.at_punct("(") {
                    self.skip_balanced("(", ")");
                }
                last_end = self.t[self.pos - 1].end;
                continue;
            }
            if matches!(&self.tok().tok, Tok::Ident(i) if is_modifier_kw(i)) {
                self.bump();
                last_end = self.t[self.pos - 1].end;
                continue;
            }
            break;
        }
        self.text_of(start, last_end)
    }

    // ---- 类型声明 ----

    fn type_decl_body(&mut self, mods: &str, mods_start: usize) -> Option<TypeDecl> {
        if self.at_value_decl() {
            self.bump();
        }
        let kind = if self.at_kw("class") {
            TypeKind::Class
        } else if self.at_kw("interface") {
            TypeKind::Interface
        } else if self.at_kw("enum") {
            TypeKind::Enum
        } else if self.at_kw("record") {
            TypeKind::Record
        } else if self.at_annotation_decl() {
            TypeKind::Annotation
        } else {
            return None;
        };
        if kind == TypeKind::Annotation {
            self.bump(); // @
        }
        self.bump(); // kw
        let mut name = match &self.tok().tok {
            Tok::Ident(i) => {
                let n = i.to_string();
                self.bump();
                n
            }
            _ => {
                self.err_at("expected type name");
                return None;
            }
        };
        // DAD/androguard 伪影：class LDemo; { —— 名字是 Dalvik 描述符
        // （合法 Java 不会有 `Name; {`），剥前导 L 与悬挂分号
        if self.at_punct(";") && self.peek(1).is_punct("{") {
            self.bump();
            if let Some(stripped) = name.strip_prefix('L') {
                if !stripped.is_empty() {
                    name = stripped.to_string();
                }
            }
        }
        // 泛型参数原文
        let mut ty_params = String::new();
        if self.at_punct("<") {
            ty_params = self.skip_balanced("<", ">").unwrap_or_default();
        }
        // record 头
        let mut header = String::new();
        if kind == TypeKind::Record && self.at_punct("(") {
            header = self.skip_balanced("(", ")").unwrap_or_default();
        }
        let mut extends = Vec::new();
        let mut implements = Vec::new();
        let mut permits: Vec<String> = Vec::new();
        loop {
            if self.at_kw("extends") {
                self.bump();
                extends = self.type_list();
                continue;
            }
            if self.at_kw("implements") {
                self.bump();
                implements = self.type_list();
                continue;
            }
            if self.at_kw("permits") {
                self.bump();
                permits = self.type_list();
                continue;
            }
            break;
        }
        if !self.expect("{") {
            // 允许 `;`（如注释掉的声明体）
            self.eat(";");
            return Some(TypeDecl {
                kind,
                mods: mods.to_string(),
                name,
                ty_params,
                header,
                permits: permits.clone(),
                extends,
                implements,
                enum_constants: Vec::new(),
                members: Vec::new(),
            });
        }
        let mut enum_constants = Vec::new();
        let mut members = Vec::new();
        if kind == TypeKind::Enum {
            // 枚举常量（原文保真；常量可带前注解 enum E{m, @Deprecated f;}
            // ——JavaConcepts 抓获：曾断在 @ 上 → 常量流落进成员解析风暴）
            while !self.at_eof() && !self.at_punct(";") && !self.at_punct("}") {
                let cstart = self.cur_start();
                while self.at_punct("@") {
                    self.bump();
                    self.bump();
                    if self.at_punct("(") {
                        self.skip_balanced("(", ")");
                    }
                }
                if !matches!(self.tok().tok, Tok::Ident(_)) {
                    break;
                }
                self.bump();
                if self.at_punct("(") {
                    self.skip_balanced("(", ")");
                }
                if self.at_punct("{") {
                    self.skip_balanced_braces();
                }
                let text = self
                    .text_of(cstart, self.t[self.pos.saturating_sub(1)].end)
                    .trim()
                    .to_string();
                enum_constants.push(text);
                if !self.eat(",") {
                    break;
                }
            }
            self.eat(";");
        }
        let _ = mods_start;
        // 成员循环
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > STEP_GUARD || self.at_eof() {
                break;
            }
            if self.at_punct("}") {
                self.bump();
                break;
            }
            // @interface 成员：`Type name() [default expr];` 无方法体 + 默认值
            // 形态无专用节点——整段 RAW 保真（可编译、逐字往返；对抗波 3 抓获
            // "expected method body" 错误流）
            if kind == TypeKind::Annotation {
                let mstart0 = self.cur_start();
                let text = self.sync_member();
                if text.is_empty() {
                    break;
                }
                members.push(Member::Raw(text.trim().to_string()));
                let _ = mstart0;
                continue;
            }
            if self.at_punct(";") {
                self.bump();
                continue;
            }
            let mstart = self.cur_start();
            let mmods = self.modifiers();
            // 嵌套值类（value class——JDK 28 预览）
            if self.at_value_decl() {
                self.bump();
            }
            // 嵌套类型
            if self.at_kw("class")
                || self.at_kw("interface")
                || self.at_kw("enum")
                || self.at_kw("record")
                || self.at_annotation_decl()
            {
                if let Some(t) = self.type_decl_body(&mmods, mstart) {
                    members.push(Member::Type(Box::new(t)));
                    continue;
                }
                let text = self.sync_member();
                members.push(Member::Raw(text));
                continue;
            }
            // 初始化块
            if self.at_punct("{") {
                let is_static = mmods.contains("static");
                let body = self.parse_block_raw();
                members.push(Member::Initializer { is_static, body });
                continue;
            }
            match self.member_rest(&mmods, &name) {
                Some(m) => members.push(m),
                None => {
                    // 前缀必须在 sync_member() **之前**取——它消费失败点之后
                    // 的全部残余，之后再取 cur_start 会抓成整段（调试锁定）。
                    // 前缀 = 成员起点到失败点（已消费的 mods/类型名）——
                    // 曾用 text_of(mstart, mstart) 恒空串把 `public PmdTest`
                    // 静默丢掉（ParserCornerCases18 抓获：「保留原文」承诺
                    // 被打破，输出 () { … }）
                    let prefix = self.text_of(mstart, self.cur_start());
                    let text = self.sync_member();
                    members.push(Member::Raw(prefix + &text));
                }
            }
        }
        Some(TypeDecl {
            kind,
            mods: mods.to_string(),
            name,
            ty_params,
            header,
            permits,
            extends,
            implements,
            enum_constants,
            members,
        })
    }

    /// `extends A, B` 风格的类型列表（原文逐项）。
    fn type_list(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > 10_000 {
                break;
            }
            let start = self.cur_start();
            if self.parse_type().is_none() {
                break;
            }
            let text = self.text_of(start, self.t[self.pos - 1].end);
            out.push(text.trim().to_string());
            if !self.eat(",") {
                break;
            }
        }
        out
    }

    /// 修饰符/注解之后的成员主体：字段 / 方法 / 构造器。
    fn member_rest(&mut self, mods: &str, class_name: &str) -> Option<Member> {
        // 泛型参数（方法/构造器都可能带）
        let mut ty_params = String::new();
        if self.at_punct("<") {
            ty_params = self.skip_balanced("<", ">")?;
        }
        // 构造器：Name( … 或 record 紧凑构造器 Name {
        if let Tok::Ident(n) = &self.tok().tok {
            if *n == class_name && (self.peek(1).is_punct("(") || self.peek(1).is_punct("{")) {
                let name = n.to_string();
                self.bump();
                let compact = self.at_punct("{");
                let params = if compact { Vec::new() } else { self.param_list() };
                let throws = self.throws_clause();
                let body = self.member_body();
                return Some(Member::Constructor {
                    mods: mods.to_string(),
                    ty_params,
                    name,
                    params,
                    throws,
                    body,
                    compact,
                });
            }
        }

        // 字段或方法：Type name
        let ty = self.parse_type()?;
        let mut name = match &self.tok().tok {
            Tok::Ident(i) => {
                let n = i.to_string();
                self.bump();
                n
            }
            _ => {
                self.err_at("expected member name");
                return None;
            }
        };
        // C 风格维度 int a[]：**不并入 ty**——ty 保持基类型，首个声明符
        // 的维度记入 first_extra 由声明符自带（曾 wrap 进 ty 又打印各声明符
        // 原始后缀 → `int f[], g[][]` 打成 `int[] f, g[][]` = g 三维，
        // Adv6 差分抓获：javac 读作 int[][][]）
        let first_extra = {
            let mut extra_dims = 0u16;
            loop {
                self.skip_mods_annotations();
                if self.at_punct("[") && self.peek(1).is_punct("]") {
                    self.bump();
                    self.bump();
                    extra_dims += 1;
                } else {
                    break;
                }
            }
            extra_dims
        };
        if self.at_punct("(") {
            // 方法
            let params = self.param_list();
            // C 风格数组返回后缀 int doSomething()[]（JavaConcepts 抓获：
            // 曾直接进 body 期待 → "expected method body"）
            let mut ret_dims = 0u16;
            while self.at_punct("[") && self.peek(1).is_punct("]") {
                self.bump();
                self.bump();
                ret_dims += 1;
            }
            let ty = wrap_dims(ty, ret_dims as u32);
            let throws = self.throws_clause();
            let body = self.member_body();
            return Some(Member::Method {
                mods: mods.to_string(),
                ty_params,
                ret: ty,
                name,
                params,
                throws,
                body,
            });
        }
        // 字段（多声明符）
        let mut declarators = Vec::new();
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > 10_000 {
                break;
            }
            let mut d_dims = if declarators.is_empty() { first_extra } else { 0 };
            loop {
                // 维度间注解：String [] @B [] x（openjdk LocalVariables）
                self.skip_mods_annotations();
                if self.at_punct("[") && self.peek(1).is_punct("]") {
                    self.bump();
                    self.bump();
                    d_dims += 1;
                } else {
                    break;
                }
            }
            let init = if self.eat("=") {
                match self.parse_expr(PREC_ASSIGN) {
                    Some(e) => Some(e),
                    None => {
                        self.err_at("bad field initializer");
                        None
                    }
                }
            } else {
                None
            };
            let full_ty = wrap_dims(ty.clone(), d_dims as u32);
            let init = self.wrap_decl_init_array(init, &full_ty);
            declarators.push(Declarator {
                name: name.clone(),
                extra_dims: d_dims,
                init,
            });
            if !self.eat(",") {
                break;
            }
            // 后续声明符：跳过注解（@Deprecated f——JavaConcepts 抓获），
            // 取**新名字**（曾丢弃名字 → `int a, b;` 打成 `int a, a;`——
            // 语义破坏，往返自检抓不到，本轮探针抓获）
            while self.at_punct("@") {
                self.bump();
                self.bump();
                if self.at_punct("(") {
                    self.skip_balanced("(", ")");
                }
            }
            match &self.tok().tok {
                Tok::Ident(n) => {
                    name = n.to_string();
                    self.bump();
                }
                _ => break,
            }
            continue;
        }
        self.expect(";");
        Some(Member::Field {
            mods: mods.to_string(),
            ty,
            declarators,
        })
    }

    fn member_body(&mut self) -> Option<JavaId> {
        if self.eat(";") {
            return None;
        }
        if self.at_punct("{") {
            return Some(self.parse_block_raw());
        }
        self.err_at("expected method body");
        let text = self.sync_member();
        Some(self.ast.raw(&text))
    }

    fn param_list(&mut self) -> Vec<Param> {
        let mut out = Vec::new();
        if !self.expect("(") {
            return out;
        }
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > 10_000 || self.at_eof() {
                break;
            }
            if self.at_punct(")") {
                self.bump();
                break;
            }
            let start = self.cur_start();
            let mods = self.modifiers();
            if mods.is_empty() {
                let _ = start;
            }
            let ty = match self.parse_type() {
                Some(t) => t,
                None => {
                    self.err_at("bad parameter type");
                    self.sync_param();
                    continue;
                }
            };
            // varargs：`T... name`（词法器产出单 token "..."）
            let varargs = if self.at_punct("...") {
                self.bump();
                true
            } else {
                false
            };
            let name = match &self.tok().tok {
                Tok::Ident(i) => {
                    let n = i.to_string();
                    self.bump();
                    n
                }
                _ => {
                    self.err_at("bad parameter name");
                    self.sync_param();
                    continue;
                }
            };
            // 参数名后的额外维度 int a[]
            let mut dims = 0u32;
            while self.at_punct("[") && self.peek(1).is_punct("]") {
                self.bump();
                self.bump();
                dims += 1;
            }
            out.push(Param {
                mods,
                ty: wrap_dims(ty, dims),
                varargs,
                name,
            });
            if !self.eat(",") {
                self.expect(")");
                break;
            }
        }
        out
    }

    fn sync_param(&mut self) {
        // 跳到 `,` 或 `)`
        let mut depth = 0i32;
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > STEP_GUARD || self.at_eof() {
                break;
            }
            if depth == 0 && self.at_punct(",") {
                self.bump();
                break;
            }
            if depth == 0 && self.at_punct(")") {
                self.bump();
                break;
            }
            match &self.tok().tok {
                Tok::Punct("(") | Tok::Punct("[") | Tok::Punct("{") | Tok::Punct("<") => {
                    depth += 1
                }
                Tok::Punct(")") | Tok::Punct("]") | Tok::Punct("}") | Tok::Punct(">") => {
                    depth -= 1
                }
                _ => {}
            }
            self.bump();
        }
    }

    fn throws_clause(&mut self) -> Vec<String> {
        if !self.at_kw("throws") {
            return Vec::new();
        }
        self.bump();
        self.type_list()
    }

    // ---- 类型 ----

    /// 解析类型（含数组后缀）；失败返回 None（调用方自行保存 pos 回滚）。
    fn parse_type(&mut self) -> Option<JType> {
        // 类型注解前缀（JSR 308：instanceof/泛型等类型位置的 @Anno(…)）——
        // **原文保留**并入 Ref 名（`<G> @A G m()` 的返回位置/instanceof 的
        // 纯 TYPE_USE 注解曾丢弃——TU 电池抓获）；基类型为原语时仍跳过
        //（原语类型注解无处安放——罕见形态，维持跳过）。spoon Pozole 的
        // instanceof 前注解曾使 parse_type 直接失败 → 表达式风暴（此处
        // 兼容历史行为：无注解路径完全不变）
        let mut prefix = String::new();
        while self.at_punct("@") {
            let s = self.cur_start();
            self.bump(); // @
            self.bump(); // 注解名
            if self.at_punct("(") {
                self.skip_balanced("(", ")");
            }
            prefix.push_str(self.text_of(s, self.cur_start()).trim());
            prefix.push(' ');
        }
        let base = self.parse_type_base()?;
        let mut ty = match base {
            JType::Ref(name) if !prefix.is_empty() => JType::Ref(format!("{prefix}{name}")),
            other => other,
        };
        loop {
            // 维度间注解：String [] @B [] x（openjdk LocalVariables——
            // 注解位于 [] 对之间，丢弃维度注解并入数组类型）
            self.skip_mods_annotations();
            if self.at_punct("[") && self.peek(1).is_punct("]") {
                self.bump();
                self.bump();
                ty = JType::Array(Box::new(ty));
            } else {
                break;
            }
        }
        Some(ty)
    }

    /// 解析"基类型"（不含 `[]` 后缀）——`new int[3]` 的 `[]` 属于数组创建。
    fn parse_type_base(&mut self) -> Option<JType> {
        let base = match &self.tok().tok {
            Tok::Ident(i) if is_primitive_kw(i) => {
                let t = match *i {
                    "byte" => JType::Byte,
                    "short" => JType::Short,
                    "int" => JType::Int,
                    "long" => JType::Long,
                    "char" => JType::Char,
                    "float" => JType::Float,
                    "double" => JType::Double,
                    "boolean" => JType::Bool,
                    _ => JType::Void,
                };
                self.bump();
                t
            }
            Tok::Ident(i) if !is_type_reserved(i) => {
                let mut name = i.to_string();
                self.bump();
                // 点分名 + 段级泛型（GO<String>.RU<Integer>——内部类型限定，
                // TWR/声明里出现；ProblemReferenceBinding 形态）：循环吞
                // （<…> | .seg）直到耗尽
                loop {
                    if self.at_punct("<") {
                        if let Some(text) = self.type_args_raw() {
                            name.push_str(&text);
                        } else {
                            break;
                        }
                    }
                    if self.at_punct(".") {
                        if self.peek(1).is_punct("@") {
                            // 段间注解：Map.@NonNull Entry（JSR 308 段级——
                            // PMD FullTypeAnnotations 抓获；注解原文并入名保
                            // 往返，打印 `Map.@NonNull Entry`——第 11 轮代理
                            // 抓获曾丢弃）
                            let seg_start = self.cur_start();
                            self.bump(); // .
                            let anno_start = self.cur_start();
                            self.skip_mods_annotations();
                            let anno_text = self.text_of(anno_start, self.cur_start()).trim().to_string();
                            if let Tok::Ident(seg) = &self.tok().tok {
                                let seg = seg.to_string();
                                self.bump();
                                // 注解跨行/多注解时归一为单空格分隔
                                let text = self
                                    .text_of(seg_start, self.t[self.pos - 1].end)
                                    .trim()
                                    .to_string();
                                let normalized = if anno_text.contains('\n') {
                                    format!(".{} {}", anno_text, seg)
                                } else {
                                    text
                                };
                                name.push_str(&normalized);
                            } else {
                                break;
                            }
                        } else if let Tok::Ident(seg) = &self.peek(1).tok {
                            // .new/.this/.super/.class 是表达式后缀不是类型段
                            if is_type_reserved(seg) {
                                break;
                            }
                            let seg = seg.to_string();
                            self.bump();
                            self.bump();
                            name.push('.');
                            name.push_str(&seg);
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                JType::Ref(name)
            }
            _ => return None,
        };
        Some(base)
    }

    /// 泛型实参原文（含尖括号）。处理 `>>`/`>>>` 多重闭合。
    fn type_args_raw(&mut self) -> Option<String> {
        if !self.at_punct("<") {
            return None;
        }
        let start = self.cur_start();
        let mut depth = 0i32;
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > 100_000 || self.at_eof() {
                return None;
            }
            if self.at_punct("<") {
                depth += 1;
                self.bump();
                continue;
            }
            let closes = match &self.tok().tok {
                Tok::Punct(">") => 1,
                Tok::Punct(">>") => 2,
                Tok::Punct(">>>") => 3,
                _ => 0,
            };
            if closes > 0 {
                depth -= closes;
                self.bump();
                if depth <= 0 {
                    break;
                }
                continue;
            }
            // 注解元素值可含 = @ {} 字面量（@Anno(clazz=X.class, arr={Y.class})——
            // JSR 308 类型注解可出现在泛型实参内；spoon Pozole 抓获：曾按"非法
            // 字符"整体拒绝 → new 表达式解析失败风暴）
            if matches!(self.tok().tok, Tok::Ident(_))
                || matches!(
                    &self.tok().tok,
                    Tok::Num(_) | Tok::Str(_) | Tok::Char(_) | Tok::TextBlock(_)
                )
                || matches!(
                    &self.tok().tok,
                    Tok::Punct(p)
                        if matches!(
                            *p,
                            "." | "," | "?" | "&" | "|" | "[" | "]" | "(" | ")" | "@" | "="
                                | "{" | "}" | "::"
                        )
                )
            {
                self.bump();
                continue;
            }
            return None; // 非法字符出现在泛型里
        }
        Some(self.text_of(start, self.t[self.pos - 1].end))
    }

    // ---- 语句 ----

    fn parse_block_raw(&mut self) -> JavaId {
        if !self.at_punct("{") {
            self.err_at("expected block");
            let text = self.sync_member();
            return self.ast.raw(&text);
        }
        let mut children = Vec::new();
        self.bump(); // {
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > STEP_GUARD {
                break;
            }
            if self.at_eof() {
                self.err_at("unexpected EOF in block");
                break;
            }
            if self.at_punct("}") {
                self.bump();
                break;
            }
            let pos_before = self.pos;
            let s = self.parse_stmt();
            // 停滞守卫：残缺语句起点是闭括号等**不消费失败位**（sync_stmt
            // 停在闭括号防吞外层收尾）→ 单 token Raw 保真 + 强制推进 1 步
            //（曾原位自旋 STEP_GUARD=2M 轮：4M 错误/529MB——lombok after-ecj
            // `for (…;; (…); …)` 残骸抓获；保 Raw 使输出逐字保真且幂等）
            if self.pos == pos_before && !self.at_punct("}") && !self.at_eof() {
                let tok_start = self.cur_start();
                let tok_end = self.tok().end;
                let text = self.text_of(tok_start, tok_end).trim().to_string();
                if !text.is_empty() {
                    children.push(self.ast.raw(&text));
                }
                self.bump();
                continue;
            }
            // Group（多声明符等）就地展开为兄弟语句
            if matches!(self.ast.data(s), &NodeData::Group) {
                children.extend(self.ast.children(s).iter().copied());
            } else if matches!(self.ast.data(s), &NodeData::Empty) {
                // 块内空语句（;）不进树：打印端本就不输出，留着会让
                // is_empty_block 判否 → if_else_empty 级联断链（ASTParser
                // 幂等失败 22 处抓获：`if (c) { ; } else {B}` 永不简化）。
                // 语句体位置的 Empty（while(x);）不经过此路径，不受影响。
            } else {
                children.push(s);
            }
        }
        self.ast.block(children)
    }

    fn parse_stmt(&mut self) -> JavaId {
        let start = self.cur_start();
        // 块
        if self.at_punct("{") {
            return self.parse_block_raw();
        }
        // 语句级注解：@Anno(...) 后跟声明/语句 → 整条原文保真（注解附着性暂不建模）
        if self.at_punct("@") {
            self.bump();
            self.bump(); // 注解名
            // 限定名后缀（@lombok.Cleanup 语句级限定注解）：跳过 .seg 段
            // 防假警告（曾遇 . 即败——lombok Cleanup 抓获：4 文件假报
            // unexpected token，内容虽保真但错误计数污染）
            while self.at_punct(".") {
                if matches!(self.peek(1).tok, Tok::Ident(_)) {
                    self.bump();
                    self.bump();
                } else {
                    break;
                }
            }
            if self.at_punct("(") {
                self.skip_balanced("(", ")");
            }
            let _ = self.parse_stmt();
            let end = self.tok().start;
            let text = self.text_of(start, end).trim().to_string();
            return if text.is_empty() {
                self.ast.empty()
            } else {
                self.ast.raw(&text)
            };
        }
        // 空语句
        if self.at_punct(";") {
            self.bump();
            return self.ast.empty();
        }
        // 语句关键字
        if self.at_kw("if") {
            self.bump();
            let cond = self
                .paren_expr()
                .unwrap_or_else(|| self.ast.raw("/*bad cond*/"));
            let then = self.stmt_or_block();
            let mut els = None;
            if self.at_kw("else") {
                self.bump();
                els = Some(self.stmt_or_block());
            }
            return self.ast.if_(cond, then, els);
        }
        if self.at_kw("while") {
            self.bump();
            let cond = self
                .paren_expr()
                .unwrap_or_else(|| self.ast.raw("/*bad cond*/"));
            let body = self.stmt_or_block();
            return self.ast.while_(cond, body);
        }
        if self.at_kw("do") {
            self.bump();
            let body = self.stmt_or_block();
            if !self.expect("while") {
                let text = self.take_raw(start);
                return self.ast.raw(&text);
            }
            let cond = self
                .paren_expr()
                .unwrap_or_else(|| self.ast.raw("/*bad cond*/"));
            self.expect(";");
            return self.ast.do_while(body, cond);
        }
        if self.at_kw("for") {
            self.bump();
            return self.parse_for();
        }
        if self.at_kw("try") {
            self.bump();
            return self.parse_try();
        }
        if self.at_kw("switch") {
            self.bump();
            return self.parse_switch();
        }
        if self.at_kw("synchronized") {
            self.bump();
            let lock = self
                .paren_expr()
                .unwrap_or_else(|| self.ast.raw("/*bad lock*/"));
            let body = self.parse_block_raw();
            return self.ast.synchronized(lock, body);
        }
        if self.at_kw("return") {
            self.bump();
            let value = if self.at_punct(";") {
                None
            } else {
                self.parse_expr(PREC_ASSIGN)
            };
            self.expect(";");
            return self.ast.ret(value);
        }
        if self.at_kw("throw") {
            self.bump();
            let e = self.parse_expr(PREC_ASSIGN);
            self.expect(";");
            match e {
                Some(e) => return self.ast.throw(e),
                None => {
                    let t = self.ast.raw("/*bad throw*/");
                    return self.ast.throw(t);
                }
            }
        }
        if self.at_kw("break") || self.at_kw("continue") {
            let is_break = self.at_kw("break");
            self.bump();
            let label = match &self.tok().tok {
                Tok::Ident(i) if !self.at_punct(";") => {
                    let l = i.to_string();
                    self.bump();
                    Some(l)
                }
                _ => None,
            };
            self.expect(";");
            return if is_break {
                self.ast.break_(label.as_deref())
            } else {
                self.ast.continue_(label.as_deref())
            };
        }
        // 局部 record/class/interface/enum 声明（Java 16+ 局部类型——
        // ES93GenericFlatVectorsReader 抓获：语句级解析曾落入表达式路径
        // 产生 4M 错误风暴）。按 Raw 整段保真（类体语义不触碰）。
        // 前导修饰符（abstract/final/注解——guava CompactLinkedHashMap
        // 的 `abstract class Class3` 曾丢 abstract）一并并入原文
        if let Some(type_kw_pos) = self.find_local_type_decl() {
            let start = self.cur_start();
            self.pos = type_kw_pos;
            self.bump(); // 关键字
            if self.at_punct("<") {
                let _ = self.skip_balanced("<", ">");
            }
            if self.at_punct("(") {
                let _ = self.skip_balanced("(", ")");
            }
            let mut guard = 0usize;
            while !self.at_eof() && !self.at_punct("{") && guard < 100_000 {
                guard += 1;
                self.bump();
            }
            if self.at_punct("{") {
                if let Some(_body) = self.skip_balanced_braces() {
                    let end = self.t[self.pos - 1].end;
                    let full = self.text_of(start, end);
                    return self.ast.raw(full.trim());
                }
            }
        }
        if self.at_kw("record") || self.at_kw("class") || self.at_kw("interface")
            || self.at_kw("enum")
        {
            let save = self.pos;
            // record 需要后随 ( 或 Ident（record 也可能是变量名：
            // `record record = ...` 合法）——用形态判定
            // record 是**上下文关键字**：声明形态 = record Ident ( / < / {
            //（DataSourceInventoryCounters 抓获：`record(counts, m);`
            // 调用曾被误判成声明）。调用形态 peek(1) 是 `(` —— 直接排除
            let is_type_decl = match &self.tok().tok {
                Tok::Ident(_) if self.at_kw("record") => {
                    matches!(&self.peek(1).tok, Tok::Ident(_))
                        && matches!(
                            &self.peek(2).tok,
                            Tok::Punct("(") | Tok::Punct("<") | Tok::Punct("{")
                        )
                }
                _ => {
                    // class/interface/enum：后随 Ident 即类型声明（x = class …
                    // 不合法；但 `enum` 作变量名后随 = / ; 时排除）
                    matches!(&self.peek(1).tok, Tok::Ident(_))
                        && !matches!(&self.peek(2).tok, Tok::Punct("=") | Tok::Punct(";") | Tok::Punct(","))
                }
            };
            if is_type_decl {
                let start = self.cur_start();
                self.bump(); // 关键字
                // 泛型/头/体整体吞（平衡大括号即类体）
                if self.at_punct("<") {
                    let _ = self.skip_balanced("<", ">");
                }
                // record 头 (…)
                if self.at_punct("(") {
                    let _ = self.skip_balanced("(", ")");
                }
                // implements/extends 列表扫到 {
                let mut guard = 0usize;
                while !self.at_eof() && !self.at_punct("{") && guard < 100_000 {
                    guard += 1;
                    self.bump();
                }
                if self.at_punct("{") {
                    if let Some(body) = self.skip_balanced_braces() {
                        let text = format!("{} {}", self.text_of(start, self.t[self.pos.saturating_sub(1)].end - body.len()), "");
                        // 完整原文（含体）——重新取：start 到体末
                        let _ = text;
                        let end = self.t[self.pos - 1].end;
                        let full = self.text_of(start, end);
                        return self.ast.raw(full.trim());
                    }
                }
                self.pos = save; // 形态不完整——回退按表达式解
            }
        }
        if self.at_kw("yield") {
            // switch 表达式块内的 yield：结构化节点（孩子 = 表达式——
            // Raw 曾使一切 AST 分析对 yield 体内引用全盲，
            // InputUnusedLocalVariableSwitchExpression 抓获：++line 在
            // yield switch 内，逃逸/死码分析看不见 → line 声明被删）。
            // yield 也可作方法名（openjdk T8326204：yield((Map) null, 2)
            // 是调用）——解析出非 `;` 收尾即回退按表达式语句重解
            let save = self.pos;
            let err_len = self.errs.len();
            self.bump();
            if let Some(e) = self.parse_expr(PREC_ASSIGN) {
                if self.eat(";") {
                    return self.ast.yield_(e);
                }
            }
            // 回退：试探期间的诊断一并回滚（T8326204 曾泄漏 1 错误）
            self.pos = save;
            self.errs.truncate(err_len);
        }
        if self.at_kw("assert") {
            // assert 作方法名（PMD jdkversiontests：assert() 调用——1.4 前
            // 旧代码兼容）：表达式解析失败即回退按语句重解（不消费 assert）
            let save_assert = self.pos;
            let err_len = self.errs.len();
            self.bump();
            let parsed = self.parse_expr(PREC_ASSIGN);
            let cond = match parsed {
                None => {
                    self.pos = save_assert;
                    self.errs.truncate(err_len);
                    None // 回退：落入下方语句解析（assert 按方法名）
                }
                Some(c) => Some(c),
            };
            if let Some(cond) = cond {
                let cond = cond;
                let msg = if self.eat(":") {
                    self.parse_expr(PREC_ASSIGN)
                } else {
                    None
                };
                self.expect(";");
                return self.ast.assert_(cond, msg);
            }
        }
        {
            // 局部类/接口/枚举/record 声明：原文保真。可带 final/@Anno 修饰
            //（final record X(…) {}——checkstyle Java15FinalLocalRecord 抓获：
            // 修饰前缀曾使局部 record 走变量声明路径 → 风暴）
            let save = self.pos;
            let raw_start = self.cur_start();
            // 修饰全形态（strictfp enum E{…};——checkstyle Java16LocalEnum：
            // 修饰集含 strictfp/public/static 等，非仅 final/@Anno）。
            // 修饰与注解**任意交错**（F2 抓获：`final @Deprecated static
            // record R(…) {}` 的 static 在注解后——旧三段式循环止于 @，
            // static 残留使类型关键字判定失败 → 局部 record 退化为残句
            // 输出丢 {} 不可编译）。raw_start 记在修饰前——注解/修饰
            // 一并入原文（@Deprecated 有运行时可观察性，丢弃即有损）
            loop {
                if matches!(&self.tok().tok, Tok::Ident(i) if is_modifier_kw(i)) {
                    self.bump();
                    continue;
                }
                if self.at_value_decl() {
                    self.bump();
                    continue;
                }
                if self.at_punct("@") && !self.at_annotation_decl() {
                    self.bump();
                    self.bump();
                    if self.at_punct("(") {
                        self.skip_balanced("(", ")");
                    }
                    continue;
                }
                break;
            }
            if self.at_kw("class")
                || self.at_kw("interface")
                || self.at_kw("enum")
                || self.at_kw("record")
            {
                let text = self.sync_member();
                // 尾分号（enum E{…}; 容错形态）一并并入
                if self.at_punct(";") {
                    self.bump();
                }
                // 从修饰前起整体保真（sync_member 只覆盖关键字起的段）
                let text = self
                    .text_of(raw_start, self.t[self.pos.min(self.t.len() - 1)].start)
                    .trim()
                    .to_string();
                return self.ast.raw(&text);
            }
            self.pos = save;
        }
        // 标签语句： Ident ':'
        if let Tok::Ident(_) = &self.tok().tok {
            if self.peek(1).is_punct(":") && !self.peek(2).text().starts_with(':') {
                let name = match &self.tok().tok {
                    Tok::Ident(i) => i.to_string(),
                    _ => unreachable!(),
                };
                self.bump();
                self.bump();
                let inner = self.parse_stmt();
                return self.ast.label(&name, inner);
            }
        }
        // 局部变量声明 vs 表达式语句（回溯判定）
        let save = self.pos;
        if let Some((ty, had_final)) = self.try_decl_prefix() {
            let mut first_name = match &self.tok().tok {
                Tok::Ident(i) => {
                    let n = i.to_string();
                    self.bump();
                    Some(n)
                }
                _ => None,
            };
            if first_name.is_none() {
                self.pos = save;
                return self.expr_stmt_fallback(start);
            }
            let name0 = first_name.take().unwrap();
            let mut extra = 0u32;
            while self.at_punct("[") && self.peek(1).is_punct("]") {
                self.bump();
                self.bump();
                extra += 1;
            }
            let base_ty = ty.clone();
            let ty0 = wrap_dims(ty, extra);
            let mut decls: Vec<(String, JType, Option<JavaId>)> = Vec::new();
            // 第一个声明符
            let had_eq = self.eat("=");
            let init0 = if had_eq {
                self.parse_expr(PREC_ASSIGN)
            } else {
                None
            };
            let init0 = self.wrap_decl_init_array(init0, &ty0);
            if had_eq && init0.is_none() {
                // `int x = ;` 这类残缺：整条语句原文保真
                return self.raw_from(start);
            }
            decls.push((name0, ty0, init0));
            // 后续声明符 `, name [= init]`
            let mut guard = 0usize;
            loop {
                guard += 1;
                if guard > 10_000 {
                    break;
                }
                if !self.eat(",") {
                    break;
                }
                // 声明符间注解（int a, @Deprecated b;）：跳过
                while self.at_punct("@") {
                    self.bump();
                    self.bump();
                    if self.at_punct("(") {
                        self.skip_balanced("(", ")");
                    }
                }
                let name = match &self.tok().tok {
                    Tok::Ident(i) => {
                        let n = i.to_string();
                        self.bump();
                        n
                    }
                    _ => {
                        self.err_at("bad declarator");
                        break;
                    }
                };
                let mut d_extra = 0u32;
                loop {
                    self.skip_mods_annotations();
                    if self.at_punct("[") && self.peek(1).is_punct("]") {
                        self.bump();
                        self.bump();
                        d_extra += 1;
                    } else {
                        break;
                    }
                }
                let had_eq = self.eat("=");
                let init = if had_eq {
                    self.parse_expr(PREC_ASSIGN)
                } else {
                    None
                };
                if had_eq && init.is_none() {
                    return self.raw_from(start);
                }
                // 后续声明符维度**相加**（JLS 14.4：声明类型 + 声明符自带
                // C 风格维度——`int[] p, q[][]` 的 q = int[3]——A5e 抓获：
                // 曾按首声明符剥一层继承使 q 降维）。基类型 = 解析出的
                // 声明类型（parse_type 的 dims），非首声明符的整型
                let base = base_ty.clone();
                let decl_ty = wrap_dims(base, d_extra);
                let init = self.wrap_decl_init_array(init, &decl_ty);
                decls.push((name, decl_ty, init));
            }
            self.expect(";");
            if decls.len() == 1 {
                let (n, t, i) = &decls[0];
                return self.ast.var_decl_final(n, t.clone(), i.clone(), had_final);
            }
            // 多声明符 → 合成 Group（无作用域）：语句列表处就地展开；
            // 逃逸到打印时按同缩进无括号输出。绝不能用 Block——那是词法
            // 作用域，会把声明的作用域错误地圈进花括号（曾经因此被
            // 零用途 DeadStore 误删逃逸变量，对抗差分抓获）。
            let stmts = decls
                .iter()
                .map(|(n, t, i)| self.ast.var_decl_final(n, t.clone(), i.clone(), had_final))
                .collect();
            return self.ast.group(stmts);
        }
        self.pos = save;
        self.expr_stmt_fallback(start)
    }

    fn expr_stmt_fallback(&mut self, start: usize) -> JavaId {
        match self.parse_expr(PREC_ASSIGN) {
            Some(e) => {
                if self.eat(";") || self.at_punct("}") || self.at_eof() {
                    // 正常，或块尾缺分号（容忍）
                    self.ast.expr_stmt(e)
                } else {
                    // 表达式后还有残留 token → 语句不完整，整体 Raw 保真
                    self.raw_from(start)
                }
            }
            None => self.raw_from(start),
        }
    }

    /// 尝试判定"局部变量声明"前缀（类型 + 名字）。成功则消费类型与名字之前的
    /// token（名字不消费，由调用方处理），失败回滚。
    fn try_decl_prefix(&mut self) -> Option<(JType, bool)> {
        let save = self.pos;
        // var x = ...（上下文关键字）
        if self.at_kw("var") {
            if let Tok::Ident(_) = &self.peek(1).tok {
                let t = JType::Var;
                self.bump();
                return Some((t, false));
            }
            return None;
        }
        // final/注解交错前缀（@A final C c / final @A C c——GJF testdata
        // TryWithResources 抓获：TWR 资源头注解须按声明解析，误入表达式
        // 解析会风暴）。final 的存在记入 ctx（JLS 4.12.4：case 标签常量
        // 的必要条件——final 曾被静默丢弃，`case ARRAY_BOUND:` 失常量性
        // 不可编译，LargeFile.java 抓获）
        let had_final = self.skip_mods_annotations_final();
        let ty = self.parse_type();
        if ty.is_none() {
            self.pos = save;
            return None;
        }
        let ty = ty.unwrap();
        match &self.tok().tok {
            // 返回 (类型, 是否 final)
            
            // instanceof is by no means a valid declaration name (k instanceof String was once judged
            // as the declaration "type k, name instanceof" — jdk-sources SignatureUtil:
            // case EDDSA -> k instanceof EdECPrivateKey ? … storm root cause)
            Tok::Ident(i) if *i != "instanceof" => Some((ty, had_final)),
            _ => {
                self.pos = save;
                None
            }
        }
    }

    fn stmt_or_block(&mut self) -> JavaId {
        if self.at_punct("{") {
            self.parse_block_raw()
        } else if self.at_punct(";") {
            self.bump();
            self.ast.empty()
        } else {
            self.parse_stmt()
        }
    }

    fn paren_expr(&mut self) -> Option<JavaId> {
        if !self.expect("(") {
            return None;
        }
        let e = self.parse_expr(PREC_ASSIGN);
        self.expect(")");
        e
    }

    fn parse_for(&mut self) -> JavaId {
        // for_start 含 `for` 关键字（parse_stmt 已 bump——cur_start 在
        // `(` 上；pos-1 即关键字 token）。破损回退的 Raw 若丢关键字，
        // 输出 `(int $i = 0;; …)` 顶层残句不可解析（R13 P0-1 抓获）
        let for_start = self.t[self.pos - 1].start;
        if !self.expect("(") {
            let text = self.sync_stmt();
            return self.ast.raw(&text);
        }
        // for-each 判定（先跳过 final 与**注解**——for (@Anno int i : a)
        // 是合法形态；注解原文丢弃（容错优先），否则解析风暴）
        let save = self.pos;
        while self.at_kw("final") || self.at_punct("@") {
            if self.at_punct("@") {
                self.bump(); // @
                self.bump(); // 注解名
                if self.at_punct("(") {
                    self.skip_balanced("(", ")");
                }
                continue;
            }
            self.bump();
        }
        // for-each record 模式：for (ARecord(String name, final int age) :
        // records)（Java 21，checkstyle 抓获）——循环变量是模式而非名字，
        // ForEach 节点装不下 → 整条 for 语句原文保真（含前缀注解/final）
        {
            let det = self.pos;
            if matches!(self.tok().tok, Tok::Ident(_)) {
                self.bump();
                if self.at_punct("<") {
                    let _ = self.skip_balanced("<", ">");
                }
                if self.at_punct("(") {
                    let ok = self.skip_balanced("(", ")").is_some() && self.at_punct(":");
                    if ok {
                        // 重建整条 for：for 关键字已被 parse_stmt 消费——头部从
                        // （ 起原文拼回前缀；平衡扫到 for 头收尾 )（深度前值为 0
                        // 的 ）才是头括号——模式自身的 () 不计）；体随行
                        self.pos = det;
                        let mut depth = 0i32;
                        let mut guard = 0usize;
                        while !self.at_eof() && guard < STEP_GUARD {
                            guard += 1;
                            if self.at_punct("(") {
                                depth += 1;
                                self.bump();
                                continue;
                            }
                            if self.at_punct(")") {
                                if depth == 0 {
                                    self.bump();
                                    break;
                                }
                                depth -= 1;
                                self.bump();
                                continue;
                            }
                            self.bump();
                        }
                        let header = self.text_of(for_start, self.t[self.pos - 1].end);
                        let mut text = header;
                        if self.at_punct("{") {
                            if let Some(b) = self.skip_balanced_braces() {
                                text.push(' ');
                                text.push_str(&b);
                            }
                        } else {
                            let b = self.sync_stmt();
                            if !b.is_empty() {
                                text.push(' ');
                                text.push_str(&b);
                            }
                        }
                        return self.ast.raw(text.trim());
                    }
                }
            }
            self.pos = det;
        }
        if let Some(ty) = self.parse_type() {
            if let Tok::Ident(n) = &self.tok().tok {
                let name = n.to_string();
                self.bump();
                // C 风格维度变量：for (int _[] : …) / for (int x[] : arr)
                //（openjdk UnnamedErrors 抓获——曾残留 [ 触发经典 for 路径风暴）
                let mut dims = 0u32;
                while self.at_punct("[") && self.peek(1).is_punct("]") {
                    self.bump();
                    self.bump();
                    dims += 1;
                }
                let ty = if dims > 0 { wrap_dims(ty, dims) } else { ty };
                if self.at_punct(":") {
                    self.bump();
                    let iterable = self.parse_expr(PREC_ASSIGN);
                    self.expect(")");
                    let body = self.stmt_or_block();
                    let iterable = match iterable {
                        Some(e) => e,
                        None => self.ast.empty(),
                    };
                    return self.ast.for_each(&name, ty, iterable, body);
                }
            }
        }
        self.pos = save;
        // 经典 for
        let mut inits: Vec<JavaId> = Vec::new();
        if !self.at_punct(";") {
            let save2 = self.pos;
            // init 前置注解（for (@Anno int i = 0; …)）：跳过（原文丢弃）
            while self.at_punct("@") {
                self.bump();
                self.bump();
                if self.at_punct("(") {
                    self.skip_balanced("(", ")");
                }
            }
            if let Some((ty, had_final)) = self.try_decl_prefix() {
                // 声明式 init：`int i = 0, j = 1`（后续声明符无类型 token）
                let name = match &self.tok().tok {
                    Tok::Ident(i) => {
                        let n = i.to_string();
                        self.bump();
                        n
                    }
                    _ => {
                        self.err_at("bad for-init");
                        String::new()
                    }
                };
                let mut extra = 0u32;
                while self.at_punct("[") && self.peek(1).is_punct("]") {
                    self.bump();
                    self.bump();
                    extra += 1;
                }
                let had_eq = self.eat("=");
                let init = if had_eq {
                    self.parse_expr(PREC_ASSIGN)
                } else {
                    None
                };
                if had_eq && init.is_none() {
                    return self.raw_from(for_start);
                }
                // type_dims 取**包装前**的解析类型（`int k[]` 的 C 风格
                // extra 不能感染兄弟声明符 l——for_ty 含 extra 曾使
                // `for (int k[], l = 0)` 打出 l[]）
                let type_dims = {
                    let mut d = 0u32;
                    let mut t = &ty;
                    while let JType::Array(inner) = t {
                        d += 1;
                        t = inner;
                    }
                    d
                };
                let for_ty = wrap_dims(ty, extra);
                let init = self.wrap_decl_init_array(init, &for_ty);
                inits.push(self.ast.var_decl(&name, for_ty, init));
                while self.eat(",") {
                    let n2 = match &self.tok().tok {
                        Tok::Ident(i) => {
                            let n = i.to_string();
                            self.bump();
                            n
                        }
                        _ => {
                            self.err_at("bad for-init declarator");
                            break;
                        }
                    };
                    let mut d_extra = 0u32;
                    loop {
                        self.skip_mods_annotations();
                        if self.at_punct("[") && self.peek(1).is_punct("]") {
                            self.bump();
                            self.bump();
                            d_extra += 1;
                        } else {
                            break;
                        }
                    }
                    let init2 = if self.eat("=") {
                        self.parse_expr(PREC_ASSIGN)
                    } else {
                        None
                    };
                    // with_type=false 打印时只输出名字，类型用 Var 占位即可。
                    // 维度**相加**（JLS 14.14 与 14.4 同：声明类型 dims +
                    // 声明符自带——`for (String[] s, t[][];;)` 的 t =
                    // String[3]——A5e 抓获曾只记自带维度降维）
                    let for_ty2 = wrap_dims(JType::Var, type_dims + d_extra);
                    let init2 = self.wrap_decl_init_array(init2, &for_ty2);
                    inits.push(self.ast.var_decl(&n2, for_ty2, init2));
                }
                self.expect(";");
            } else {
                self.pos = save2;
                // 表达式 init
                loop {
                    if self.at_punct(";") || self.at_eof() {
                        break;
                    }
                    match self.parse_expr(PREC_ASSIGN) {
                        Some(e) => inits.push(self.ast.expr_stmt(e)),
                        None => break,
                    }
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect(";");
            }
        } else {
            self.bump();
        }
        let cond = if self.at_punct(";") {
            None
        } else {
            self.parse_expr(PREC_ASSIGN)
        };
        self.expect(";");
        let mut steps = Vec::new();
        while !self.at_punct(")") && !self.at_eof() {
            if let Some(e) = self.parse_expr(PREC_ASSIGN) {
                steps.push(self.ast.expr_stmt(e));
            } else {
                break;
            }
            if !self.eat(",") {
                break;
            }
        }
        if !self.expect(")") {
            // for 头破损（三半分号等非法形态——after-ecj 残骸 R13 P0-1）：
            // 部分构造的 For + 孤儿残句曾使循环体语句丢失、类尾成员移位
            // 顶层、非幂等。整段 Raw 保真（sync 到 `;`/语句边界；体块作
            // 兄弟语句另行解析，类结构不破坏）
            return self.raw_from(for_start);
        }
        let body = self.stmt_or_block();
        // steps 存的是 ExprStmt，解包成裸表达式（For 头部打印不带分号）
        let steps_bare: Vec<JavaId> = steps
            .into_iter()
            .map(|s| match self.ast.data(s) {
                NodeData::ExprStmt => self.ast.children(s)[0],
                _ => s,
            })
            .collect();
        self.ast.for_(inits, cond, steps_bare, body)
    }

    fn parse_try(&mut self) -> JavaId {
        let mut resources = Vec::new();
        if self.at_punct("(") {
            self.bump();
            let mut guard = 0usize;
            loop {
                guard += 1;
                if guard > 10_000 || self.at_eof() {
                    break;
                }
                if self.at_punct(")") {
                    self.bump();
                    break;
                }
                // 资源：声明或表达式（@ 开头必为带注解声明——表达式不可能
                // 以注解开头）
                let save = self.pos;
                let mut is_decl = false;
                if self.at_kw("var")
                    || self.at_kw("final")
                    || self.at_punct("@")
                    || self.try_decl_prefix().is_some()
                {
                    is_decl = true;
                }
                self.pos = save;
                if is_decl {
                    if let Some((ty, had_final)) = self.try_decl_prefix() {
                        let name = match &self.tok().tok {
                            Tok::Ident(i) => {
                                let n = i.to_string();
                                self.bump();
                                n
                            }
                            _ => {
                                self.err_at("bad resource");
                                break;
                            }
                        };
                        let init = if self.eat("=") {
                            self.parse_expr(PREC_ASSIGN)
                        } else {
                            None
                        };
                        resources.push(self.ast.var_decl(&name, ty, init));
                    }
                } else if let Some(e) = self.parse_expr(PREC_ASSIGN) {
                    resources.push(self.ast.expr_stmt(e));
                } else {
                    self.err_at("bad resource");
                    break;
                }
                if !self.eat(";") {
                    self.expect(")");
                    break;
                }
            }
        }
        let try_block = self.parse_block_raw();
        let mut catches = Vec::new();
        while self.at_kw("catch") {
            self.bump();
            if !self.expect("(") {
                break;
            }
            // 多类型 catch：A | B | C（可带 final：catch (final IOException e)——
            // JavaConceptsMethods 抓获：曾使 parse_type 失败 → 风暴）
            let mut ty_parts: Vec<String> = Vec::new();
            let mut guard = 0usize;
            loop {
                guard += 1;
                if guard > 10_000 {
                    break;
                }
                // catch 形参 final 保留（mockito 5 + lombok 21 文件抓获：
                // 曾丢弃——风格保真；start 前移到 final 之前使 ty_raw 含它）。
                // 注解与 final 任意序（JLS 14.20 VariableModifier——A9g 抓获：
                // `catch (@A final X e)` 曾 parse_type 失败输出结构破坏且
                // 非幂等）——与方法形参路径同法跳过
                let start = self.cur_start();
                while self.at_kw("final") || self.at_punct("@") {
                    if self.at_punct("@") {
                        self.bump(); // @
                        self.bump(); // 注解名
                        if self.at_punct("(") {
                            self.skip_balanced("(", ")");
                        }
                        continue;
                    }
                    self.bump();
                }
                if self.parse_type().is_none() {
                    self.err_at("bad catch type");
                    break;
                }
                ty_parts.push(self.text_of(start, self.t[self.pos - 1].end).trim().to_string());
                if !self.eat("|") {
                    break;
                }
            }
            // 裸无名参数：catch (_)（Java 21+ 匿名 catch——openjdk
            // UnnamedErrors 抓获：`_` 先被 parse_type 当类型名吃掉 →
            // ty_parts==["_"] 且无名位收尾即匿名形态）
            if ty_parts.len() == 1 && ty_parts[0] == "_" && self.at_punct(")") {
                self.bump();
                let block = self.parse_block_raw();
                catches.push(self.ast.catch_("_", "_", block));
                continue;
            }
            let name = match &self.tok().tok {
                Tok::Ident(i) => {
                    let n = i.to_string();
                    self.bump();
                    n
                }
                _ => {
                    self.err_at("bad catch parameter");
                    String::new()
                }
            };
            self.expect(")");
            let block = self.parse_block_raw();
            catches.push(self.ast.catch_(&ty_parts.join(" | "), &name, block));
        }
        let mut finally = None;
        if self.at_kw("finally") {
            self.bump();
            finally = Some(self.parse_block_raw());
        }
        self.ast.try_(resources, try_block, catches, finally)
    }

    fn parse_switch(&mut self) -> JavaId {
        let subject = self
            .paren_expr()
            .unwrap_or_else(|| self.ast.raw("/*bad subject*/"));
        if !self.expect("{") {
            let text = self.sync_stmt();
            return self.ast.raw(&text);
        }
        let mut cases = Vec::new();
        let mut guard = 0usize;
        let mut stall_pos = usize::MAX;
        loop {
            guard += 1;
            if guard > 100_000 || self.at_eof() {
                break;
            }
            if self.at_punct("}") {
                self.bump();
                break;
            }
            // 位置停滞后强制推进到块尾：case 解析失败但 sync 不动时空转
            // 到 10 万次（ES 文件 100 行膨胀 10 万行的第二风暴源）。
            // 推进由下方「垃圾 case」分支的 sync_stmt 承担；此处兜底
            // 跳出（把剩余部分整体 raw）
            if self.pos == stall_pos {
                let start = self.cur_start();
                let text = self.sync_to_block_end();
                if !text.is_empty() {
                    cases.push(self.ast.raw(&text));
                }
                let _ = start;
                break;
            }
            stall_pos = self.pos;
            let mut labels = Vec::new();
            let mut is_default = false;
            if self.at_kw("case") {
                self.bump();
                self.in_case_label = true;
                loop {
                    // 组合标签尾：`case null, default:`（Java 21 JLS 14.11.1）
                    // ——default 关键字收尾（SwitchDouble 抓获：曾拆成
                    // `case null:` + `case default:` 不可编译）
                    if self.at_kw("default") {
                        self.bump();
                        is_default = true;
                        break;
                    }
                    // 标签头修饰：final/注解可修饰整个模式标签（case final
                    // Pair<I>(…)——checkstyle BindingWithModifiers）——跳过并入
                    // 原文保真
                    self.skip_mods_annotations();
                    let handled = 'label: {
                        // 1. 括号化形态：先试 cast 表达式标签（case (int) X /
                        // case (char) 0xFFFF——jdk-sources P11*/Switch02 拷问），
                        // 收尾合法（->/:/,/when）即用；否则括号化模式
                        //（case (String s) when … ->）整段 raw
                        if self.at_punct("(") {
                            let start = self.cur_start();
                            let save_tok = self.pos;
                            let err_len = self.errs.len();
                            if let Some(e2) = self.parse_expr(PREC_TERNARY) {
                                if self.at_punct("->")
                                    || self.at_punct(":")
                                    || self.at_punct(",")
                                    || self.at_kw("when")
                                {
                                    labels.push(e2);
                                    break 'label true;
                                }
                            }
                            // 试探失败回退（位置 + 泄漏诊断）
                            self.pos = save_tok;
                            self.errs.truncate(err_len);
                            self.skip_balanced("(", ")");
                            self.consume_when_guard();
                            let text = self.text_of(start, self.t[self.pos - 1].end);
                            labels.push(self.ast.raw(text.trim()));
                            break 'label true;
                        }
                        // 2. record 模式：[Name|.Name|<T>…]*( ——判定后整体原文
                        // 括号内是类型模式/嵌套模式而非实参，误入调用解析会
                        // 风暴；含泛型/限定名/模式绑定名/when 守卫全形态
                        {
                            let save = self.pos;
                            let mut ok = false;
                            if matches!(self.tok().tok, Tok::Ident(_)) {
                                self.bump();
                                if self.at_punct("<") {
                                    let _ = self.skip_balanced("<", ">");
                                }
                                while self.at_punct(".")
                                    && matches!(self.peek(1).tok, Tok::Ident(_))
                                {
                                    self.bump();
                                    self.bump();
                                    if self.at_punct("<") {
                                        let _ = self.skip_balanced("<", ">");
                                    }
                                }
                                ok = self.at_punct("(");
                            }
                            self.pos = save;
                            if ok {
                                let start = self.cur_start();
                                self.bump(); // record 名
                                if self.at_punct("<") {
                                    self.skip_balanced("<", ">");
                                }
                                while self.at_punct(".")
                                    && matches!(self.peek(1).tok, Tok::Ident(_))
                                {
                                    self.bump();
                                    self.bump();
                                    if self.at_punct("<") {
                                        self.skip_balanced("<", ">");
                                    }
                                }
                                self.skip_balanced("(", ")");
                                // 模式绑定名：case Box<String>(String s) box
                                if matches!(&self.tok().tok, Tok::Ident(i) if *i != "when") {
                                    self.bump();
                                }
                                self.consume_when_guard();
                                let text = self.text_of(start, self.t[self.pos - 1].end);
                                labels.push(self.ast.raw(text.trim()));
                                break 'label true;
                            }
                        }
                        // 3. 类型模式（含数组维度与 C 风格后缀）：case Foo f /
                        // case int[] arr / case Long a[]（openjdk Switches、
                        // T8309054）——parse_type 全形 + 绑定名 + []*，须以
                        // ->/: 收尾（防常量表达式误判），否则回退
                        {
                            let save = self.pos;
                            let start = self.cur_start();
                            if self.parse_type().is_some() {
                                if let Tok::Ident(bind) = &self.tok().tok {
                                    if !is_reserved_after_type(bind) && *bind != "when" {
                                        self.bump();
                                        while self.at_punct("[")
                                            && self.peek(1).is_punct("]")
                                        {
                                            self.bump();
                                            self.bump();
                                        }
                                        self.consume_when_guard();
                                        // 收尾：->/: 或 ,（多标签 case Integer
                                        // _, String _ ->——UnnamedErrors 抓获：
                                        // 曾只认 ->/: → 首标签拒绝风暴）
                                        if self.at_punct("->")
                                            || self.at_punct(":")
                                            || self.at_punct(",")
                                        {
                                            let text =
                                                self.text_of(start, self.t[self.pos - 1].end);
                                            labels.push(self.ast.raw(text.trim()));
                                            break 'label true;
                                        }
                                    }
                                }
                            }
                            self.pos = save;
                        }
                        // 4. 常量表达式标签
                        if let Some(e) = self.parse_expr(PREC_TERNARY) {
                            labels.push(e);
                            break 'label true;
                        }
                        false
                    };
                    if !handled {
                        self.err_at("bad case label");
                        break;
                    }
                    if self.eat(",") {
                        continue;
                    }
                    break;
                }
                self.in_case_label = false;
            } else if self.at_kw("default") {
                self.bump();
                is_default = true;
            } else {
                // case 区外的垃圾 → 原文。闭括号等**不消费失败位**（sync
                // 停在 depth-0 闭括号）：单 token raw + 强制推进——否则
                // 上层 stall 兜底把剩余语句整体吞进一个 raw（R13c 的
                // m.put/unmodifiableMap 丢失即此）
                let pos_before = self.pos;
                let text = self.sync_stmt();
                if self.pos == pos_before && !self.at_eof() && !self.at_punct("}") {
                    let ts = self.cur_start();
                    let te = self.tok().end;
                    let t2 = self.text_of(ts, te).trim().to_string();
                    cases.push(self.ast.raw(&t2));
                    self.bump();
                    continue;
                }
                cases.push(self.ast.raw(&text));
                continue;
            }
            let arrow = self.eat("->");
            let mut stmts = Vec::new();
            if arrow {
                if self.at_punct("{") {
                    stmts.push(self.parse_block_raw());
                } else if self.at_kw("switch") {
                    // 箭头体是 switch 表达式（case 1 -> switch (b) {…}）：
                    // 按表达式解析（先消耗关键字——parse_switch 期待
                    // paren_expr 上下文；语句形态会吞 case 体的语句语义）。
                    // 语句表达式形态带分号（JLS 15.28 switch 表达式可作
                    // 表达式语句——checkstyle 语料 `…};` 实形）
                    self.bump();
                    let e = self.parse_switch();
                    self.eat(";");
                    let es = self.ast.expr_stmt(e);
                    stmts.push(es);
                } else {
                    let s = self.parse_stmt();
                    if matches!(self.ast.data(s), &NodeData::Group) {
                        stmts.extend(self.ast.children(s).iter().copied());
                    } else {
                        stmts.push(s);
                    }
                }
            } else {
                self.expect(":");
                let mut g2 = 0usize;
                let mut last_pos = usize::MAX;
                loop {
                    g2 += 1;
                    if g2 > STEP_GUARD || self.at_eof() {
                        break;
                    }
                    if self.at_kw("case") || self.at_kw("default") || self.at_punct("}") {
                        break;
                    }
                    // **位置不动即跳出**：语句解析失败但 sync 未推进时
                    // 旧行为空转到 STEP_GUARD（10 万次报错风暴 → 打印器
                    // 产出 10 万个 /* bad case */——ES
                    // InsertDefaultInnerTimeSeriesAggregate 曾 100 行膨胀
                    // 成 10 万行）。跳到块尾让 Raw 保真接管
                    if self.pos == last_pos {
                        break;
                    }
                    last_pos = self.pos;
                    let s = self.parse_stmt();
                    if matches!(self.ast.data(s), &NodeData::Group) {
                        stmts.extend(self.ast.children(s).iter().copied());
                    } else if !matches!(self.ast.data(s), &NodeData::Empty) {
                        // 空语句（`case X:;` 的裸 ;）不进树——打印空行后重
                        // 解析即消失，破坏幂等（Switches F3 抓获）
                        stmts.push(s);
                    }
                }
            }
            cases.push(self.ast.case_(labels, is_default, arrow, stmts));
        }
        self.ast.switch_(subject, cases)
    }

    // ---- 表达式（Pratt）----

    fn parse_expr(&mut self, min_prec: u8) -> Option<JavaId> {
        let mut lhs = self.parse_unary()?;
        loop {
            let (op_prec, right_assoc) = self.binop_at();
            if self.at_kw("instanceof") && PREC_RELATIONAL >= min_prec {
                self.bump();
                // 模式修饰：final 与注解交错（x instanceof @A final @B C
                // pattern——GJF I588 / checkstyle BindingWithModifiers）；
                // static 为 javac 拒绝的垃圾修饰（openjdk NoModifiersOnBinding
                // 负向测试）——容忍跳过（容错优先，输出仍合法）
                while self.at_kw("static") {
                    self.bump();
                }
                let pat_start = self.cur_start();
                // 注解捕获（instanceof @A String s 的类型注解 / @A final @B C
                // 的模式注解——GJF I588 / checkstyle BindingWithModifiers）：
                // 原文并入类型名保往返（曾整体跳过丢弃——TU 电池抓获）；
                // final 与注解交错容忍。static 为 javac 拒绝的垃圾修饰
                //（openjdk NoModifiersOnBinding 负向测试）——容忍跳过
                //（容错优先，输出仍合法）
                let mut anno_prefix = String::new();
                loop {
                    if self.at_kw("final") {
                        self.bump();
                        continue;
                    }
                    if self.at_punct("@") {
                        let s = self.cur_start();
                        self.bump(); // @
                        self.bump(); // 注解名
                        if self.at_punct("(") {
                            self.skip_balanced("(", ")");
                        }
                        anno_prefix.push_str(self.text_of(s, self.cur_start()).trim());
                        anno_prefix.push(' ');
                        continue;
                    }
                    break;
                }
                // 括号化模式：o instanceof (String s)（openjdk Parenthesized）——
                // 整体并入 Ref 原文（含可选绑定/when）
                if self.at_punct("(") {
                    self.skip_balanced("(", ")");
                    if matches!(&self.tok().tok, Tok::Ident(i) if *i != "when" && !is_reserved_after_type(i)) {
                        self.bump();
                    }
                    self.consume_when_guard();
                    let text = self
                        .text_of(pat_start, self.t[self.pos - 1].end)
                        .trim()
                        .to_string();
                    lhs = self.ast.instance_of(lhs, JType::Ref(text), None);
                    continue;
                }
                let mut ty = self.parse_type()?;
                if !anno_prefix.is_empty() {
                    if let JType::Ref(n) = ty {
                        ty = JType::Ref(format!("{anno_prefix}{n}"));
                    }
                }
                if self.at_punct("(") {
                    // instanceof record 模式：x instanceof ColoredPoint(int a,
                    // _, _)（Java 21，checkstyle 抓获）——模式整体并入 Ref 名
                    // 原文保真；后随可选绑定名（…(…) w1——嵌套模式带绑定，
                    // GuardsWithExtraParenthesis 抓获：漏消费残留 Ident →
                    // 风暴）并入 InstanceOf.bind
                    self.skip_balanced("(", ")");
                    // 文本区间在吃绑定**之前**取（吃过再取会把绑定名并入
                    // 模式文本 → 打印时 bind 再打一次 → `… w1 w1` 语法错）
                    let text = self
                        .text_of(pat_start, self.t[self.pos - 1].end)
                        .trim()
                        .to_string();
                    let bind = match &self.tok().tok {
                        Tok::Ident(i) if !is_reserved_after_type(i) => {
                            let b = i.to_string();
                            self.bump();
                            Some(b)
                        }
                        _ => None,
                    };
                    lhs = self.ast.instance_of(lhs, JType::Ref(text), bind.as_deref());
                    continue;
                }
                let bind = match &self.tok().tok {
                    Tok::Ident(i) if !is_reserved_after_type(i) && *i != "when" => {
                        let b = i.to_string();
                        self.bump();
                        Some(b)
                    }
                    _ => None,
                };
                // C 风格绑定维度：instanceof Float a[][]（T8309054）——
                // 维度并入类型（打印 Float[][] a，语义同形）
                let mut dims = 0u32;
                while self.at_punct("[") && self.peek(1).is_punct("]") {
                    self.bump();
                    self.bump();
                    dims += 1;
                }
                let ty = if dims > 0 { wrap_dims(ty, dims) } else { ty };
                lhs = self.ast.instance_of(lhs, ty, bind.as_deref());
                continue;
            }
            if op_prec == 0 || op_prec < min_prec {
                break;
            }
            if right_assoc {
                // 赋值 / 三元
                if self.at_punct("?") {
                    self.bump();
                    let a = self.parse_expr(PREC_ASSIGN)?;
                    self.expect(":");
                    let b = self.parse_expr(PREC_TERNARY)?;
                    lhs = self.ast.ternary(lhs, a, b);
                    continue;
                }
                let op = self.assign_op_take();
                let rhs = self.parse_expr(PREC_ASSIGN)?;
                lhs = match op {
                    Some(o) => self.ast.assign_op(o, lhs, rhs),
                    None => self.ast.assign(lhs, rhs),
                };
                continue;
            }
            let op = self.binop_take();
            let rhs = self.parse_expr(op_prec + 1)?;
            lhs = self.ast.bin(op, lhs, rhs);
        }
        Some(lhs)
    }

    /// 当前 token 是否为二元运算符；返回 (优先级, 是否右结合)。
    fn binop_at(&self) -> (u8, bool) {
        let t = self.tok();
        if let Tok::Punct(p) = &t.tok {
            let prec = match *p {
                "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>="
                | ">>>=" => PREC_ASSIGN,
                "?" => PREC_TERNARY,
                "||" => PREC_OR,
                "&&" => PREC_AND,
                "|" => PREC_BITOR,
                "^" => PREC_BITXOR,
                "&" => PREC_BITAND,
                "==" | "!=" => PREC_EQUALITY,
                "<" | ">" | "<=" | ">=" => PREC_RELATIONAL,
                "<<" | ">>" | ">>>" => PREC_SHIFT,
                "+" | "-" => PREC_ADDITIVE,
                "*" | "/" | "%" => PREC_MULTIPLICATIVE,
                _ => return (0, false),
            };
            let ra = prec == PREC_ASSIGN || prec == PREC_TERNARY;
            return (prec, ra);
        }
        (0, false)
    }

    /// 消费二元运算符（仅非赋值、非三元）。
    fn binop_take(&mut self) -> BinOp {
        use BinOp::*;
        let t = self.bump();
        let s = t.text();
        match s.as_str() {
            "||" => Or,
            "&&" => And,
            "|" => BitOr,
            "^" => BitXor,
            "&" => BitAnd,
            "==" => Eq,
            "!=" => Ne,
            "<" => Lt,
            "<=" => Le,
            ">" => Gt,
            ">=" => Ge,
            "<<" => Shl,
            ">>" => Shr,
            ">>>" => UShr,
            "+" => Add,
            "-" => Sub,
            "*" => Mul,
            "/" => Div,
            "%" => Rem,
            _ => unreachable!("{s}"),
        }
    }

    fn assign_op_take(&mut self) -> Option<BinOp> {
        let t = self.bump();
        match t.text().as_str() {
            "=" => None,
            "+=" => Some(BinOp::Add),
            "-=" => Some(BinOp::Sub),
            "*=" => Some(BinOp::Mul),
            "/=" => Some(BinOp::Div),
            "%=" => Some(BinOp::Rem),
            "&=" => Some(BinOp::BitAnd),
            "|=" => Some(BinOp::BitOr),
            "^=" => Some(BinOp::BitXor),
            "<<=" => Some(BinOp::Shl),
            ">>=" => Some(BinOp::Shr),
            ">>>=" => Some(BinOp::UShr),
            _ => None,
        }
    }

    fn parse_unary(&mut self) -> Option<JavaId> {
        // 表达式位置的前导类型注解（@A int.class——JSR 308 拷问文件）：
        // 跳过后继续（TYPE_USE 无运行时语义）
        if self.at_punct("@") {
            self.bump();
            self.bump();
            if self.at_punct("(") {
                self.skip_balanced("(", ")");
            }
        }
        // 前缀一元
        let t = self.tok().clone();
        let prefix = match &t.tok {
            Tok::Punct("!") => Some(UnOp::Not),
            Tok::Punct("~") => Some(UnOp::BitNot),
            Tok::Punct("-") => Some(UnOp::Neg),
            Tok::Punct("++") => Some(UnOp::PreInc),
            Tok::Punct("--") => Some(UnOp::PreDec),
            Tok::Punct("+") => Some(UnOp::Plus),
            _ => None,
        };
        if let Some(op) = prefix {
            self.bump();
            // 负数字面量：-1 / -1.5 直接折叠为字面量（常量规则的规范化形态）
            if t.is_punct("-") {
                if let Tok::Num(text) = &self.tok().tok {
                    let neg = format!("-{text}");
                    let lit = num_lit(&neg);
                    self.bump();
                    return Some(self.ast.lit(lit));
                }
            }
            let operand = self.parse_unary()?;
            return Some(self.ast.un(op, operand));
        }
        // ( → lambda / cast / 括号（后缀循环仍需处理其后的 .member/[i] 等）
        let mut e = if self.at_punct("(") {
            self.parse_paren_prefixed()?
        } else {
            self.parse_primary()?
        };
        // 后缀循环
        loop {
            // 顶层类型实参 + 方法引用：List<String>::size /
            // List<@A String>::size（PMD FullTypeAnnotations——`<` 分支
            // 曾只在 `.` 之后（x.<T>m() 形态），无点前缀永不进入）
            if self.at_punct("<") {
                let save_lt = self.pos;
                if let Some(ta_text) = self.type_args_raw() {
                    // 可选数组维度（Class<?>[]::new——jdk-sources 拷问）：
                    // 维度并入方法引用名（打印机维度在 :: 前）
                    let mut dims = String::new();
                    while self.at_punct("[") && self.peek(1).is_punct("]") {
                        self.bump();
                        self.bump();
                        dims.push_str("[]");
                    }
                    if self.at_punct("::") {
                        self.bump();
                        // 显式类型实参（Main::<String>new——JLS 15.13）：
                        // 上方链头 type_args_raw 已消费 <TA>（文本在
                        // ta_text——曾丢弃致 GitHubBug309 的 r11 语句
                        // 静默消失/ArrayList::<String>new 丢 TA）。
                        // 并入方法引用名保往返
                        let ta = ta_text.clone();
                        let m = match &self.tok().tok {
                            Tok::Ident(m) => m.to_string(),
                            _ => String::new(),
                        };
                        if !m.is_empty() {
                            self.bump();
                        }
                        // 打印约定：无维度时纯方法名（打印机自打 ::）；带维度
                        // 时 "[]::m"（维度在 :: 前——打印机按 find("::") 切分）
                        let name = if dims.is_empty() {
                            // 无维度：TA 丢弃（`ArrayList<String>::new` →
                            // `ArrayList::new`——类型实参由目标类型推断，
                            // 与 H1 witness 同一权衡；保留曾打印出
                            // `recv::<String>new` 无维度错位形态不可编译）
                            format!("{m}")
                        } else {
                            // `T<?>[]::new`：TA 属于类型侧（维度前）——
                            // 打印器按 find("::") 切分后 recv+前段 =
                            // `T<?>[]`（ES NodeConstruction 抓获：曾拼成
                            // []::<?>new → `T[]::<?>new` 不可编译）
                            format!("{ta}{dims}::{m}")
                        };
                        e = self.ast.method_ref(e, &name);
                        continue;
                    }
                }
                self.pos = save_lt;
                break;
            }
            if self.at_punct(".") {
                self.bump();
                match &self.tok().tok {
                    Tok::Ident(name) if *name == "new" => {
                        // 限定内部类创建 outer.new Inner(...)：表示为
                        // Call{Member{recv, "new Inner"}, args}——打印精确还原
                        // `recv.new Inner(args)`；效果=Call（保守 Unknown）；
                        // 成员名带空格不会与真实标识符冲突，规则不误匹配。
                        self.bump();
                        // 构造器显式类型实参（JLS 15.9：new <TA> Name(…)）
                        // 在类名**之前**（InputUnusedLocalVariableNested
                        // Classes3 抓获：`new <String>InnerInner3` 曾因
                        // 期待 Ident 遇 < 而 break——第二个 .new 丢失，
                        // 输出不可编译）
                        let mut ta_prefix = String::new();
                        if self.at_punct("<") {
                            if let Some(ta) = self.type_args_raw() {
                                ta_prefix = ta;
                            }
                        }
                        let Tok::Ident(cls) = &self.tok().tok else {
                            break;
                        };
                        let mut mname = format!("new {}{cls}", ta_prefix);
                        self.bump();
                        if self.at_punct("<") {
                            if let Some(ta) = self.type_args_raw() {
                                mname.push_str(&ta);
                            }
                        }
                        // 先参数后匿名体（.new Inner(args) { body } 的固定序）
                        let mut args = Vec::new();
                        let mut had_parens = false;
                        if self.at_punct("(") {
                            had_parens = true;
                            args = self.call_args()?;
                        }
                        // 匿名类体 {…}：原文附加（与普通 new 相同）
                        if self.at_punct("{") {
                            if let Some(anon) = self.skip_balanced_braces() {
                                mname.push(' ');
                                mname.push_str(&anon);
                            }
                        }
                        if had_parens {
                            let m = self.ast.member(e, &mname);
                            e = self.ast.call(m, args);
                        } else {
                            e = self.ast.member(e, &mname);
                        }
                        continue;
                    }
                    Tok::Ident(name) => {
                        let n = name.to_string();
                        self.bump();
                        if self.at_punct("(") {
                            let args = self.call_args()?;
                            let m = self.ast.member(e, &n);
                            e = self.ast.call(m, args);
                        } else {
                            e = self.ast.member(e, &n);
                        }
                    }
                    Tok::Punct("<") => {
                        // 显式泛型方法调用 x.<T>name(...)（GJF 常见形态）：
                        // **类型实参加 witness 保留**（并入 callee Member 名——
                        // 推断上下文的 witness 是 javac 必需：PathUtils 抓获
                        // `Stream.<Path>empty().collect(c)` 曾丢 witness 后
                        // CAP#1 推断失败不可编译）；类型引用带类型实参后接 ::
                        //（List<@A String>::size）：并入方法引用名原文
                        let ta_text = self.type_args_raw();
                        if std::env::var("CURE_DBG_LT").is_ok() {
                            eprintln!(
                                "[lt] enter, ta={:?} tok_after={:?}",
                                ta_text,
                                self.tok().tok
                            );
                        }
                        let ta = match ta_text {
                            Some(t) => t,
                            None => break,
                        };
                        if std::env::var("CURE_DBG_LT").is_ok() { eprintln!("[lt] after ta, tok={:?}", self.tok().tok); }
                        if self.at_punct("::") {
                            self.bump();
                            let m = match &self.tok().tok {
                                Tok::Ident(m) => m.to_string(),
                                _ => String::new(),
                            };
                            if !m.is_empty() {
                                self.bump();
                            }
                            let name = format!("{ta}{m}");
                            e = self.ast.method_ref(e, &name);
                            continue;
                        }
                        match &self.tok().tok {
                            Tok::Ident(name) => {
                                // witness 保留：x.<T>m(...) → callee Member
                                // 名拼成 "<T>m"——打印 `x.<T>m(...)`。
                                // witness 是 javac 泛型推断的必需实参
                                //（Stream.<Path>empty().collect(c) 曾丢后
                                // CAP#1 推断失败不可编译——PathUtils 抓获）
                                let n = format!("{ta}{name}");
                                self.bump();
                                if std::env::var("CURE_DBG_LT").is_ok() {
                                    eprintln!("[lt-ident] n={:?}", n);
                                }
                                if self.at_punct("(") {
                                    let args = self.call_args()?;
                                    let m = self.ast.member(e, &n);
                                    e = self.ast.call(m, args);
                                } else {
                                    e = self.ast.member(e, &n);
                                }
                            }
                            _ => break,
                        }
                    }
                    _ => break,
                }
                continue;
            }
            // 数组类型前缀通用形态：T[]（可多维）后跟 ::new（方法引用）或 .class
            //（int[].class / CompilationUnit[][][].class）——维度串吞掉后按后续形态分支。
            if self.at_punct("[") && self.peek(1).is_punct("]") {
                // 越过全部 [] 维度后判定尾巴（一维时旧条件漏多维 .class →
                // 风暴；CompilationUnitBuildersTest 抓获）
                // 维度间的注解（int [] @A [] .class）：前瞻时跳过 @Anno(…)
                let mut p = self.pos + 2;
                loop {
                    if p < self.t.len() && self.t[p].is_punct("@") {
                        p += 2; // @ 名
                        if p < self.t.len() && self.t[p].is_punct("(") {
                            // 平衡越过（计数括号）
                            let mut depth = 0i32;
                            while p < self.t.len() {
                                if self.t[p].is_punct("(") {
                                    depth += 1;
                                } else if self.t[p].is_punct(")") {
                                    depth -= 1;
                                    if depth == 0 {
                                        p += 1;
                                        break;
                                    }
                                }
                                p += 1;
                            }
                        }
                        continue;
                    }
                    if p + 1 < self.t.len() && self.t[p].is_punct("[") && self.t[p + 1].is_punct("]") {
                        p += 2;
                        continue;
                    }
                    break;
                }
                // 可选 <…>（int[]<String>::new——PMD GitHubBug309：数组类型
                // 带类型实参再接 ::，负向拷问形态）——平衡越过（> 计 1/>>
                // 计 2/>>> 计 3，同 type_args_raw 语义）
                if p < self.t.len() && self.t[p].is_punct("<") {
                    let mut depth = 0i32;
                    while p < self.t.len() {
                        if self.t[p].is_punct("<") {
                            depth += 1;
                        } else if self.t[p].is_punct(">") {
                            depth -= 1;
                        } else if self.t[p].is_punct(">>") {
                            depth -= 2;
                        } else if self.t[p].is_punct(">>>") {
                            depth -= 3;
                        }
                        p += 1;
                        if depth <= 0 {
                            break;
                        }
                    }
                }
                let dims_tail = p < self.t.len()
                    && (matches!(&self.t[p].tok, Tok::Punct(c) if *c == "::")
                        || (matches!(&self.t[p].tok, Tok::Punct(c) if *c == ".")
                            && p + 1 < self.t.len()
                            && matches!(&self.t[p + 1].tok, Tok::Ident(i) if *i == "class")));
                if dims_tail {
                    let mut dims = String::new();
                    loop {
                        self.skip_mods_annotations();
                        if self.at_punct("[") && self.peek(1).is_punct("]") {
                            self.bump();
                            self.bump();
                            dims.push_str("[]");
                        } else {
                            break;
                        }
                    }
                    // T[].class：Member{recv=e, name=dims+"class"}（打印 recv[].class）
                    if self.at_punct(".")
                        && matches!(&self.peek(1).tok, Tok::Ident(ref i) if *i == "class")
                    {
                        self.bump(); // .
                        self.bump(); // class
                        let mname = format!("{dims}class");
                        e = self.ast.member(e, &mname);
                        continue;
                    }
                    // 跳过可选类型实参原文（丢弃）
                    if self.at_punct("<") {
                        let _ = self.skip_balanced("<", ">");
                    }
                    let mut name = dims;
                    self.bump(); // ::
                    // 可选显式类型实参（罕见）：T[]::<X>name
                    if self.at_punct("<") {
                        if let Some(ta) = self.type_args_raw() {
                            name.push_str(&ta);
                        }
                    }
                    match &self.tok().tok {
                        Tok::Ident(m) => {
                            let n = m.to_string();
                            self.bump();
                            name.push_str("::");
                            name.push_str(&n);
                            e = self.ast.method_ref(e, &name);
                            continue;
                        }
                        _ => break,
                    }
                }
            }
            if self.at_punct("[") {
                self.bump();
                let idx = self.parse_expr(PREC_ASSIGN)?;
                self.expect("]");
                e = self.ast.index(e, idx);
                continue;
            }
            if self.at_punct("++") {
                self.bump();
                e = self.ast.un(UnOp::PostInc, e);
                continue;
            }
            if self.at_punct("--") {
                self.bump();
                e = self.ast.un(UnOp::PostDec, e);
                continue;
            }
            if self.at_punct("::") {
                self.bump();
                let name = match &self.tok().tok {
                    Tok::Ident(i) => {
                        let n = i.to_string();
                        self.bump();
                        n
                    }
                    Tok::Punct("<") => {
                        // Main::<String>new（JLS 15.13 显式类型实参）：
                        // 文本并入方法引用名保往返（曾丢弃——MR2 抓获）
                        let ta = self.type_args_raw().unwrap_or_default();
                        match &self.tok().tok {
                            Tok::Ident(i) => {
                                let n = format!("{ta}{i}");
                                self.bump();
                                n
                            }
                            _ => break,
                        }
                    }
                    _ => break,
                };
                e = self.ast.method_ref(e, &name);
                continue;
            }
            break;
        }
        Some(e)
    }

    fn parse_paren_prefixed(&mut self) -> Option<JavaId> {
        // 0) 注解-only 括号：(@A) x（JSR 308 负向拷问——无类型的注解 cast）：
        // token 级判定（( 后仅注解直接收尾 ）→ 透明丢弃，解析操作数
        {
            let save_ac = self.pos;
            if self.at_punct("(") && self.peek(1).is_punct("@") {
                self.bump(); // (
                self.skip_mods_annotations();
                if self.at_punct(")") {
                    self.bump();
                    return self.parse_unary();
                }
                self.pos = save_ac;
            }
        }
        // 1) lambda: ( ... ) ->
        // case 标签内禁判：when 守卫 `case R(...) x when (expr) -> body`
        // 的 (expr) -> 恰与 lambda 形态同形（checkstyle GuardsWithExtra
        // Parenthesis 抓获——守卫被吞成 lambda 体 → 标签后残留 → 风暴）
        let save = self.pos;
        if let Some(inner) = self.skip_balanced("(", ")") {
            if self.at_punct("->") && !self.in_case_label {
                self.bump();
                // 去掉 skip_balanced 带回的外层括号
                let params = inner
                    .trim()
                    .strip_prefix('(')
                    .and_then(|t| t.strip_suffix(')'))
                    .unwrap_or(inner.trim())
                    .trim()
                    .to_string();
                let body = if self.at_punct("{") {
                    self.parse_block_raw()
                } else {
                    self.parse_expr(PREC_ASSIGN)?
                };
                return Some(self.ast.lambda(&params, body));
            }
        }
        self.pos = save;
        // 2) cast: ( Type ) unary
        if let Some(ty) = self.try_cast_type() {
            let operand = self.parse_unary()?;
            return Some(self.ast.cast(ty, operand));
        }
        // 3) 括号表达式
        self.bump(); // (
        let e = self.parse_expr(PREC_ASSIGN)?;
        self.expect(")");
        Some(e)
    }

    /// 判定 `( T ) e` 形态的 cast：类型解析成功且后随 token 可开始一元表达式。
    fn try_cast_type(&mut self) -> Option<JType> {
        let save = self.pos;
        self.bump(); // (
        // 类型注解前缀（(@Anno String) x——JSR 308）：parse_type 捕获进
        // Ref 名保真（spoon Castings.java 高频；曾跳过致注解丢失——TU 电池
        // 抓获）
        let mut ty = self.parse_type();
        // cast 数组维度与注解交错：(int @A []) a——注解位于维度间
        //（openjdk LintCast/DotClass 拷问）；维度并入类型
        if let Some(mut t) = ty.clone() {
            let mut dims = 0u32;
            loop {
                self.skip_mods_annotations();
                if self.at_punct("[") && self.peek(1).is_punct("]") {
                    self.bump();
                    self.bump();
                    dims += 1;
                } else {
                    break;
                }
            }
            if dims > 0 {
                t = wrap_dims(t, dims);
                ty = Some(t);
            }
        }
        // 交叉类型 cast：(Runnable & Serializable) lambda——按 JLS 15.16 可多类型
        // 并列；Ref 名内保真原文（打印为原文形态）
        let ty = match ty {
            Some(mut t) => {
                if self.at_punct("&") {
                    let mut names = Vec::new();
                    if let JType::Ref(n) = &t {
                        names.push(n.clone());
                    }
                    while self.eat("&") {
                        if let Some(extra) = self.parse_type() {
                            if let JType::Ref(n) = extra {
                                names.push(n);
                            }
                        } else {
                            break;
                        }
                    }
                    if !names.is_empty() {
                        t = JType::Ref(names.join(" & "));
                    }
                }
                Some(t)
            }
            None => None,
        };
        let ok = match ty {
            Some(ty) => {
                if self.at_punct(")") {
                    let nxt = self.peek(1);
                    // 一元 +/- 仅对**原始类型** cast 放行：(short) -1 是
                    // 无歧义 cast；引用类型的 (a) - b 与减法不可静态分辨
                    //（SwitchCharCast 抓获：case (short) -0x7FFF 曾被解析
                    // 成 short - 0x7FFF 二元减法，输出不可编译）
                    let prim_sign = matches!(ty, JType::Byte | JType::Short | JType::Int | JType::Long | JType::Char | JType::Float | JType::Double)
                        && matches!(nxt.tok, Tok::Punct(p) if p == "-" || p == "+");
                    if starts_unary_operand(nxt) || prim_sign {
                        self.bump(); // )
                        Some(ty)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            None => None,
        };
        if ok.is_none() {
            self.pos = save;
        }
        ok
    }

    fn parse_primary(&mut self) -> Option<JavaId> {
        let t = self.tok().clone();
        match &t.tok {
            Tok::Num(text) => {
                self.bump();
                Some(self.ast.lit(num_lit(text)))
            }
            Tok::Str(s) => {
                self.bump();
                Some(self.ast.lit(Lit::Str(unescape_java_str(&s[1..s.len().saturating_sub(1)]))))
            }
            Tok::Char(c) => {
                self.bump();
                let inner = &c[1..c.len().saturating_sub(1)];
                Some(self.ast.lit(Lit::Char(unescape_java_char(inner))))
            }
            Tok::TextBlock(content) => {
                self.bump();
                Some(self.ast.lit(Lit::TextBlock(content.clone())))
            }
            Tok::Ident(i) => {
                let name = i.to_string();
                match name.as_str() {
                    "true" => {
                        self.bump();
                        return Some(self.ast.lit(Lit::Bool(true)));
                    }
                    "false" => {
                        self.bump();
                        return Some(self.ast.lit(Lit::Bool(false)));
                    }
                    "null" => {
                        self.bump();
                        return Some(self.ast.lit(Lit::Null));
                    }
                    "new" => {
                        self.bump();
                        return self.parse_new();
                    }
                    "switch" => {
                        // switch 表达式：return switch (x) { … }
                        self.bump();
                        return Some(self.parse_switch());
                    }
                    "this" => {
                        self.bump();
                        // this(...) 构造器调用
                        if self.at_punct("(") {
                            let recv = self.ast.this();
                            let args = self.call_args()?;
                            return Some(self.ast.call(recv, args));
                        }
                        return Some(self.ast.this());
                    }
                    "super" => {
                        self.bump();
                        if self.at_punct("(") {
                            let recv = self.ast.super_();
                            let args = self.call_args()?;
                            return Some(self.ast.call(recv, args));
                        }
                        return Some(self.ast.super_());
                    }
                    _ => {}
                }
                self.bump();
                // 类字面量 Foo.class / int.class
                if self.at_punct(".") && self.peek(1).is_ident("class") {
                    self.bump();
                    self.bump();
                    return Some(self.ast.var(&format!("{name}.class")));
                }
                // 标识符 lambda: x -> ...（case 标签内 `A ->` 是 switch 箭头，不是 lambda）
                if self.at_punct("->") && !self.in_case_label {
                    self.bump();
                    let body = if self.at_punct("{") {
                        self.parse_block_raw()
                    } else {
                        self.parse_expr(PREC_ASSIGN)?
                    };
                    return Some(self.ast.lambda(&name, body));
                }
                if self.at_punct("(") {
                    let callee = self.ast.var(&name);
                    let args = self.call_args()?;
                    return Some(self.ast.call(callee, args));
                }
                Some(self.ast.var(&name))
            }
            Tok::Punct("(") => self.parse_paren_prefixed(),
            Tok::Punct("{") => {
                // 裸数组初始化（仅声明处合法，这里兜底）
                self.array_lit()
            }
            Tok::Error(_) => {
                self.err_at("unexpected token");
                None
            }
            Tok::Eof => None,
            _ => {
                self.err_at("unexpected token in expression");
                None
            }
        }
    }

    /// 实参列表。任一参数解析失败 → 返回 None（把失败传播给整条语句，
    /// 让语句级恢复以 Raw 保真），并同步越过本调用的右括号。
    fn call_args(&mut self) -> Option<Vec<JavaId>> {
        let mut args = Vec::new();
        if !self.expect("(") {
            return Some(args);
        }
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > 100_000 || self.at_eof() {
                return None;
            }
            if self.at_punct(")") {
                self.bump();
                break;
            }
            // 试性/罕见形态容忍：
            // - 展开实参 expr...（Java 24 灵活实参草案——openjdk T6967002）
            // - 裸 `?` 通配符实参（同文件负向测试）
            if self.at_punct("?") {
                self.bump();
                args.push(self.ast.raw("?"));
                if !self.eat(",") {
                    self.expect(")");
                    break;
                }
                continue;
            }
            match self.parse_expr(PREC_ASSIGN) {
                Some(a) => {
                    // 展开标记：…（丢弃——草案语法，产出按普通实参）
                    if self.at_punct("...") {
                        self.bump();
                    }
                    args.push(a)
                }
                None => {
                    self.err_at("bad argument");
                    self.sync_call_end();
                    return None;
                }
            }
            if !self.eat(",") {
                if !self.at_punct(")") {
                    // 参数后既非 , 也非 )（如 outer.new Inner 残尾）：
                    // 干净失败 → 上层回退整句 RAW 保真，而非带错继续
                    // （对抗波 2 抓获：残缺参数曾产出可打印但非法的输出）
                    return None;
                }
                self.expect(")");
                break;
            }
        }
        Some(args)
    }

    /// 消费到本调用的收尾 `)`（含），供参数解析失败后清理现场。
    fn sync_call_end(&mut self) {
        loop {
            self.sync_arg();
            if self.eat(",") {
                continue;
            }
            self.eat(")");
            break;
        }
    }

    /// 参数失败恢复：推进到 depth-0 的 `,` 或 `)`（不消费停止 token）。
    fn sync_arg(&mut self) {
        let mut depth = 0i32;
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > STEP_GUARD || self.at_eof() {
                break;
            }
            if depth == 0 && (self.at_punct(",") || self.at_punct(")")) {
                break;
            }
            match &self.tok().tok {
                Tok::Punct("(") | Tok::Punct("[") | Tok::Punct("{") => depth += 1,
                Tok::Punct(")") | Tok::Punct("]") | Tok::Punct("}") => depth -= 1,
                _ => {}
            }
            self.bump();
        }
    }

    /// 声明处的裸数组初始化 `{…}` 归一化为 `new T[…]{…}`（sized=0，无尺寸）：
    /// 裸 `{…}` 只在声明/赋值 RHS 位置合法，被局部传播移到任意表达式位置会
    /// 产出非法源码；包裹后在所有位置合法，打印侧在声明处回退短形态。
    /// 语义不变：`T[] x = {…}` 与 `T[] x = new T[]{…}` 在 JLS 下等价。
    fn wrap_decl_init_array(&mut self, init: Option<JavaId>, ty: &JType) -> Option<JavaId> {
        let init = init?;
        if !matches!(self.ast.data(init), &NodeData::ArrayLit) {
            return Some(init);
        }
        let mut dims = 0u16;
        let mut elem = ty;
        while let JType::Array(inner) = elem {
            elem = inner;
            dims += 1;
        }
        // 元素类型未知（var/推断形态）或非数组类型却给了 {…}：
        // 无法安全包裹，原样保留
        if dims == 0 || matches!(elem, JType::Var) {
            return Some(init);
        }
        Some(self.ast.new_array(elem.clone(), dims, 0, vec![], Some(init)))
    }

    fn array_lit(&mut self) -> Option<JavaId> {
        if !self.expect("{") {
            return None;
        }
        let mut elems = Vec::new();
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > 100_000 || self.at_eof() {
                break;
            }
            if self.at_punct("}") {
                self.bump();
                break;
            }
            if self.at_punct(",") {
                // 尾逗号
                self.bump();
                continue;
            }
            if let Some(e) = self.parse_expr(PREC_ASSIGN) {
                elems.push(e);
            } else {
                break;
            }
        }
        Some(self.ast.array_lit(elems))
    }

    fn parse_new(&mut self) -> Option<JavaId> {
        // new [TypeArguments] Type<...>(args) [anon] | new Type[size…] [init]
        // 构造器显式类型实参在类型名**之前**（JLS 15.9——generic_ctors.java
        // 抓获：`new <String>Object()` 曾解析失败致字段声明拆裂）。TA
        // 并入类型名原文保往返（`<String>Object` 作为 Ref 名打印原样）
        let ta = if self.at_punct("<") {
            self.type_args_raw().unwrap_or_default()
        } else {
            String::new()
        };
        // 类型注解前缀（new @A @B Integer(…)——JLS 15.9 注解可修饰
        // new 的类型；AnnotationsApplied/InputAnnotationOnSameLine 抓获：
        // 注解致字段初始化器解析拆裂）。原文并入类型名保往返
        let anno = if self.at_punct("@") {
            let start = self.cur_start();
            self.skip_mods_annotations();
            let text = self.text_of(start, self.cur_start());
            text.trim().to_string()
        } else {
            String::new()
        };
        let ty = self.parse_type_base()?;
        let mut prefix = format!("{ta}{anno}");
        if !prefix.is_empty() && !prefix.ends_with(' ') {
            prefix.push(' ');
        }
        let ty = if !prefix.is_empty() {
            match ty {
                JType::Ref(n) => JType::Ref(format!("{prefix}{n}")),
                other => other,
            }
        } else {
            ty
        };
        if self.at_punct("(") {
            let args = self.call_args()?;
            let anon_raw = if self.at_punct("{") {
                self.skip_balanced_braces()
            } else {
                None
            };
            if let Some(raw) = anon_raw {
                return Some(self.ast.new_anon(ty, args, raw));
            }
            return Some(self.ast.new_(ty, args));
        }
        if self.at_punct("[") || self.at_punct("@") {
            let mut sizes = Vec::new();
            let mut sized = 0u16;
            let mut dims = 0u16;
            let mut init = None;
            loop {
                // 维度间注解（new int @A [3] / new int [3] @A [4]——JSR 308）：
                // 跳过（与 parse_type 维度注解策略一致）；TU 电池抓获：曾使
                // parse_new 直接失败 → 整语句区域跳过
                self.skip_mods_annotations();
                if !self.at_punct("[") {
                    break;
                }
                self.bump();
                if self.at_punct("]") {
                    self.bump();
                    dims += 1;
                    continue;
                }
                let e = self.parse_expr(PREC_ASSIGN)?;
                self.expect("]");
                sizes.push(e);
                sized += 1;
                dims += 1;
            }
            if self.at_punct("{") {
                init = self.array_lit();
            }
            return Some(self.ast.new_array(ty, dims, sized, sizes, init));
        }
        self.err_at("bad new expression");
        None
    }

    fn take_raw(&mut self, start: usize) -> String {
        let text = self.sync_stmt();
        if text.is_empty() {
            self.text_of(start, self.tok().start)
        } else {
            text
        }
    }
}

fn starts_unary_operand(t: &Token) -> bool {
    match &t.tok {
        // instanceof 不是合法一元操作数——(Integer.MAX_VALUE) instanceof byte
        // 的限定名可解析为类型，instanceof 形似变量 → 误判 cast 吞掉关键字
        //（openjdk PrimitiveInstanceOfNumericValueTests 抓获）
        Tok::Ident(ref i) if *i != "instanceof" => true,
        Tok::Num(_) | Tok::Str(_) | Tok::Char(_) | Tok::TextBlock(_) => true,
        Tok::Punct(p) => matches!(*p, "(" | "!" | "~" | "++" | "--"),
        _ => false,
    }
}

fn is_reserved_after_type(s: &str) -> bool {
    matches!(s, "=" | ";" | ")" | "." | ",")
}

/// 不能作为**类型起始**的 Java 保留字（含字面量关键字）。
/// 上下文关键字（var/record/sealed/yield/permits）不在此列——它们可作类型名。
fn is_type_reserved(s: &str) -> bool {
    matches!(
        s,
        "new" | "return" | "throw" | "break" | "continue" | "if" | "else" | "while" | "do"
            | "for" | "try" | "catch" | "finally" | "switch" | "case" | "default"
            | "instanceof" | "assert" | "synchronized" | "class" | "interface" | "enum"
            | "extends" | "implements" | "import" | "package" | "public" | "private"
            | "protected" | "static" | "abstract" | "native" | "strictfp" | "transient"
            | "volatile" | "final" | "this" | "super" | "throws" | "goto" | "const"
            | "null" | "true" | "false"
    )
}

fn is_primitive_kw(s: &str) -> bool {
    matches!(
        s,
        "byte" | "short" | "int" | "long" | "char" | "float" | "double" | "boolean" | "void"
    )
}

fn is_modifier_kw(s: &str) -> bool {
    matches!(
        s,
        "public" | "private" | "protected" | "static" | "final" | "abstract" | "native"
            | "synchronized" | "strictfp" | "transient" | "volatile" | "default" | "sealed"
    )
}

fn wrap_dims(ty: JType, n: u32) -> JType {
    let mut t = ty;
    for _ in 0..n {
        t = JType::Array(Box::new(t));
    }
    t
}

// ---- 字面量解析 ----

fn num_lit(text: &str) -> Lit {
    let t = text.replace('_', "");
    let lower = t.to_ascii_lowercase();
    // hex/binary 前缀优先判定：0x7F 的 f/d/e/E 是**十六进制数字**——
    // 后缀与浮点判定曾把 hex 误入浮点路径（parse 失败落哨兵
    // Double(0.0)→ 两个不同 hex 字面量被判值等 → ternary_fold 删除
    // 活分支（UnicodeEncoder 字节序/SSL 记录头掩码，SSLEngineInputRecord
    // 抓获：isShort ? 0x7F : 0x3F 曾折成 0x7F）
    let is_radix = lower.starts_with("0x") || lower.starts_with("0b");
    let (core, suffix) = if is_radix {
        if lower.ends_with("l") {
            (&t[..t.len() - 1], "l")
        } else {
            (&t[..], "")
        }
    } else if lower.ends_with("l") {
        (&t[..t.len() - 1], "l")
    } else if lower.ends_with("f") {
        (&t[..t.len() - 1], "f")
    } else if lower.ends_with("d") {
        (&t[..t.len() - 1], "d")
    } else {
        (&t[..], "")
    };
    let is_float_lit = if is_radix {
        // hex 浮点唯一形态：0x1.8p1（含 p）；二进制无浮点字面量
        lower.contains('p')
    } else {
        core.contains('.')
            || core.contains('e')
            || core.contains('E')
            || suffix == "f"
            || suffix == "d"
    };
    if !is_float_lit {
        let radix = if core.starts_with("0x") || core.starts_with("0X") {
            16
        } else if core.starts_with("0b") || core.starts_with("0B") {
            2
        } else if core.len() > 1 && core.starts_with('0') {
            8
        } else {
            10
        };
        let digits: String = match radix {
            16 => core[2..].to_string(),
            2 => core[2..].to_string(),
            _ => core.trim_start_matches('0').to_string(),
        };
        let digits: String = if digits.is_empty() { "0".to_string() } else { digits };
        if suffix == "l" {
            if let Ok(v) = i64::from_str_radix(&digits, radix) {
                return Lit::NumRaw {
                    text: text.into(),
                    val: NumVal::Long(v),
                };
            }
        } else if let Ok(v) = i64::from_str_radix(&digits, radix) {
            return Lit::NumRaw {
                text: text.into(),
                val: NumVal::Int(v),
            };
        }
    }
    // 浮点
    let mut core2 = core.to_string();
    if (core2.starts_with("0x") || core2.starts_with("0X")) && !core2.contains('p') {
        core2.push('p');
        core2.push('0');
    }
    if suffix == "f" {
        if let Ok(v) = core2.parse::<f32>() {
            return Lit::NumRaw {
                text: text.into(),
                val: NumVal::Float(v as f64),
            };
        }
    }
    if let Ok(v) = core2.parse::<f64>() {
        return Lit::NumRaw {
            text: text.into(),
            val: NumVal::Double(v),
        };
    }
    Lit::NumRaw {
        text: text.into(),
        val: NumVal::Double(0.0),
    }
}

/// 解析 Java 字符串转义（输入不含外层引号）。
fn unescape_java_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('b') => out.push('\u{8}'),
            Some('r') => out.push('\r'),
            Some('f') => out.push('\u{c}'),
            Some('s') => out.push(' '),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('\\') => out.push('\\'),
            // 八进制转义（JLS 3.10.6/3.10.7）：\0..\377。
            // \1 之前落到 Some(other) 双字符臂（'\'+'1'）→
            // unescape_java_char 取首字符 92——差分抓获：'\1' 值
            // 1 被解码成反斜杠
            Some(d @ ('0'..='3')) => {
                let mut v = d.to_digit(8).unwrap();
                for _ in 0..2 {
                    match chars.clone().next() {
                        Some(d2 @ ('0'..='7')) => {
                            v = v * 8 + d2.to_digit(8).unwrap();
                            chars.next();
                        }
                        _ => break,
                    }
                }
                if let Some(c) = char::from_u32(v) {
                    out.push(c);
                }
            }
            Some(d @ ('4'..='7')) => {
                let mut v = d.to_digit(8).unwrap();
                if let Some(d2 @ ('0'..='7')) = chars.clone().next() {
                    v = v * 8 + d2.to_digit(8).unwrap();
                    chars.next();
                }
                if let Some(c) = char::from_u32(v) {
                    out.push(c);
                }
            }
            Some('u') => {
                let hex: String = chars.by_ref().take(4).collect();
                if let Ok(v) = u32::from_str_radix(&hex, 16) {
                    if let Some(c) = char::from_u32(v) {
                        out.push(c);
                    }
                }
            }
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

fn unescape_java_char(s: &str) -> char {
    let un = unescape_java_str(s);
    un.chars().next().unwrap_or('\0')
}

// ---- 优先级常量 ----
const PREC_ASSIGN: u8 = 2;
const PREC_TERNARY: u8 = 3;
const PREC_OR: u8 = 4;
const PREC_AND: u8 = 5;
const PREC_BITOR: u8 = 6;
const PREC_BITXOR: u8 = 7;
const PREC_BITAND: u8 = 8;
const PREC_EQUALITY: u8 = 9;
const PREC_RELATIONAL: u8 = 10;
const PREC_SHIFT: u8 = 11;
const PREC_ADDITIVE: u8 = 12;
const PREC_MULTIPLICATIVE: u8 = 13;
