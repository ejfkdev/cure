//! # cure-ts-parser
//!
//! TypeScript/JavaScript 容错解析器：lexer（[`lexer`]）+ 递归下降语法。
//!
//! 策略：
//! - **JS 值级语法**完整建模进 [`cure_ts_ast`]（引擎可简化）；
//! - **TS 类型/装饰器**以原文区间捕获（ty_raw/sig_raw——保真打印）；
//! - **JSX / 类表达式**整体原文捕获（`Raw` 表达式节点——不透明但保真）；
//! - **容错**：语句级失败回退为原文 `Raw` 节点（同步点：`;`/换行边界/
//!   `}`/EOF），解析错误永不中止整个文件——与 Java 后端同契约。
//! - **ASI**：`newline_before` 驱动的隐式分号 + restricted production
//!   （return/throw/break/continue/后缀 ++--）。

mod lexer;

use cure_engine::kind::{BinOp, UnOp};
use cure_ts_ast::{
    ClassDecl, ClassMember, JsLit, ModuleUnit, NodeData, NumVal, ParseError, TopItem,
    TsAst, TsId, VarKind,
};
pub use lexer::{LexError, Lexer, TokKind, Token};

pub struct ParseOutcome {
    pub ast: TsAst,
    pub unit: ModuleUnit,
    pub errors: Vec<ParseError>,
}

/// 解析源文件（容错）。
pub fn parse(src: &str) -> ParseOutcome {
    let mut shebang = String::new();
    let body = if let Some(rest) = src.strip_prefix("#!") {
        if let Some(nl) = rest.find('\n') {
            shebang = src[..2 + nl].to_string();
            &rest[nl + 1..]
        } else {
            shebang = src.to_string();
            ""
        }
    } else {
        src
    };
    let (toks, lex_errs) = Lexer::new(body).tokenize();
    let mut p = Parser {
        src: body,
        toks,
        pos: 0,
        ast: TsAst::new(),
        unit: ModuleUnit { shebang, items: Vec::new() },
        errors: Vec::new(),
    };
    for e in lex_errs {
        p.errors.push(ParseError {
            line: e.line,
            col: e.col,
            message: e.message,
        });
    }
    p.parse_module();
    ParseOutcome {
        ast: p.ast,
        unit: p.unit,
        errors: p.errors,
    }
}

struct Parser<'a> {
    src: &'a str,
    toks: Vec<Token>,
    pos: usize,
    ast: TsAst,
    unit: ModuleUnit,
    errors: Vec<ParseError>,
}

/// 二元运算符表：`(优先级, BinOp)`。`??` 与 `||` 同层（混合需括号——
/// 解析器宽容接受，引擎/打印保真）。
fn bin_info(op: &str) -> Option<(u8, BinOp)> {
    use BinOp::*;
    Some(match op {
        "??" => (2, Nullish),
        "||" => (2, Or),
        "&&" => (3, And),
        "|" => (4, BitOr),
        "^" => (5, BitXor),
        "&" => (6, BitAnd),
        "==" => (7, Eq),
        "!=" => (7, Ne),
        "===" => (7, StrictEq),
        "!==" => (7, StrictNe),
        "<" => (8, Lt),
        ">" => (8, Gt),
        "<=" => (8, Le),
        ">=" => (8, Ge),
        "in" => (8, In),
        "instanceof" => (8, Instanceof),
        "<<" => (9, Shl),
        ">>" => (9, Shr),
        ">>>" => (9, UShr),
        "+" => (10, Add),
        "-" => (10, Sub),
        "*" => (11, Mul),
        "/" => (11, Div),
        "%" => (11, Rem),
        "**" => (12, Pow),
        _ => return None,
    })
}

fn assign_op_of(punct: &str) -> Option<BinOp> {
    use BinOp::*;
    Some(match punct {
        "+=" => Add,
        "-=" => Sub,
        "*=" => Mul,
        "/=" => Div,
        "%=" => Rem,
        "**=" => Pow,
        "<<=" => Shl,
        ">>=" => Shr,
        ">>>=" => UShr,
        "&=" => BitAnd,
        "|=" => BitOr,
        "^=" => BitXor,
        "&&=" => And,
        "||=" => Or,
        "??=" => Nullish,
        _ => return None,
    })
}

const STMT_KW: &[&str] = &[
    "var", "let", "const", "if", "else", "while", "do", "for", "return", "break", "continue",
    "throw", "try", "switch", "function", "class", "import", "export", "with", "debugger",
    "using", "enum", "interface", "namespace", "declare", "type", "abstract",
];

impl<'a> Parser<'a> {
    // ---- token 工具 ----

    fn tk(&self) -> &Token {
        &self.toks[self.pos.min(self.toks.len() - 1)]
    }
    fn tk_at(&self, off: usize) -> &Token {
        &self.toks[(self.pos + off).min(self.toks.len() - 1)]
    }
    fn is_punct(&self, p: &str) -> bool {
        self.tk().kind == TokKind::Punct && self.tk().text == p
    }
    fn is_punct_at(&self, off: usize, p: &str) -> bool {
        let t = self.tk_at(off);
        t.kind == TokKind::Punct && t.text == p
    }
    fn is_kw(&self, w: &str) -> bool {
        self.tk().kind == TokKind::Ident && self.tk().text == w
    }
    fn text(&self) -> &str {
        &self.tk().text
    }
    fn at_eof(&self) -> bool {
        self.tk().kind == TokKind::Eof
    }
    fn advance(&mut self) -> Token {
        let t = self.toks[self.pos.min(self.toks.len() - 1)].clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }
    fn err_here(&mut self, msg: &str) {
        let t = self.tk().clone();
        self.errors.push(ParseError {
            line: t.line,
            col: t.col,
            message: msg.to_string(),
        });
    }
    /// token 下标区间 [a, b) 的源码原文。b ≤ a（空区间）→ 空串
    ///（trivia 使 toks[b-1].end < toks[a].start——直接切片会越界）。
    fn raw_between(&self, a: usize, b: usize) -> String {
        if b <= a || a >= self.toks.len() {
            return String::new();
        }
        let s = self.toks[a].start;
        let e = if b <= self.toks.len() {
            self.toks[b - 1].end
        } else {
            self.src.len()
        };
        if e <= s {
            return String::new();
        }
        self.src[s.min(self.src.len())..e.min(self.src.len())].to_string()
    }

    // ---- 模块层 ----

    fn parse_module(&mut self) {
        while !self.at_eof() {
            let before = self.pos;
            self.parse_top_item();
            if self.pos == before {
                self.advance();
            }
        }
    }

    fn parse_top_item(&mut self) {
        // 装饰器前缀（捕获进头部原文）
        let mut deco_raw = String::new();
        while self.is_punct("@") {
            let start = self.pos;
            self.skip_decorator();
            // 装饰器后的实体在同一「语句」——扫到边界会吃过头；
            // 只跳装饰器本身
            deco_raw.push_str(&self.raw_between(start, self.pos));
            deco_raw.push(' ');
        }
        if self.is_kw("import") {
            let start = self.pos;
            self.skip_balanced_to_stmt_boundary();
            let raw = self.raw_between(start, self.pos);
            self.unit.items.push(TopItem::Import(raw));
            return;
        }
        if self.is_kw("export") {
            self.parse_export(deco_raw);
            return;
        }
        if self.is_kw("declare") || self.is_kw("namespace") || self.is_kw("interface") || self.is_kw("module") {
            let start = self.pos;
            self.skip_balanced_to_stmt_boundary();
            let raw = self.raw_between(start, self.pos);
            self.unit.items.push(TopItem::Raw(raw));
            return;
        }
        if self.is_kw("type")
            && self.tk_at(1).kind == TokKind::Ident
            && self.is_punct_at(1, "=")
            && !self.tk_at(1).newline_before
        {
            let start = self.pos;
            self.skip_balanced_to_stmt_boundary();
            let raw = self.raw_between(start, self.pos);
            self.unit.items.push(TopItem::Raw(raw));
            return;
        }
        if self.is_kw("enum") || (self.is_kw("const") && self.tk_at(1).text == "enum") {
            let start = self.pos;
            self.skip_balanced_to_stmt_boundary();
            let raw = self.raw_between(start, self.pos);
            self.unit.items.push(TopItem::Raw(raw));
            return;
        }
        if let Some(f) = self.try_parse_function_decl(deco_raw, false) {
            self.unit.items.push(f);
            return;
        }
        if self.is_kw("class") || (self.is_kw("abstract") && self.tk_at(1).text == "class") {
            if let Some(c) = self.try_parse_class_decl(String::new()) {
                self.unit.items.push(TopItem::Class(c));
                return;
            }
        }
        let s = self.parse_stmt();
        self.unit.items.push(TopItem::Stmt(s));
    }

    fn parse_export(&mut self, deco_raw: String) {
        let kw_pos = self.pos;
        self.advance(); // export
        if self.is_kw("default") {
            self.advance();
            if let Some(f) = self.try_parse_function_decl(String::new(), true) {
                self.unit.items.push(f);
                return;
            }
            if let Some(c) = self.try_parse_class_decl(String::new()) {
                self.unit.items.push(TopItem::Class(c));
                return;
            }
            let e = self.parse_assign_expr();
            self.consume_semi();
            let _ = e;
            let raw = format!("export default {}", self.raw_between(kw_pos + 2, self.pos));
            self.unit.items.push(TopItem::ExportRaw(raw));
            return;
        }
        if self.is_kw("function")
            || (self.is_kw("async") && self.tk_at(1).text == "function")
        {
            if let Some(f) = self.try_parse_function_decl(deco_raw.clone(), true) {
                self.unit.items.push(f);
                return;
            }
        }
        if self.is_kw("class") || (self.is_kw("abstract") && self.tk_at(1).text == "class") {
            if let Some(c) = self.try_parse_class_decl(deco_raw) {
                self.unit.items.push(TopItem::Class(c));
                return;
            }
        }
        let start = kw_pos;
        self.skip_balanced_to_stmt_boundary();
        let raw = self.raw_between(start, self.pos);
        self.unit.items.push(TopItem::ExportRaw(raw));
    }

    /// 平衡扫描到语句边界（`;`/同层 `}`/EOF；深度感知）。
    fn skip_balanced_to_stmt_boundary(&mut self) {
        let mut paren = 0i32;
        let mut brace = 0i32;
        let mut brack = 0i32;
        loop {
            if self.at_eof() {
                return;
            }
            let t = self.tk();
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "(" => paren += 1,
                    ")" => paren -= 1,
                    "[" => brack += 1,
                    "]" => brack -= 1,
                    "{" => brace += 1,
                    "}" => {
                        if brace == 0 && paren == 0 && brack == 0 {
                            return;
                        }
                        brace -= 1;
                    }
                    ";" if brace == 0 && paren == 0 && brack == 0 => {
                        self.advance();
                        return;
                    }
                    _ => {}
                }
            }
            if brace == 0 && paren == 0 && brack == 0 && self.tk_at(1).newline_before {
                let nt = self.tk_at(1).clone();
                let ends_stmt = match nt.kind {
                    TokKind::Eof => true,
                    TokKind::Punct => matches!(nt.text.as_str(), "}" | ")" | "]" | ";"),
                    TokKind::Ident => STMT_KW.contains(&nt.text.as_str()) && nt.text != "let",
                    _ => false,
                };
                if ends_stmt {
                    self.advance();
                    return;
                }
            }
            self.advance();
        }
    }

    // ---- 函数/类声明 ----

    fn try_parse_function_decl(&mut self, prefix_raw: String, exported: bool) -> Option<TopItem> {
        let start = self.pos;
        let mut is_async = false;
        if self.is_kw("async") && self.tk_at(1).text == "function" && !self.tk_at(1).newline_before {
            is_async = true;
            self.advance();
        }
        if !self.is_kw("function") {
            self.pos = start;
            return None;
        }
        self.advance();
        let mut is_generator = false;
        if self.is_punct("*") {
            is_generator = true;
            self.advance();
        }
        let name = if self.tk().kind == TokKind::Ident {
            self.advance().text
        } else {
            String::new()
        };
        if !self.is_punct("(") {
            self.pos = start;
            return None;
        }
        let ps = self.pos;
        self.skip_parens();
        let params_full = self.raw_between(ps, self.pos);
        let mut ret_raw = String::new();
        if self.is_punct(":") {
            let rs = self.pos;
            while !self.at_eof() && !self.is_punct("{") && !self.is_punct(";") {
                self.advance();
            }
            ret_raw = self.raw_between(rs, self.pos);
        }
        let body = if self.is_punct("{") {
            Some(self.parse_block())
        } else {
            // declare/abstract 无体形态——原文回退
            if self.is_punct(";") {
                self.advance();
            }
            None
        };
        let mut sig = String::new();
        if !prefix_raw.is_empty() {
            sig.push_str(&prefix_raw);
            sig.push(' ');
        }
        if exported {
            sig.push_str("export ");
        }
        if is_async {
            sig.push_str("async ");
        }
        sig.push_str("function");
        if is_generator {
            sig.push('*');
        }
        if !name.is_empty() {
            sig.push(' ');
            sig.push_str(&name);
        }
        sig.push_str(&params_full);
        sig.push_str(&ret_raw);
        Some(TopItem::Func { sig_raw: sig, name, body })
    }

    fn try_parse_class_decl(&mut self, deco_raw: String) -> Option<ClassDecl> {
        let start = self.pos;
        let mut head = String::new();
        if !deco_raw.is_empty() {
            head = deco_raw;
            head.push(' ');
        }
        let mut kind_kw = String::new();
        if self.is_kw("abstract") && self.tk_at(1).text == "class" {
            head.push_str("abstract ");
            self.advance();
        }
        if self.is_kw("class") {
            kind_kw = "class".into();
            self.advance();
        } else {
            self.pos = start;
            return None;
        }
        let name = if self.tk().kind == TokKind::Ident {
            self.advance().text
        } else {
            String::new()
        };
        let hs = self.pos;
        while !self.at_eof() && !self.is_punct("{") && !self.is_punct(";") {
            self.advance();
        }
        head.push_str(&self.raw_between(hs, self.pos));
        if !self.is_punct("{") {
            self.pos = start;
            return None;
        }
        self.advance();
        let mut members = Vec::new();
        while !self.at_eof() && !self.is_punct("}") {
            let before = self.pos;
            if let Some(m) = self.parse_class_member() {
                members.push(m);
            }
            if self.pos == before {
                self.advance();
            }
        }
        if self.is_punct("}") {
            self.advance();
        }
        Some(ClassDecl { head_raw: head, kind_kw, name, members })
    }

    fn parse_class_member(&mut self) -> Option<ClassMember> {
        let start = self.pos;
        // static 块
        if self.is_kw("static") && self.is_punct_at(1, "{") && !self.tk_at(1).newline_before {
            self.advance();
            let body = self.parse_block();
            return Some(ClassMember::StaticBlock { body });
        }
        if self.is_punct("{") {
            let body = self.parse_block();
            return Some(ClassMember::StaticBlock { body });
        }
        while self.is_punct("@") {
            self.skip_decorator();
        }
        loop {
            let t = self.tk().clone();
            if t.kind == TokKind::Ident
                && matches!(
                    t.text.as_str(),
                    "static" | "public" | "private" | "protected" | "readonly" | "abstract"
                        | "override" | "declare" | "async" | "get" | "set" | "const" | "accessor"
                )
                && self.tk_at(1).kind == TokKind::Ident
                && !self.tk_at(1).newline_before
            {
                self.advance();
            } else {
                break;
            }
        }
        if self.is_punct("*") {
            self.advance();
        }
        let name_tok = self.tk().clone();
        let name = if name_tok.kind == TokKind::Ident {
            self.advance().text
        } else if name_tok.kind == TokKind::Punct && name_tok.text == "#" {
            self.advance();
            let mut n = String::from("#");
            if self.tk().kind == TokKind::Ident {
                n.push_str(&self.advance().text);
            }
            n
        } else {
            let s = self.pos;
            self.skip_balanced_to_stmt_boundary();
            return Some(ClassMember::Raw(self.raw_between(s, self.pos)));
        };
        // 泛型参数
        if self.is_punct("<") {
            self.skip_angle_balanced();
        }
        // 方法
        if self.is_punct("(") && !self.tk().newline_before {
            self.skip_parens();
            if self.is_punct(":") {
                while !self.at_eof() && !self.is_punct("{") && !self.is_punct(";") {
                    self.advance();
                }
            }
            // sig 捕获到 `{` 之前（body 是 arena 节点——含进去会双重打印；
            // 幂等差分抓获：constructor 曾打印体两遍）
            let sig_end = self.pos;
            let body = if self.is_punct("{") {
                Some(self.parse_block())
            } else {
                self.consume_semi();
                None
            };
            let sig_raw = self.raw_between(start, sig_end);
            return Some(ClassMember::Method { sig_raw, body });
        }
        // 字段：[: ty] [= init] ;
        if self.is_punct(":") && !self.tk().newline_before {
            while !self.at_eof() && !self.is_punct("=") && !self.is_punct(";") {
                if self.is_punct("{") {
                    break;
                }
                self.advance();
            }
        }
        let init = if self.is_punct("=") && !self.tk().newline_before {
            self.advance();
            Some(self.parse_assign_expr())
        } else {
            None
        };
        // sig 到 `=` 之前（init 是 arena 节点——打印不重复）
        let sig_end = if init.is_some() {
            // `=` token 的下标 = 当前 pos 之前回退一个（init 表达式可能
            // 多 token）——记录等号位置
            self.eq_pos_back(start)
        } else {
            self.pos
        };
        self.consume_semi();
        let sig_raw = self.raw_between(start, sig_end);
        Some(ClassMember::Field { sig_raw, name, init })
    }

    /// 在 [start, pos) 找最后一个顶层 `=` 的下标（字段 sig/init 分界）。
    fn eq_pos_back(&self, start: usize) -> usize {
        let mut eq = self.pos;
        for i in (start..self.pos).rev() {
            if self.toks[i].kind == TokKind::Punct && self.toks[i].text == "=" {
                eq = i;
                break;
            }
        }
        eq
    }

    fn skip_decorator(&mut self) {
        self.advance(); // @
        while self.tk().kind == TokKind::Ident || self.is_punct(".") {
            self.advance();
        }
        if self.is_punct("(") {
            self.skip_parens();
        }
    }

    fn skip_parens(&mut self) {
        if !self.is_punct("(") {
            return;
        }
        let mut depth = 0i32;
        loop {
            if self.at_eof() {
                return;
            }
            if self.is_punct("(") {
                depth += 1;
            } else if self.is_punct(")") {
                depth -= 1;
                if depth == 0 {
                    self.advance();
                    return;
                }
            }
            self.advance();
        }
    }

    fn skip_angle_balanced(&mut self) {
        if !self.is_punct("<") {
            return;
        }
        let mut depth = 0i32;
        loop {
            if self.at_eof() {
                return;
            }
            if self.is_punct("<") {
                depth += 1;
            } else if self.is_punct(">") {
                depth -= 1;
                if depth == 0 {
                    self.advance();
                    return;
                }
            } else if self.is_punct(">>") {
                depth -= 2;
                if depth <= 0 {
                    self.advance();
                    return;
                }
            } else if self.is_punct(">>>") {
                depth -= 3;
                if depth <= 0 {
                    self.advance();
                    return;
                }
            }
            self.advance();
        }
    }

    // ---- 语句 ----

    fn parse_block(&mut self) -> TsId {
        if !self.is_punct("{") {
            self.err_here("expected {");
            return self.ast.block(vec![]);
        }
        self.advance();
        let mut stmts = Vec::new();
        while !self.at_eof() && !self.is_punct("}") {
            let before = self.pos;
            let s = self.parse_stmt();
            stmts.push(s);
            if self.pos == before {
                self.advance();
            }
        }
        if self.is_punct("}") {
            self.advance();
        } else {
            self.err_here("expected }");
        }
        self.ast.block(stmts)
    }

    /// 消费分号（含 ASI：`}`/EOF/换行前）。
    fn consume_semi(&mut self) {
        if self.is_punct(";") {
            self.advance();
            return;
        }
        if self.is_punct("}") || self.at_eof() || self.tk().newline_before {
            return;
        }
        self.err_here("expected ;");
    }

    fn parse_stmt(&mut self) -> TsId {
        let t = self.tk().clone();
        match t.kind {
            TokKind::Punct => match t.text.as_str() {
                "{" => self.parse_block(),
                ";" => {
                    self.advance();
                    self.ast.empty()
                }
                "@" => {
                    let s = self.pos;
                    self.skip_balanced_to_stmt_boundary();
                    self.ast.raw(&self.raw_between(s, self.pos))
                }
                _ => self.parse_expr_or_decl_stmt(),
            },
            TokKind::Ident => match t.text.as_str() {
                "var" | "const" => self.parse_var_decl_stmt(),
                "let" if self.let_is_decl() => self.parse_var_decl_stmt(),
                "using" if self.using_is_decl() => self.parse_var_decl_stmt(),
                "if" => self.parse_if(),
                "while" => self.parse_while(),
                "do" => self.parse_do_while(),
                "for" => self.parse_for(),
                "return" => self.parse_return(),
                "break" | "continue" => self.parse_break_continue(),
                "throw" => self.parse_throw(),
                "try" => self.parse_try(),
                "switch" => self.parse_switch(),
                "function" | "async" if self.async_function_ahead() => {
                    match self.try_parse_function_stmt() {
                        Some(s) => s,
                        None => self.parse_expr_or_decl_stmt(),
                    }
                }
                "class" | "abstract" if self.tk_at(1).kind == TokKind::Ident => {
                    let s = self.pos;
                    self.skip_class_expr_stmt();
                    self.ast.raw(&self.raw_between(s, self.pos))
                }
                "debugger" => {
                    self.advance();
                    self.consume_semi();
                    self.ast.raw("debugger;")
                }
                "with" => {
                    let s = self.pos;
                    self.skip_balanced_to_stmt_boundary();
                    self.ast.raw(&self.raw_between(s, self.pos))
                }
                "enum" | "interface" | "namespace" | "module" => {
                    let s = self.pos;
                    self.skip_balanced_to_stmt_boundary();
                    self.ast.raw(&self.raw_between(s, self.pos))
                }
                "type"
                    if self.tk_at(1).kind == TokKind::Ident
                        && self.is_punct_at(1, "=")
                        && !self.tk_at(1).newline_before =>
                {
                    let s = self.pos;
                    self.skip_balanced_to_stmt_boundary();
                    self.ast.raw(&self.raw_between(s, self.pos))
                }
                _ => self.parse_expr_or_decl_stmt(),
            },
            _ => self.parse_expr_or_decl_stmt(),
        }
    }

    /// `let` 是声明还是标识符。
    fn let_is_decl(&self) -> bool {
        let n = self.tk_at(1);
        if n.newline_before {
            return false;
        }
        matches!(n.kind, TokKind::Ident)
            || (n.kind == TokKind::Punct && matches!(n.text.as_str(), "[" | "{"))
    }

    fn using_is_decl(&self) -> bool {
        let n = self.tk_at(1);
        !n.newline_before && n.kind == TokKind::Ident
    }

    fn async_function_ahead(&self) -> bool {
        if self.is_kw("function") {
            return true;
        }
        self.is_kw("async") && self.tk_at(1).text == "function" && !self.tk_at(1).newline_before
    }

    /// 嵌套函数声明语句 → FuncStmt（Custom——引擎不触碰，体进 arena）。
    fn try_parse_function_stmt(&mut self) -> Option<TsId> {
        let start = self.pos;
        let mut is_async = false;
        if self.is_kw("async") && self.tk_at(1).text == "function" {
            is_async = true;
            self.advance();
        }
        if !self.is_kw("function") {
            self.pos = start;
            return None;
        }
        self.advance();
        let mut is_generator = false;
        if self.is_punct("*") {
            is_generator = true;
            self.advance();
        }
        let name = if self.tk().kind == TokKind::Ident {
            self.advance().text
        } else {
            String::new()
        };
        if !self.is_punct("(") {
            self.pos = start;
            return None;
        }
        let ps = self.pos;
        self.skip_parens();
        let params_full = self.raw_between(ps, self.pos);
        let params_inner = params_full
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(&params_full)
            .to_string();
        let mut ret_raw = String::new();
        if self.is_punct(":") {
            let rs = self.pos;
            while !self.at_eof() && !self.is_punct("{") && !self.is_punct(";") {
                self.advance();
            }
            ret_raw = self.raw_between(rs, self.pos);
        }
        let body = if self.is_punct("{") {
            self.parse_block()
        } else {
            self.pos = start;
            return None;
        };
        let mut sig = String::new();
        if is_async {
            sig.push_str("async ");
        }
        sig.push_str("function");
        if is_generator {
            sig.push('*');
        }
        if !name.is_empty() {
            sig.push(' ');
            sig.push_str(&name);
        }
        sig.push_str(&params_full);
        sig.push_str(&ret_raw);
        Some(self.ast.func_stmt(&sig, &name, &params_inner, body))
    }

    /// 嵌套类声明语句：扫 `class … { … }` 平衡。
    fn skip_class_expr_stmt(&mut self) {
        self.skip_class_expr();
        self.consume_semi();
    }

    fn parse_var_decl_stmt(&mut self) -> TsId {
        let kw = self.advance().text;
        let kind = match kw.as_str() {
            "var" => VarKind::Var,
            "let" => VarKind::Let,
            "using" => VarKind::Const, // using ≈ const（显式释放语义原文保真）
            _ => VarKind::Const,
        };
        let kw_for_print = kw.clone();
        let mut stmts: Vec<TsId> = Vec::new();
        loop {
            if self.is_punct("{") || self.is_punct("[") {
                // 解构声明
                let ps = self.pos;
                if self.is_punct("{") {
                    self.skip_braces();
                } else {
                    self.skip_brackets();
                }
                let mut pattern = self.raw_between(ps, self.pos);
                if self.is_punct(":") && !self.tk().newline_before {
                    let ts = self.pos;
                    while !self.at_eof()
                        && !self.is_punct("=")
                        && !self.is_punct(";")
                        && !self.is_punct(",")
                    {
                        self.advance();
                    }
                    pattern.push_str(&self.raw_between(ts, self.pos));
                }
                if self.is_punct("=") && !self.tk().newline_before {
                    self.advance();
                    let value = self.parse_assign_expr();
                    stmts.push(self.ast.destruct_stmt(&kw_for_print, &pattern, value));
                } else {
                    self.err_here("destructuring declaration without initializer");
                    stmts.push(self.ast.raw(&format!("{kw_for_print} {pattern}")));
                }
            } else if self.tk().kind == TokKind::Ident {
                let name = self.advance().text;
                let mut ty_raw: Option<String> = None;
                if self.is_punct(":") && !self.tk().newline_before {
                    let ts = self.pos;
                    while !self.at_eof()
                        && !self.is_punct("=")
                        && !self.is_punct(";")
                        && !self.is_punct(",")
                    {
                        if self.is_punct("{") || self.is_punct("[") {
                            break;
                        }
                        self.advance();
                    }
                    ty_raw = Some(self.raw_between(ts, self.pos));
                }
                let init = if self.is_punct("=") && !self.tk().newline_before {
                    self.advance();
                    Some(self.parse_assign_expr())
                } else {
                    None
                };
                stmts.push(self.ast.var_decl(&name, kind, ty_raw.as_deref(), init));
            } else {
                self.err_here("expected declarator");
                break;
            }
            if self.is_punct(",") {
                self.advance();
                continue;
            }
            break;
        }
        self.consume_semi();
        if stmts.len() == 1 {
            stmts[0]
        } else {
            self.ast.group(stmts)
        }
    }

    fn skip_braces(&mut self) {
        if !self.is_punct("{") {
            return;
        }
        let mut depth = 0i32;
        loop {
            if self.at_eof() {
                return;
            }
            if self.is_punct("{") {
                depth += 1;
            } else if self.is_punct("}") {
                depth -= 1;
                if depth == 0 {
                    self.advance();
                    return;
                }
            }
            self.advance();
        }
    }

    fn skip_brackets(&mut self) {
        if !self.is_punct("[") {
            return;
        }
        let mut depth = 0i32;
        loop {
            if self.at_eof() {
                return;
            }
            if self.is_punct("[") {
                depth += 1;
            } else if self.is_punct("]") {
                depth -= 1;
                if depth == 0 {
                    self.advance();
                    return;
                }
            }
            self.advance();
        }
    }

    fn parse_if(&mut self) -> TsId {
        self.advance();
        if !self.is_punct("(") {
            self.err_here("expected ( after if");
            let s = self.pos;
            self.skip_balanced_to_stmt_boundary();
            return self.ast.raw(&self.raw_between(s, self.pos));
        }
        self.advance();
        let cond = self.parse_expr();
        if self.is_punct(")") {
            self.advance();
        } else {
            self.err_here("expected )");
        }
        let then = self.parse_stmt_or_group();
        let els = if self.is_kw("else") {
            self.advance();
            Some(self.parse_stmt_or_group())
        } else {
            None
        };
        self.ast.if_(cond, then, els)
    }

    fn parse_stmt_or_group(&mut self) -> TsId {
        if self.is_punct("{") {
            self.parse_block()
        } else {
            self.parse_stmt()
        }
    }

    fn parse_while(&mut self) -> TsId {
        self.advance();
        if !self.is_punct("(") {
            self.err_here("expected ( after while");
            let s = self.pos;
            self.skip_balanced_to_stmt_boundary();
            return self.ast.raw(&self.raw_between(s, self.pos));
        }
        self.advance();
        let cond = self.parse_expr();
        if self.is_punct(")") {
            self.advance();
        } else {
            self.err_here("expected )");
        }
        let body = self.parse_stmt_or_group();
        self.ast.while_(cond, body)
    }

    fn parse_do_while(&mut self) -> TsId {
        self.advance();
        let body = self.parse_stmt_or_group();
        if self.is_kw("while") {
            self.advance();
            if self.is_punct("(") {
                self.advance();
                let cond = self.parse_expr();
                if self.is_punct(")") {
                    self.advance();
                } else {
                    self.err_here("expected )");
                }
                self.consume_semi();
                return self.ast.do_while(body, cond);
            }
        }
        self.err_here("malformed do-while");
        self.ast.empty()
    }

    fn parse_for(&mut self) -> TsId {
        self.advance();
        let is_await = if self.is_kw("await") {
            self.advance();
            true
        } else {
            false
        };
        if !self.is_punct("(") {
            self.err_here("expected ( after for");
            let s = self.pos;
            self.skip_balanced_to_stmt_boundary();
            return self.ast.raw(&self.raw_between(s, self.pos));
        }
        self.advance();
        let head_start = self.pos;
        // 头部分派：of / in / 经典
        let mut depth = 0i32;
        let mut of_at = None;
        let mut in_at = None;
        let mut i = self.pos;
        while i < self.toks.len() {
            let t = &self.toks[i];
            if t.kind == TokKind::Eof {
                break;
            }
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "(" | "[" | "{" => depth += 1,
                    ")" if depth == 0 => break,
                    ")" | "]" | "}" => depth -= 1,
                    ";" if depth == 0 => break,
                    _ => {}
                }
            } else if t.kind == TokKind::Ident && depth == 0 {
                if t.text == "of" && of_at.is_none() && in_at.is_none() {
                    of_at = Some(i);
                    break;
                }
                if t.text == "in" && of_at.is_none() && in_at.is_none() {
                    in_at = Some(i);
                    break;
                }
            }
            i += 1;
        }
        if let Some(kw_at) = of_at.or(in_at) {
            let is_of = of_at.is_some();
            let bind_raw = self.raw_between(head_start, kw_at);
            let decl = if self.toks[head_start].kind == TokKind::Ident
                && matches!(self.toks[head_start].text.as_str(), "var" | "let" | "const")
            {
                Some(match self.toks[head_start].text.as_str() {
                    "var" => VarKind::Var,
                    "let" => VarKind::Let,
                    _ => VarKind::Const,
                })
            } else {
                None
            };
            let (name, ty_raw) = if decl.is_some() {
                let n = &self.toks[head_start + 1];
                if n.kind == TokKind::Ident {
                    let nm = n.text.clone();
                    let ty = if head_start + 2 < kw_at && self.toks[head_start + 2].text == ":" {
                        Some(self.raw_between(head_start + 2, kw_at))
                    } else {
                        None
                    };
                    (nm, ty)
                } else {
                    // 解构 for-of——整条原文
                    self.pos = head_start;
                    let s = self.pos;
                    self.skip_balanced_to_stmt_boundary();
                    return self.ast.raw(&self.raw_between(s, self.pos));
                }
            } else {
                (bind_raw.trim().to_string(), None)
            };
            self.pos = kw_at + 1;
            let iterable = self.parse_expr();
            if self.is_punct(")") {
                self.advance();
            } else {
                self.err_here("expected ) in for head");
            }
            let body = self.parse_stmt_or_group();
            return self.ast.for_each(
                &name,
                is_of,
                decl,
                ty_raw.as_deref(),
                iterable,
                body,
                is_await,
            );
        }
        // 经典 for：init（声明已含 `;` 消费）
        let mut inits: Vec<TsId> = Vec::new();
        if !self.is_punct(";") {
            if self.is_kw("var")
                || self.is_kw("const")
                || (self.is_kw("let") && self.let_is_decl())
            {
                let d = self.parse_var_decl_stmt();
                if self.ast.kind(d) == cure_engine::NodeKind::Block {
                    inits = self.ast.children(d).to_vec();
                } else {
                    inits = vec![d];
                }
            } else {
                let e = self.parse_expr();
                inits.push(self.ast.expr_stmt(e));
                if self.is_punct(";") {
                    self.advance();
                } else {
                    self.err_here("expected ; in for head");
                }
            }
        } else {
            self.advance();
        }
        let cond = if self.is_punct(";") {
            None
        } else {
            Some(self.parse_expr())
        };
        if self.is_punct(";") {
            self.advance();
        } else {
            self.err_here("expected ; in for head");
        }
        let mut steps: Vec<TsId> = Vec::new();
        while !self.is_punct(")") && !self.at_eof() {
            let e = self.parse_expr();
            steps.push(self.ast.expr_stmt(e));
            if self.is_punct(",") {
                self.advance();
            } else {
                break;
            }
        }
        if self.is_punct(")") {
            self.advance();
        } else {
            self.err_here("expected ) in for head");
        }
        let body = self.parse_stmt_or_group();
        self.ast.for_(inits, cond, steps, body)
    }

    fn parse_return(&mut self) -> TsId {
        self.advance();
        if self.tk().newline_before || self.is_punct(";") || self.is_punct("}") || self.at_eof() {
            self.consume_semi();
            return self.ast.ret(None);
        }
        let e = self.parse_expr();
        self.consume_semi();
        self.ast.ret(Some(e))
    }

    fn parse_break_continue(&mut self) -> TsId {
        let kw = self.advance().text;
        let label = if self.tk().kind == TokKind::Ident && !self.tk().newline_before {
            Some(self.advance().text)
        } else {
            None
        };
        self.consume_semi();
        if kw == "break" {
            self.ast.break_(label.as_deref())
        } else {
            self.ast.continue_(label.as_deref())
        }
    }

    fn parse_throw(&mut self) -> TsId {
        self.advance();
        if self.tk().newline_before {
            self.err_here("throw followed by newline (ASI)");
            self.consume_semi();
            return self.ast.raw("throw;");
        }
        let e = self.parse_expr();
        self.consume_semi();
        self.ast.throw(e)
    }

    fn parse_try(&mut self) -> TsId {
        self.advance();
        let block = if self.is_punct("{") {
            self.parse_block()
        } else {
            self.err_here("expected { after try");
            return self.ast.raw("try");
        };
        let mut catches = Vec::new();
        while self.is_kw("catch") {
            self.advance();
            let param_raw = if self.is_punct("(") {
                let ps = self.pos;
                self.skip_parens();
                let full = self.raw_between(ps, self.pos);
                Some(
                    full.strip_prefix('(')
                        .and_then(|s| s.strip_suffix(')'))
                        .unwrap_or(&full)
                        .to_string(),
                )
            } else {
                None
            };
            let cbody = if self.is_punct("{") {
                self.parse_block()
            } else {
                self.err_here("expected { after catch");
                self.ast.block(vec![])
            };
            catches.push(self.ast.catch_(param_raw.as_deref(), cbody));
        }
        let finally = if self.is_kw("finally") {
            self.advance();
            if self.is_punct("{") {
                Some(self.parse_block())
            } else {
                self.err_here("expected { after finally");
                None
            }
        } else {
            None
        };
        if catches.is_empty() && finally.is_none() {
            self.err_here("try without catch/finally");
        }
        self.ast.try_(block, catches, finally)
    }

    fn parse_switch(&mut self) -> TsId {
        self.advance();
        if !self.is_punct("(") {
            self.err_here("expected ( after switch");
            let s = self.pos;
            self.skip_balanced_to_stmt_boundary();
            return self.ast.raw(&self.raw_between(s, self.pos));
        }
        self.advance();
        let subject = self.parse_expr();
        if self.is_punct(")") {
            self.advance();
        } else {
            self.err_here("expected )");
        }
        if !self.is_punct("{") {
            self.err_here("expected { after switch");
            let s = self.pos;
            self.skip_balanced_to_stmt_boundary();
            return self.ast.raw(&self.raw_between(s, self.pos));
        }
        self.advance();
        let mut cases = Vec::new();
        while !self.at_eof() && !self.is_punct("}") {
            let before = self.pos;
            if let Some(c) = self.parse_case() {
                cases.push(c);
            }
            if self.pos == before {
                self.advance();
            }
        }
        if self.is_punct("}") {
            self.advance();
        } else {
            self.err_here("expected }");
        }
        self.ast.switch_(subject, cases)
    }

    fn parse_case(&mut self) -> Option<TsId> {
        let case_start = self.pos;
        if self.is_kw("case") {
            self.advance();
            let label = self.parse_expr();
            if self.is_punct("=>") {
                // TS switch 表达式箭头 case——整条原文
                self.skip_balanced_to_stmt_boundary();
                return Some(self.ast.raw(&self.raw_between(case_start, self.pos)));
            }
            if self.is_punct(":") {
                self.advance();
            } else {
                self.err_here("expected : after case");
            }
            let stmts = self.parse_case_stmts();
            Some(self.ast.case_(vec![label], false, false, stmts))
        } else if self.is_kw("default") {
            self.advance();
            if self.is_punct("=>") {
                self.skip_balanced_to_stmt_boundary();
                return Some(self.ast.raw(&self.raw_between(case_start, self.pos)));
            }
            if self.is_punct(":") {
                self.advance();
            } else {
                self.err_here("expected : after default");
            }
            let stmts = self.parse_case_stmts();
            Some(self.ast.case_(vec![], true, false, stmts))
        } else {
            self.err_here("expected case/default in switch");
            None
        }
    }

    fn parse_case_stmts(&mut self) -> Vec<TsId> {
        let mut stmts = Vec::new();
        while !self.at_eof() && !self.is_punct("}") {
            if self.is_kw("case") || self.is_kw("default") {
                break;
            }
            let before = self.pos;
            let s = self.parse_stmt();
            stmts.push(s);
            if self.pos == before {
                self.advance();
            }
        }
        stmts
    }

    fn parse_expr_or_decl_stmt(&mut self) -> TsId {
        let t = self.tk().clone();
        // 标签语句：ident `:` 后随语句形态
        if t.kind == TokKind::Ident && self.is_punct_at(1, ":") {
            let next = self.tk_at(2);
            let stmt_like = match next.kind {
                TokKind::Ident => true,
                TokKind::Punct => matches!(next.text.as_str(), "{" | "("),
                _ => false,
            };
            if stmt_like && !self.tk_at(1).newline_before {
                let name = self.advance().text;
                self.advance(); // :
                let stmt = self.parse_stmt();
                return self.ast.label(&name, stmt);
            }
        }
        let start = self.pos;
        let e = self.parse_expr();
        // 解构赋值语句：Assign(target 是 ObjLit/ArrayLit)
        if let Some(stmt) = self.wrap_destruct_assign(e, start) {
            return stmt;
        }
        self.consume_semi();
        self.ast.expr_stmt(e)
    }

    fn wrap_destruct_assign(&mut self, e: TsId, start: usize) -> Option<TsId> {
        if self.ast.kind(e) != cure_engine::NodeKind::Assign {
            return None;
        }
        let ch = self.ast.children(e).to_vec();
        if ch.len() != 2 {
            return None;
        }
        let (target, value) = (ch[0], ch[1]);
        let pattern = match self.ast.data(target) {
            NodeData::ObjLit { .. } | NodeData::ArrayLit => {
                let eq = self.eq_pos_back(start);
                self.raw_between(start, eq)
            }
            _ => return None,
        };
        Some(self.ast.destruct_stmt("", &pattern, value))
    }

    // ---- 表达式 ----

    fn parse_expr(&mut self) -> TsId {
        let first = self.parse_assign_expr();
        if !self.is_punct(",") {
            return first;
        }
        let mut elems = vec![first];
        while self.is_punct(",") {
            self.advance();
            elems.push(self.parse_assign_expr());
        }
        self.ast.seq(elems)
    }

    fn parse_assign_expr(&mut self) -> TsId {
        if let Some(a) = self.try_parse_arrow() {
            return a;
        }
        if self.is_kw("yield") {
            // yield 表达式：原文捕获（生成器恢复 v1 不简化）
            let s = self.pos;
            self.advance();
            if self.is_punct("*") {
                self.advance();
            }
            if !self.tk().newline_before
                && !self.is_punct(";")
                && !self.is_punct(")")
                && !self.is_punct("]")
                && !self.is_punct("}")
                && !self.is_punct(",")
            {
                let _ = self.parse_assign_expr();
            }
            return self.ast.raw(&self.raw_between(s, self.pos));
        }
        let left = self.parse_conditional();
        if self.tk().kind == TokKind::Punct {
            if let Some(op) = assign_op_of(&self.tk().text) {
                self.advance();
                let right = self.parse_assign_expr();
                return self.ast.assign_op(op, left, right);
            }
            if self.tk().text == "=" {
                self.advance();
                let right = self.parse_assign_expr();
                return self.ast.assign(left, right);
            }
        }
        left
    }

    fn try_parse_arrow(&mut self) -> Option<TsId> {
        let mut p = self.pos;
        let mut is_async = false;
        if self.toks[p].kind == TokKind::Ident && self.toks[p].text == "async" {
            let n = self.toks.get(p + 1)?;
            if (n.kind == TokKind::Ident || (n.kind == TokKind::Punct && n.text == "("))
                && !n.newline_before
            {
                is_async = true;
                p += 1;
            }
        }
        // ident =>
        if self.toks[p].kind == TokKind::Ident {
            let n = self.toks.get(p + 1)?;
            if n.kind == TokKind::Punct && n.text == "=>" {
                let param = self.toks[p].text.clone();
                self.pos = p + 2;
                let body = self.parse_arrow_body();
                return Some(self.ast.lambda(&param, body, true, is_async, false));
            }
            return None;
        }
        // ( … ) [: ret] =>
        if !(self.toks[p].kind == TokKind::Punct && self.toks[p].text == "(") {
            return None;
        }
        let close = self.match_paren(p)?;
        let mut q = close + 1;
        if q < self.toks.len() && self.toks[q].kind == TokKind::Punct && self.toks[q].text == ":" {
            let mut depth = 0i32;
            while q < self.toks.len() {
                let t = &self.toks[q];
                if t.kind == TokKind::Eof {
                    return None;
                }
                if t.kind == TokKind::Punct {
                    match t.text.as_str() {
                        "<" | "(" | "[" | "{" => depth += 1,
                        ">" | ")" | "]" | "}" => depth -= 1,
                        "=>" if depth <= 0 => break,
                        _ => {}
                    }
                }
                q += 1;
            }
        }
        let arrow_tok = self.toks.get(q)?;
        if !(arrow_tok.kind == TokKind::Punct && arrow_tok.text == "=>") {
            return None;
        }
        let params_full = self.raw_between(p, close + 1);
        let params_inner = params_full
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(&params_full)
            .to_string();
        self.pos = q + 1;
        let body = self.parse_arrow_body();
        Some(self.ast.lambda(&params_inner, body, true, is_async, false))
    }

    fn match_paren(&self, i: usize) -> Option<usize> {
        let mut depth = 0i32;
        let mut j = i;
        while j < self.toks.len() {
            let t = &self.toks[j];
            if t.kind == TokKind::Eof {
                return None;
            }
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "(" => depth += 1,
                    ")" => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(j);
                        }
                    }
                    _ => {}
                }
            }
            j += 1;
        }
        None
    }

    fn parse_arrow_body(&mut self) -> TsId {
        if self.is_punct("{") {
            self.parse_block()
        } else {
            self.parse_assign_expr()
        }
    }

    fn parse_conditional(&mut self) -> TsId {
        let cond = self.parse_binary(2);
        if self.is_punct("?") {
            self.advance();
            let then = self.parse_assign_expr();
            if self.is_punct(":") {
                self.advance();
            } else {
                self.err_here("expected : in conditional");
            }
            let els = self.parse_assign_expr();
            return self.ast.ternary(cond, then, els);
        }
        cond
    }

    fn parse_binary(&mut self, min_prec: u8) -> TsId {
        let mut left = self.parse_unary();
        loop {
            let op = if self.tk().kind == TokKind::Punct {
                self.tk().text.clone()
            } else if self.tk().kind == TokKind::Ident
                && matches!(self.tk().text.as_str(), "in" | "instanceof")
            {
                self.tk().text.clone()
            } else {
                break;
            };
            let Some((prec, binop)) = bin_info(&op) else {
                break;
            };
            if prec < min_prec {
                break;
            }
            self.advance();
            let right = if op == "**" {
                self.parse_binary(prec) // 右结合
            } else {
                self.parse_binary(prec + 1)
            };
            left = self.ast.bin(binop, left, right);
        }
        left
    }

    fn parse_unary(&mut self) -> TsId {
        let t = self.tk().clone();
        if t.kind == TokKind::Punct {
            let op = match t.text.as_str() {
                "!" => UnOp::Not,
                "~" => UnOp::BitNot,
                "+" => UnOp::Plus,
                "-" => UnOp::Neg,
                "++" => UnOp::PreInc,
                "--" => UnOp::PreDec,
                _ => return self.parse_postfix(),
            };
            self.advance();
            let e = self.parse_unary();
            return self.ast.un(op, e);
        }
        if t.kind == TokKind::Ident {
            match t.text.as_str() {
                "typeof" => {
                    self.advance();
                    let e = self.parse_unary();
                    return self.ast.un(UnOp::TypeOf, e);
                }
                "void" => {
                    self.advance();
                    let e = self.parse_unary();
                    return self.ast.un(UnOp::Void, e);
                }
                "delete" => {
                    self.advance();
                    let e = self.parse_unary();
                    return self.ast.un(UnOp::Delete, e);
                }
                "await" if !self.await_is_ident() => {
                    self.advance();
                    let e = self.parse_unary();
                    return self.ast.un(UnOp::Await, e);
                }
                _ => {}
            }
        }
        self.parse_postfix()
    }

    /// `await` 是否是普通标识符（作为值引用——`await(x)` 调用形/实参）。
    fn await_is_ident(&self) -> bool {
        self.is_punct_at(1, "(") || self.tk_at(1).newline_before
    }

    fn parse_postfix(&mut self) -> TsId {
        let mut e = self.parse_primary_with_new();
        loop {
            let t = self.tk().clone();
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "." => {
                        self.advance();
                        let name = self.expect_member_name();
                        e = self.ast.member(e, &name, false);
                    }
                    "?." => {
                        self.advance();
                        if self.is_punct("(") {
                            let args = self.parse_call_args();
                            e = self.ast.opt_call(e, args);
                        } else if self.is_punct("[") {
                            self.advance();
                            let idx = self.parse_expr();
                            if self.is_punct("]") {
                                self.advance();
                            } else {
                                self.err_here("expected ]");
                            }
                            e = self.ast.index(e, idx, true);
                        } else {
                            let name = self.expect_member_name();
                            e = self.ast.member(e, &name, true);
                        }
                    }
                    "[" => {
                        self.advance();
                        let idx = self.parse_expr();
                        if self.is_punct("]") {
                            self.advance();
                        } else {
                            self.err_here("expected ]");
                        }
                        e = self.ast.index(e, idx, false);
                    }
                    "(" => {
                        let args = self.parse_call_args();
                        e = self.ast.call(e, args);
                    }
                    "!" if !t.newline_before => {
                        // TS 非空断言
                        self.advance();
                        e = self.ast.ts_non_null(e);
                    }
                    "++" | "--" if !t.newline_before => {
                        let op = if t.text == "++" { UnOp::PostInc } else { UnOp::PostDec };
                        self.advance();
                        e = self.ast.un(op, e);
                    }
                    _ => break,
                }
                continue;
            }
            if t.kind == TokKind::Ident
                && matches!(t.text.as_str(), "as" | "satisfies")
                && !t.newline_before
                && !self.is_punct_at(1, "(")
                && !self.is_punct_at(1, "<")
            {
                let is_satisfies = t.text == "satisfies";
                self.advance();
                let ts = self.pos;
                self.skip_type_region();
                let ty = self.raw_between(ts, self.pos);
                e = if is_satisfies {
                    self.ast.ts_satisfies(&ty, e)
                } else {
                    self.ast.ts_as(&ty, e)
                };
                continue;
            }
            // 标签模板：**新模板开始**（cont=false——插值续接段属外围
            // 模板，parse_template_after 的循环在等它，不能当 tag 附着）
            if matches!(t.kind, TokKind::TemplateQuasi { cont: false, .. }) {
                e = self.parse_template_after(Some(e));
                continue;
            }
            break;
        }
        e
    }

    fn parse_primary_with_new(&mut self) -> TsId {
        if self.is_kw("new") {
            self.advance();
            let mut e = self.parse_primary();
            loop {
                if self.is_punct(".") {
                    self.advance();
                    let name = self.expect_member_name();
                    e = self.ast.member(e, &name, false);
                } else if self.is_punct("[") {
                    self.advance();
                    let idx = self.parse_expr();
                    if self.is_punct("]") {
                        self.advance();
                    } else {
                        self.err_here("expected ]");
                    }
                    e = self.ast.index(e, idx, false);
                } else {
                    break;
                }
            }
            let args = if self.is_punct("(") {
                self.parse_call_args()
            } else {
                vec![]
            };
            return self.ast.new_(e, args);
        }
        self.parse_primary()
    }

    fn parse_call_args(&mut self) -> Vec<TsId> {
        if !self.is_punct("(") {
            self.err_here("expected (");
            return vec![];
        }
        self.advance();
        let mut args = Vec::new();
        while !self.is_punct(")") && !self.at_eof() {
            if self.is_punct("...") {
                self.advance();
                let e = self.parse_assign_expr();
                args.push(self.ast.spread(e));
            } else {
                args.push(self.parse_assign_expr());
            }
            if self.is_punct(",") {
                self.advance();
            } else {
                break;
            }
        }
        if self.is_punct(")") {
            self.advance();
        } else {
            self.err_here("expected )");
        }
        args
    }

    /// 模板字面量解析（当前 token 是首 quasi）。tag = Some 时构造标签模板。
    fn parse_template_after(&mut self, tag: Option<TsId>) -> TsId {
        let mut quasis: Vec<String> = Vec::new();
        let mut interps: Vec<TsId> = Vec::new();
        let first = self.advance();
        let mut is_end = match first.kind {
            TokKind::TemplateQuasi { is_end, .. } => is_end,
            _ => true,
        };
        quasis.push(first.text.clone());
        while !is_end {
            let e = self.parse_expr();
            interps.push(e);
            let t = self.tk().clone();
            match t.kind {
                TokKind::TemplateQuasi { is_end: ie, .. } => {
                    self.advance();
                    quasis.push(t.text.clone());
                    is_end = ie;
                }
                _ => {
                    self.err_here("expected template continuation");
                    break;
                }
            }
        }
        match tag {
            Some(t) => self.ast.tagged_template(t, quasis, interps),
            None => self.ast.template(quasis, interps),
        }
    }

    fn expect_member_name(&mut self) -> String {
        let t = self.tk().clone();
        if t.kind == TokKind::Ident || t.kind == TokKind::Num {
            self.advance();
            t.text
        } else if t.kind == TokKind::Punct && t.text == "#" {
            self.advance();
            let n = self.tk().text.clone();
            if self.tk().kind == TokKind::Ident {
                self.advance();
            }
            format!("#{n}")
        } else {
            self.err_here("expected member name");
            String::new()
        }
    }

    /// 类型区域扫描（as/satisfies/注解后）：平衡泛型括号，停在表达式边界。
    fn skip_type_region(&mut self) {
        let mut depth = 0i32;
        while !self.at_eof() {
            let t = self.tk().clone();
            if t.kind == TokKind::Punct {
                match t.text.as_str() {
                    "<" | "(" | "[" | "{" => {
                        depth += 1;
                        self.advance();
                    }
                    ">" => {
                        depth -= 1;
                        self.advance();
                        if depth <= 0 {
                            return;
                        }
                    }
                    ">>" => {
                        depth -= 2;
                        self.advance();
                        if depth <= 0 {
                            return;
                        }
                    }
                    ">>>" => {
                        depth -= 3;
                        self.advance();
                        if depth <= 0 {
                            return;
                        }
                    }
                    ")" | "]" | "}" => {
                        if depth == 0 {
                            return;
                        }
                        depth -= 1;
                        self.advance();
                    }
                    ";" | "," | "=" if depth == 0 => { return; },
                    _ => { self.advance(); },
                }
                continue;
            }
            if t.kind == TokKind::Ident {
                let nxt = self.tk_at(1).clone();
                let stop_after_ident = nxt.kind == TokKind::Punct
                    && matches!(nxt.text.as_str(), ")" | "]" | "}" | "," | ";")
                    && depth == 0;
                self.advance();
                if stop_after_ident {
                    return;
                }
                continue;
            }
            if t.kind == TokKind::Str || t.kind == TokKind::Num {
                self.advance();
                continue;
            }
            return;
        }
    }

    fn parse_primary(&mut self) -> TsId {
        let t = self.tk().clone();
        match t.kind {
            TokKind::Num => {
                self.advance();
                let lit = parse_num_lit(&t.text);
                self.ast.lit(lit)
            }
            TokKind::Str => {
                self.advance();
                let v = decode_str_lit(&t.text);
                self.ast.lit(JsLit::Str(v))
            }
            TokKind::Regex => {
                self.advance();
                self.ast.lit(JsLit::Regex(t.text))
            }
            TokKind::TemplateQuasi { .. } => self.parse_template_after(None),
            TokKind::Ident => match t.text.as_str() {
                "function" => self.parse_function_expr(false),
                "async" if self.tk_at(1).text == "function" && !self.tk_at(1).newline_before => {
                    self.parse_function_expr(true)
                }
                "class" if self.tk_at(1).kind == TokKind::Ident || self.tk_at(1).kind == TokKind::Punct => {
                    // 类表达式（class / class Name …）
                    if self.tk_at(1).kind == TokKind::Ident
                        || self.is_punct_at(1, "extends")
                        || self.is_punct_at(1, "{")
                        || self.is_kw_at(1, "extends")
                    {
                        let s = self.pos;
                        self.skip_class_expr();
                        let raw = self.raw_between(s, self.pos);
                        self.ast.class_expr(&raw)
                    } else {
                        self.advance();
                        self.ast.var(&t.text)
                    }
                }
                "this" => {
                    self.advance();
                    self.ast.this()
                }
                "super" => {
                    let s = self.pos;
                    self.advance();
                    let mut raw = String::from("super");
                    if self.is_punct(".") {
                        self.advance();
                        raw.push('.');
                        raw.push_str(&self.expect_member_name());
                    } else if self.is_punct("[") {
                        self.advance();
                        let _ = self.parse_expr();
                        if self.is_punct("]") {
                            self.advance();
                        }
                        raw.push_str(&self.raw_between(s, self.pos));
                    } else if self.is_punct("(") {
                        let ps = self.pos;
                        self.skip_parens();
                        raw.push_str(&self.raw_between(ps, self.pos));
                    }
                    self.ast.raw(&raw)
                }
                "true" | "false" => {
                    self.advance();
                    self.ast.lit(JsLit::Bool(t.text == "true"))
                }
                "null" => {
                    self.advance();
                    self.ast.lit(JsLit::Null)
                }
                _ => {
                    self.advance();
                    self.ast.var(&t.text)
                }
            },
            TokKind::Punct => match t.text.as_str() {
                "(" => {
                    self.advance();
                    let e = self.parse_expr();
                    if self.is_punct(")") {
                        self.advance();
                    } else {
                        self.err_here("expected )");
                    }
                    self.ast.paren(e)
                }
                "[" => {
                    self.advance();
                    let mut elems = Vec::new();
                    while !self.is_punct("]") && !self.at_eof() {
                        if self.is_punct("...") {
                            self.advance();
                            let e = self.parse_assign_expr();
                            elems.push(self.ast.spread(e));
                        } else {
                            elems.push(self.parse_assign_expr());
                        }
                        if self.is_punct(",") {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    if self.is_punct("]") {
                        self.advance();
                    } else {
                        self.err_here("expected ]");
                    }
                    self.ast.array_lit(elems)
                }
                "{" => self.parse_obj_lit(),
                "<" => {
                    // JSX 元素（.tsx）——原文捕获
                    let raw = self.capture_jsx();
                    self.ast.raw(&raw)
                }
                "#" => {
                    self.advance();
                    let n = self.expect_member_name();
                    self.ast.var(&n)
                }
                _ => {
                    self.err_here(&format!("unexpected token {:?}", t.text));
                    self.advance();
                    self.ast.raw(&t.text)
                }
            },
            TokKind::Eof => {
                self.err_here("unexpected EOF");
                self.ast.empty()
            }
        }
    }

    fn is_kw_at(&self, off: usize, w: &str) -> bool {
        let t = self.tk_at(off);
        t.kind == TokKind::Ident && t.text == w
    }

    fn parse_function_expr(&mut self, is_async: bool) -> TsId {
        self.advance(); // function（async 已在外层 advance）
        let mut is_gen = false;
        if self.is_punct("*") {
            is_gen = true;
            self.advance();
        }
        let _name = if self.tk().kind == TokKind::Ident && !self.is_punct("(") {
            self.advance().text
        } else {
            String::new()
        };
        if !self.is_punct("(") {
            self.err_here("expected ( after function");
            return self.ast.raw("function");
        }
        let ps = self.pos;
        self.skip_parens();
        let full = self.raw_between(ps, self.pos);
        let inner = full
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(&full)
            .to_string();
        let body = if self.is_punct("{") {
            self.parse_block()
        } else {
            self.err_here("expected { in function expression");
            self.ast.block(vec![])
        };
        self.ast.lambda(&inner, body, false, is_async, is_gen)
    }

    fn parse_obj_lit(&mut self) -> TsId {
        self.advance(); // {
        let mut props: Vec<String> = Vec::new();
        let mut values: Vec<TsId> = Vec::new();
        while !self.is_punct("}") && !self.at_eof() {
            if self.is_punct("...") {
                self.advance();
                let e = self.parse_assign_expr();
                props.push("...".into());
                values.push(e);
            } else {
                let (p, v) = self.parse_obj_prop();
                props.push(p);
                values.push(v);
            }
            if self.is_punct(",") {
                self.advance();
            } else {
                break;
            }
        }
        if self.is_punct("}") {
            self.advance();
        } else {
            self.err_here("expected }");
        }
        self.ast.obj_lit(props, values)
    }

    fn parse_obj_prop(&mut self) -> (String, TsId) {
        let t = self.tk().clone();
        // 访问器/异步方法：get x() / set x(v) / async m()——整体原文
        if t.kind == TokKind::Ident
            && matches!(t.text.as_str(), "get" | "set" | "async")
            && self.tk_at(1).kind == TokKind::Ident
            && self.is_punct_at(2, "(")
        {
            let s = self.pos;
            // 扫过方法体（平衡花括号）
            self.skip_obj_method();
            let raw = self.raw_between(s, self.pos);
            let raw = raw.trim_end().trim_end_matches(';').to_string();
            return (raw, self.ast.raw(""));
        }
        // 方法简写 name(params) { body }
        if t.kind == TokKind::Ident && self.is_punct_at(1, "(") && !self.tk_at(1).newline_before {
            let name = self.advance().text;
            let ps = self.pos;
            self.skip_parens();
            let params_full = self.raw_between(ps, self.pos);
            let inner = params_full
                .strip_prefix('(')
                .and_then(|x| x.strip_suffix(')'))
                .unwrap_or(&params_full)
                .to_string();
            let body = if self.is_punct("{") {
                self.parse_block()
            } else {
                self.err_here("expected { in object method");
                self.ast.block(vec![])
            };
            let lam = self.ast.lambda(&inner, body, false, false, false);
            return (format!("{name}{params_full}"), lam);
        }
        // 键
        let key = match t.kind {
            TokKind::Ident => {
                self.advance();
                t.text.clone()
            }
            TokKind::Str | TokKind::Num => {
                self.advance();
                t.text.clone()
            }
            TokKind::Punct if t.text == "[" => {
                // 计算键 [expr]：raw 区间**不含**两端括号（含进去会让
                // 打印层 format 再包一层——每轮往返多一层 `[`；真实
                // 语料三轮振荡累积抓获）
                let s = self.pos;
                self.advance();
                let _ = self.parse_assign_expr();
                if self.is_punct("]") {
                    self.advance();
                } else {
                    self.err_here("expected ]");
                }
                format!("[{}]", self.raw_between(s + 1, self.pos - 1))
            }
            _ => {
                self.err_here("expected property key");
                self.advance();
                String::new()
            }
        };
        if !self.is_punct(":") {
            // 简写 { x }
            if t.kind == TokKind::Ident {
                return (key.clone(), self.ast.var(&t.text));
            }
            return (key, self.ast.raw(""));
        }
        self.advance(); // :
        let v = self.parse_assign_expr();
        (key, v)
    }

    /// 扫过对象方法（`name(…) { … }`）到平衡体尾。
    fn skip_obj_method(&mut self) {
        // 跳键
        self.advance(); // get/set/async
        if self.tk().kind == TokKind::Ident {
            self.advance();
        }
        self.skip_parens();
        if self.is_punct("{") {
            self.skip_braces();
        }
        // 尾分号可选
        if self.is_punct(";") {
            self.advance();
        }
    }

    /// 类表达式边界扫描。
    fn skip_class_expr(&mut self) {
        self.advance(); // class
        if self.tk().kind == TokKind::Ident && !self.is_punct("(") {
            self.advance();
        }
        while !self.at_eof() && !self.is_punct("{") && !self.is_punct(";") && !self.is_punct(")") && !self.is_punct(",") {
            self.advance();
        }
        if self.is_punct("{") {
            let mut depth = 0i32;
            loop {
                if self.at_eof() {
                    return;
                }
                if self.is_punct("{") {
                    depth += 1;
                } else if self.is_punct("}") {
                    depth -= 1;
                    if depth == 0 {
                        self.advance();
                        return;
                    }
                }
                self.advance();
            }
        }
    }

    /// JSX 元素原文捕获（平衡标签/插值/字符串）。
    fn capture_jsx(&mut self) -> String {
        let start = self.pos;
        let mut stack: Vec<String> = Vec::new();
        loop {
            if self.at_eof() {
                break;
            }
            let t = self.tk().clone();
            match t.kind {
                TokKind::Punct => match t.text.as_str() {
                    "<" => {
                        self.advance();
                        if self.is_punct("/") {
                            self.advance();
                            let name = self.expect_member_name();
                            if self.is_punct(">") {
                                self.advance();
                            }
                            loop {
                                match stack.pop() {
                                    Some(n) if n == name => break,
                                    Some(_) => continue,
                                    None => break,
                                }
                            }
                            if stack.is_empty() {
                                break;
                            }
                        } else {
                            let name = if self.is_punct(">") {
                                String::new()
                            } else {
                                self.expect_member_name()
                            };
                            stack.push(name.clone());
                            // 属性扫描到 `>` / `/>`
                            let mut attr_depth = 0i32;
                            loop {
                                if self.at_eof() {
                                    return self.raw_between(start, self.pos);
                                }
                                let a = self.tk().clone();
                                if a.kind == TokKind::Punct {
                                    if a.text == "/" && self.is_punct_at(1, ">") && attr_depth == 0 {
                                        // `/>` 自闭合（两 token 形态）
                                        self.advance();
                                        self.advance();
                                        stack.pop();
                                        if stack.is_empty() {
                                            return self.raw_between(start, self.pos);
                                        }
                                        break;
                                    }
                                    match a.text.as_str() {
                                        "{" => {
                                            attr_depth += 1;
                                            self.advance();
                                            if attr_depth == 1 {
                                                let _ = self.parse_expr();
                                                if self.is_punct("}") {
                                                    self.advance();
                                                    attr_depth -= 1;
                                                }
                                            }
                                        }
                                        "}" => {
                                            attr_depth -= 1;
                                            self.advance();
                                        }
                                        ">" if attr_depth == 0 => {
                                            self.advance();
                                            break;
                                        }
                                        _ => { self.advance(); },
                                    }
                                } else {
                                    self.advance();
                                }
                            }
                        }
                    }
                    "{" => {
                        self.advance();
                        let _ = self.parse_expr();
                        if self.is_punct("}") {
                            self.advance();
                        }
                    }
                    _ => { self.advance(); },
                },
                _ => { self.advance(); },
            }
        }
        self.raw_between(start, self.pos)
    }
}

/// 数值字面量 → JsLit（十六/八/二进制与分隔符原文保留）。
fn parse_num_lit(text: &str) -> JsLit {
    let clean: String = text.chars().filter(|c| *c != '_').collect();
    if clean.ends_with('n') {
        return JsLit::BigInt(text.to_string());
    }
    for (prefix, radix) in [("0x", 16u32), ("0X", 16), ("0o", 8), ("0O", 8), ("0b", 2), ("0B", 2)] {
        if let Some(digits) = clean.strip_prefix(prefix) {
            if let Ok(v) = i64::from_str_radix(digits, radix) {
                return JsLit::NumRaw { text: text.to_string(), val: NumVal::Int(v) };
            }
            break;
        }
    }
    if !clean.contains('.') && !clean.contains('e') && !clean.contains('E') {
        if let Ok(v) = clean.parse::<i64>() {
            return JsLit::Int(v);
        }
    }
    if let Ok(v) = clean.parse::<f64>() {
        if v.is_finite() {
            return JsLit::Double(v);
        }
    }
    JsLit::NumRaw { text: text.to_string(), val: NumVal::Double(f64::NAN) }
}

/// 字符串字面量解码（原文含引号）。
fn decode_str_lit(raw: &str) -> String {
    let bytes = raw.as_bytes();
    if bytes.len() < 2 {
        return raw.to_string();
    }
    let inner = &raw[1..raw.len() - 1];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('b') => out.push('\u{0008}'),
            Some('f') => out.push('\u{000c}'),
            Some('v') => out.push('\u{000b}'),
            Some('0') => out.push('\0'),
            Some('x') => {
                let h: String = (0..2).filter_map(|_| chars.next()).collect();
                if let Ok(v) = u32::from_str_radix(&h, 16) {
                    if let Some(ch) = char::from_u32(v) {
                        out.push(ch);
                    }
                }
            }
            Some('u') => {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    let h: String = chars.by_ref().take_while(|c| *c != '}').collect();
                    if let Ok(v) = u32::from_str_radix(&h, 16) {
                        if let Some(ch) = char::from_u32(v) {
                            out.push(ch);
                        }
                    }
                } else {
                    let h: String = (0..4).filter_map(|_| chars.next()).collect();
                    if let Ok(v) = u32::from_str_radix(&h, 16) {
                        if let Some(ch) = char::from_u32(v) {
                            out.push(ch);
                        }
                    }
                }
            }
            Some('\n') => {}
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip_ok(src: &str) -> ParseOutcome {
        let out = parse(src);
        assert!(out.errors.is_empty(), "parse errors: {:#?}", out.errors);
        out
    }

    #[test]
    fn parses_basic_module() {
        let out = roundtrip_ok("const x = 1;\nfunction f(a) { return a + x; }\n");
        assert_eq!(out.unit.items.len(), 2);
    }

    #[test]
    fn parses_arrow_and_template() {
        let out = roundtrip_ok("const f = (a, b) => a + b;\nconst g = x => x * 2;\nconst s = `x${a}y`;\n");
        assert!(out.errors.is_empty());
    }

    #[test]
    fn parses_control_flow() {
        let out = roundtrip_ok(
            "function f(n) {\n  if (n > 1) { return 1; } else { return 2; }\n  for (let i = 0; i < n; i++) { f(i); }\n  for (const x of [1,2]) { x; }\n  try { f(1); } catch (e) { throw e; } finally { f(0); }\n  switch (n) { case 1: return 2; default: return 3; }\n  while (true) { break; }\n}\n",
        );
        assert!(out.errors.is_empty(), "{:#?}", out.errors);
    }

    #[test]
    fn asi_return() {
        let out = roundtrip_ok("function f() {\n  return\n  1;\n}\n");
        assert!(out.errors.is_empty());
    }

    #[test]
    fn regex_vs_division_in_parser() {
        let out = roundtrip_ok("var re = /ab+c/g;\nvar d = a / b;\nvar x = /x/.test(s);\n");
        assert!(out.errors.is_empty());
    }

    #[test]
    fn nested_template() {
        let out = roundtrip_ok("const s = `a${ `b${x}` }c`;\n");
        assert!(out.errors.is_empty());
    }

    #[test]
    fn modern_syntax_no_panic() {
        let out = parse(
            "class A extends B {\n  #x = 1;\n  static y = 2;\n  constructor(a) { super(a); }\n  async m(x) { return x; }\n  get val() { return this.#x; }\n  static { A.z = 3; }\n}\nconst o = { a: 1, b, m() { return 1; } };\nconst [p, q = 2] = arr;\nconst { a: { b: c } } = obj;\nlabel: for (;;) { break label; }\n",
        );
        let _ = out;
    }

    #[test]
    fn optional_chain_and_nullish() {
        let out = roundtrip_ok("const v = a?.b ?? d ?? e;\n");
        assert!(out.errors.is_empty(), "{:#?}", out.errors);
    }

    #[test]
    fn jsx_raw_capture() {
        let out = roundtrip_ok(
            "const e = <div className=\"a\">hello {name}<br/><span>x</span></div>;\n",
        );
        assert!(out.errors.is_empty());
    }

    #[test]
    fn ts_annotations() {
        let out = roundtrip_ok(
            "const x: number = 1;\nlet y = z satisfies string;\nlet w = q as unknown as string;\nfunction f(a: number): string { return `${a}`; }\n",
        );
        assert!(out.errors.is_empty(), "{:#?}", out.errors);
    }

    #[test]
    fn logical_assign_and_pow() {
        let out = roundtrip_ok("a ||= b;\nc &&= d;\ne ??= f;\ng **= 2;\nconst h = 2 ** 3 ** 2;\n");
        assert!(out.errors.is_empty());
    }

    #[test]
    fn destructuring_statements() {
        let out = roundtrip_ok("[a, b] = [b, a];\nconst { x, y } = point;\n");
        let _ = out;
    }

    #[test]
    fn for_await_of() {
        let out = roundtrip_ok("async function f(stream) { for await (const chunk of stream) { use(chunk); } }\n");
        assert!(out.errors.is_empty());
    }

    #[test]
    fn generators_yield() {
        let out = parse("function* g() { yield 1; yield* h; const x = yield; }\n");
        let _ = out;
    }

    #[test]
    fn comma_seq_and_paren() {
        let out = roundtrip_ok("for (i = 0, j = 1; i < j; i++, j--) { f(i, j); }\n");
        assert!(out.errors.is_empty());
    }
}
