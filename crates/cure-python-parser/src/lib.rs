//! 容错 Python 3.15 解析器：缩进块、装饰器、类型参数、match、f/t-string
//! 等语法；未建模/解析失败的语句整段原文保真（含其缩进块）。

pub mod lexer;

use cure_python_ast::{
    CallArg, CompKind, LitKind, Literal, NodeData, PyAst, PyBin, PyBoolOp, PyCmp, PyId, PyUn,
};
use lexer::{tokenize, TokKind, Token};

#[derive(Clone, Debug)]
pub struct ParseError {
    pub line: u32,
    pub col: u32,
    pub message: String,
}

pub struct ParseOutcome {
    pub ast: PyAst,
    pub module: PyId,
    pub errors: Vec<ParseError>,
}

type R<T> = Result<T, String>;

pub fn parse(src: &str) -> ParseOutcome {
    // UTF-8 BOM 不是 Python 源代码的一部分（CPython 自行剥离）。
    let src = src.strip_prefix('\u{feff}').unwrap_or(src);
    let (toks, lex_errs) = tokenize(src);
    let bool_names = bool_literals_are_shadowed(&toks);
    let mut p = Parser {
        src,
        toks,
        pos: 0,
        ast: PyAst::new(),
        errors: Vec::new(),
        indent_stack: vec![String::new()],
        depth: 0,
        no_cond_lambda: false,
        bool_names,
    };
    for e in lex_errs {
        p.errors.push(ParseError {
            line: e.line,
            col: e.col,
            message: e.message,
        });
    }
    let mut items = Vec::new();
    // 保留 shebang / PEP 263 编码声明：Python 2 源码没有它就无法按
    // UTF-8 解码非 ASCII 内容，改成别的编码也可能改变语义。
    if let Some(header) = leading_header(src) {
        items.push(p.ast.raw_stmt(&header));
    }
    while !p.at_eof() {
        let before = p.pos;
        if p.eat_kind(TokKind::Newline)
            || p.eat_kind(TokKind::Indent)
            || p.eat_kind(TokKind::Dedent)
        {
            continue;
        }
        p.parse_stmt_into(&mut items);
        if p.pos == before {
            p.advance();
        }
    }
    let module = p.ast.module(items);
    p.ast.set_bool_literals_shadowed(p.bool_names);
    ParseOutcome {
        ast: p.ast,
        module,
        errors: p.errors,
    }
}

struct Parser<'a> {
    src: &'a str,
    toks: Vec<Token>,
    pos: usize,
    ast: PyAst,
    errors: Vec<ParseError>,
    indent_stack: Vec<String>,
    depth: usize,
    /// 推导式 iterable 顶层的 lambda：Python 2 把随后的 `if` 归给 comp_if，
    /// 而不是 lambda 体的条件表达式。
    no_cond_lambda: bool,
    /// Python 2 允许 `True = 0`/`False = 1`；一旦出现绑定，模块内所有
    /// True/False 都按普通名字处理，避免把被遮蔽的条件折成常量。
    bool_names: bool,
}

impl<'a> Parser<'a> {
    fn tk(&self) -> &Token {
        &self.toks[self.pos.min(self.toks.len() - 1)]
    }
    fn tk_at(&self, off: usize) -> &Token {
        &self.toks[(self.pos + off).min(self.toks.len() - 1)]
    }
    fn at_eof(&self) -> bool {
        self.tk().kind == TokKind::Eof
    }
    fn advance(&mut self) -> Token {
        let t = self.tk().clone();
        if self.pos + 1 < self.toks.len() {
            self.pos += 1;
        }
        t
    }
    fn eat_kind(&mut self, k: TokKind) -> bool {
        if self.tk().kind == k {
            self.advance();
            true
        } else {
            false
        }
    }
    fn at_op(&self, op: &str) -> bool {
        self.tk().kind == TokKind::Op && self.tk().text == op
    }
    fn at_op_at(&self, off: usize, op: &str) -> bool {
        self.tk_at(off).kind == TokKind::Op && self.tk_at(off).text == op
    }
    fn eat_op(&mut self, op: &str) -> bool {
        if self.at_op(op) {
            self.advance();
            true
        } else {
            false
        }
    }
    fn expect_op(&mut self, op: &str) -> R<Token> {
        if self.at_op(op) {
            Ok(self.advance())
        } else {
            Err(format!("expected `{op}`, found `{}`", self.tk().text))
        }
    }
    fn at_kw(&self, kw: &str) -> bool {
        self.tk().kind == TokKind::Name && self.tk().text == kw
    }
    fn at_kw_at(&self, off: usize, kw: &str) -> bool {
        self.tk_at(off).kind == TokKind::Name && self.tk_at(off).text == kw
    }
    fn eat_kw(&mut self, kw: &str) -> bool {
        if self.at_kw(kw) {
            self.advance();
            true
        } else {
            false
        }
    }
    fn expect_kw(&mut self, kw: &str) -> R<Token> {
        if self.at_kw(kw) {
            Ok(self.advance())
        } else {
            Err(format!("expected `{kw}`, found `{}`", self.tk().text))
        }
    }
    fn expect_name(&mut self) -> R<String> {
        if self.tk().kind == TokKind::Name {
            Ok(self.advance().text)
        } else {
            Err(format!("expected identifier, found `{}`", self.tk().text))
        }
    }
    fn err_here(&mut self, msg: &str) {
        let t = self.tk().clone();
        self.errors.push(ParseError {
            line: t.line,
            col: t.col,
            message: msg.into(),
        });
    }
    fn eat_newlines(&mut self) {
        while self.eat_kind(TokKind::Newline) {}
    }

    fn parse_stmt_into(&mut self, out: &mut Vec<PyId>) {
        let start = self.pos;
        if let Err(msg) = self.parse_stmt(out) {
            self.err_here(&msg);
            let raw = self.capture_raw_from(start);
            out.push(raw);
        }
    }

    fn parse_stmt(&mut self, out: &mut Vec<PyId>) -> R<()> {
        if self.depth > cure_python_ast::MAX_NEST {
            return Err("nesting too deep".into());
        }
        if self.at_op("@") {
            let first = self.pos;
            let mut decos = Vec::new();
            let base = self.indent_stack.last().cloned().unwrap_or_default();
            let target = "    ".repeat(self.indent_stack.len().saturating_sub(1));
            while self.at_op("@") {
                let s = self.pos;
                while !self.at_eof() && self.tk().kind != TokKind::Newline {
                    self.advance();
                }
                let raw = self.slice_tokens(s, self.pos).trim_end().to_string();
                let raw_start = self.toks[s].start as usize;
                let spans: Vec<(usize, usize)> = self.toks[s..self.pos]
                    .iter()
                    .filter(|t| t.kind == TokKind::Str)
                    .map(|t| {
                        (
                            (t.start as usize).saturating_sub(raw_start),
                            (t.end as usize).saturating_sub(raw_start),
                        )
                    })
                    .collect();
                decos.push(reindent_decorator(&raw, &spans, &base, &target));
                self.eat_newlines();
            }
            // 首个装饰器行由打印层加当前缩进；后续装饰器行是绝对缩进，
            // 这里补上所在块的基准缩进（每个装饰器的续行本身已绝对）。
            let deco = decos
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    if i == 0 {
                        d.clone()
                    } else {
                        format!("{target}{d}")
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            let mut is_async = false;
            if self.at_kw("async") && self.at_kw_at(1, "def") {
                is_async = true;
                self.advance();
            }
            if self.at_kw("def") {
                let f = self.parse_function(is_async, &deco)?;
                out.push(f);
                return Ok(());
            }
            if !is_async && self.at_kw("class") {
                let c = self.parse_class(&deco)?;
                out.push(c);
                return Ok(());
            }
            self.pos = first;
            return Err("decorator without def/class".into());
        }
        if self.at_kw("async") && self.at_kw_at(1, "def") {
            self.advance();
            out.push(self.parse_function(true, "")?);
            return Ok(());
        }
        if self.at_kw("async") && self.at_kw_at(1, "for") {
            self.advance();
            out.push(self.parse_for(true)?);
            return Ok(());
        }
        if self.at_kw("async") && self.at_kw_at(1, "with") {
            self.advance();
            out.push(self.parse_with(true)?);
            return Ok(());
        }
        if self.at_kw("def") {
            out.push(self.parse_function(false, "")?);
            return Ok(());
        }
        if self.at_kw("class") {
            out.push(self.parse_class("")?);
            return Ok(());
        }
        if self.at_kw("if") {
            out.push(self.parse_if_chain()?);
            return Ok(());
        }
        if self.at_kw("while") {
            out.push(self.parse_while()?);
            return Ok(());
        }
        if self.at_kw("for") {
            out.push(self.parse_for(false)?);
            return Ok(());
        }
        if self.at_kw("try") {
            out.push(self.parse_try()?);
            return Ok(());
        }
        if self.at_kw("with") {
            out.push(self.parse_with(false)?);
            return Ok(());
        }
        if self.at_kw("match") && self.looks_like_match_stmt() {
            out.push(self.parse_match()?);
            return Ok(());
        }
        if self.at_kw("type")
            && self.tk_at(1).kind == TokKind::Name
            && (self.at_op_at(2, "=") || self.at_op_at(2, "["))
        {
            let s = self.pos;
            self.skip_to_newline();
            let raw = self.slice_tokens(s, self.pos).trim().to_string();
            out.push(self.ast.raw_stmt(&raw));
            self.eat_newlines();
            return Ok(());
        }
        self.parse_simple_line(out)
    }

    fn parse_simple_line(&mut self, out: &mut Vec<PyId>) -> R<()> {
        loop {
            if self.at_eof() || self.tk().kind == TokKind::Newline {
                break;
            }
            let start = self.pos;
            match self.parse_small_stmt() {
                Ok(id) => out.push(id),
                Err(msg) => {
                    self.err_here(&msg);
                    let raw = self.capture_raw_from(start);
                    out.push(raw);
                    return Ok(());
                }
            }
            if self.eat_op(";") {
                continue;
            }
            break;
        }
        if self.tk().kind == TokKind::Newline {
            self.advance();
            Ok(())
        } else if self.at_eof() || self.tk().kind == TokKind::Dedent {
            Ok(())
        } else {
            Err(format!("unexpected `{}` after statement", self.tk().text))
        }
    }

    fn parse_small_stmt(&mut self) -> R<PyId> {
        // `print[1, 2]` / `print{...}` 在 Python 2 是 print 语句，在 Python 3
        // 是下标；两种语义都只能靠原文保全（下标 tuple 会被加点号）。
        if (self.at_kw("print") || self.at_kw("exec"))
            && (self.at_op_at(1, "[") || self.at_op_at(1, "{"))
            && self.next_token_adjacent()
        {
            let s = self.pos;
            return Ok(self.capture_line_from(s));
        }
        // `print (a), b` / `print(a) if x else b`：Python 2 是 print 语句，
        // Python 3 是（或非法）表达式；只有调用恰好构成整条语句时才按
        // 表达式处理，其余原文保全。
        if self.at_kw("print") && self.at_op_at(1, "(") && !self.print_call_is_whole_stmt() {
            let s = self.pos;
            return Ok(self.capture_line_from(s));
        }
        // `print -1 + 2` 在 Python 3 是合法的二元表达式（运行时 TypeError），
        // 折叠成 `print 1` 会把合法程序变成语法错误；整行原文保全。
        if self.at_kw("print") && (self.at_op_at(1, "-") || self.at_op_at(1, "+")) {
            let s = self.pos;
            return Ok(self.capture_line_from(s));
        }
        if self.at_kw("print") && self.looks_like_print_stmt() {
            return self.parse_print_stmt();
        }
        // `exec` 在 Python 3 可以是普通名字；无论哪种形态，整行原文
        // 保全都保持原版本语义（Python 2 的 exec 语句与 Python 3 的
        // exec() 调用在参数逗号处语义不同）。
        if self.at_kw("exec") {
            let s = self.pos;
            return Ok(self.capture_line_from(s));
        }
        if self.at_kw("pass") {
            self.advance();
            return Ok(self.ast.simple(NodeData::Pass));
        }
        if self.at_kw("break") {
            self.advance();
            return Ok(self.ast.simple(NodeData::Break));
        }
        if self.at_kw("continue") {
            self.advance();
            return Ok(self.ast.simple(NodeData::Continue));
        }
        if self.at_kw("return") {
            self.advance();
            let value = if self.at_stmt_end() {
                None
            } else {
                Some(self.parse_testlist_star()?)
            };
            return Ok(self.ast.return_(value));
        }
        if self.at_kw("raise") {
            let raise_start = self.pos;
            self.advance();
            let exc = if self.at_stmt_end() || self.at_kw("from") {
                None
            } else {
                Some(self.parse_test()?)
            };
            // Python 2: raise E, V[, TB] —— 逗号形式整体保留原文
            if self.at_op(",") {
                return Ok(self.capture_line_from(raise_start));
            }
            let cause = if self.eat_kw("from") {
                Some(self.parse_test()?)
            } else {
                None
            };
            return Ok(self.ast.raise_(exc, cause));
        }
        if self.at_kw("assert") {
            self.advance();
            let test = self.parse_test()?;
            let msg = if self.eat_op(",") {
                Some(self.parse_test()?)
            } else {
                None
            };
            return Ok(self.ast.assert_(test, msg));
        }
        if self.at_kw("del") {
            self.advance();
            let mut targets = vec![self.parse_named_expr()?];
            while self.eat_op(",") {
                if self.at_stmt_end() {
                    break;
                }
                targets.push(self.parse_named_expr()?);
            }
            return Ok(self.ast.push(NodeData::Delete, targets));
        }
        if self.at_kw("lazy") && (self.at_kw_at(1, "import") || self.at_kw_at(1, "from")) {
            let s = self.pos;
            self.skip_to_newline();
            let raw = self.slice_tokens(s, self.pos).trim().to_string();
            return Ok(self.ast.simple(NodeData::Import(raw)));
        }
        if self.at_kw("import") {
            let s = self.pos;
            self.skip_to_newline();
            let raw = self.slice_tokens(s, self.pos).trim().to_string();
            return Ok(self.ast.simple(NodeData::Import(raw)));
        }
        if self.at_kw("from") {
            let s = self.pos;
            self.skip_to_newline();
            let raw = self.slice_tokens(s, self.pos).trim().to_string();
            return Ok(self.ast.simple(NodeData::FromImport(raw)));
        }
        if self.at_kw("global") || self.at_kw("nonlocal") {
            let kw = self.advance().text;
            let s = self.pos;
            self.skip_to_newline();
            let raw = self.slice_tokens(s, self.pos).trim().to_string();
            return Ok(self.ast.simple(if kw == "global" {
                NodeData::Global(raw)
            } else {
                NodeData::Nonlocal(raw)
            }));
        }
        if self.at_kw("yield") {
            let y = self.parse_yield()?;
            return Ok(self.ast.expr_stmt(y));
        }
        let first_expr = self.parse_testlist_star()?;
        if self.at_op(":") {
            self.advance();
            let ann_start = self.pos;
            // 注解里可以有 `Field(min_length=1)` 这类带 `=` 的下标参数；
            // 只有深度 0 的 `=` / `;` / 换行才结束注解。
            let mut depth = 0i32;
            while !self.at_eof() {
                let t = self.tk();
                if t.kind == TokKind::Op {
                    match t.text.as_str() {
                        "(" | "[" | "{" => depth += 1,
                        ")" | "]" | "}" => depth -= 1,
                        "=" | ";" if depth <= 0 => break,
                        _ => {}
                    }
                }
                if t.kind == TokKind::Newline && depth <= 0 {
                    break;
                }
                self.advance();
            }
            let annotation = self.slice_tokens(ann_start, self.pos).trim().to_string();
            let value = if self.eat_op("=") {
                Some(self.parse_testlist_star()?)
            } else {
                None
            };
            return Ok(self.ast.ann_assign(first_expr, &annotation, value));
        }
        if let Some(op) = self.aug_assign_op() {
            self.advance();
            let value = self.parse_testlist_star()?;
            return Ok(self.ast.aug_assign(op, first_expr, value));
        }
        if self.eat_op("=") {
            let mut targets = vec![first_expr];
            loop {
                let rhs = self.parse_testlist_star()?;
                if self.eat_op("=") {
                    targets.push(rhs);
                    continue;
                }
                return Ok(self.ast.assign(targets, rhs));
            }
        }
        Ok(self.ast.expr_stmt(first_expr))
    }

    fn aug_assign_op(&self) -> Option<PyBin> {
        if self.tk().kind != TokKind::Op {
            return None;
        }
        Some(match self.tk().text.as_str() {
            "+=" => PyBin::Add,
            "-=" => PyBin::Sub,
            "*=" => PyBin::Mul,
            "/=" => PyBin::Div,
            "//=" => PyBin::FloorDiv,
            "%=" => PyBin::Mod,
            "**=" => PyBin::Pow,
            "@=" => PyBin::MatMul,
            "<<=" => PyBin::LShift,
            ">>=" => PyBin::RShift,
            "&=" => PyBin::BitAnd,
            "|=" => PyBin::BitOr,
            "^=" => PyBin::BitXor,
            _ => return None,
        })
    }

    fn at_stmt_end(&self) -> bool {
        self.at_eof()
            || self.tk().kind == TokKind::Newline
            || self.at_op(";")
            || self.at_op(")")
            || self.at_op("]")
            || self.at_op("}")
            || self.at_op("=")
            || self.at_op(":=")
            || self.at_op(":")
    }

    fn skip_to_newline(&mut self) {
        let mut depth = 0i32;
        while !self.at_eof() {
            let t = self.tk().clone();
            if t.kind == TokKind::Op {
                match t.text.as_str() {
                    "(" | "[" | "{" => depth += 1,
                    ")" | "]" | "}" => depth -= 1,
                    _ => {}
                }
            }
            if t.kind == TokKind::Newline && depth <= 0 {
                return;
            }
            self.advance();
        }
    }

    fn slice_tokens(&self, a: usize, b: usize) -> String {
        if b <= a || a >= self.toks.len() {
            return String::new();
        }
        let s = self.toks[a].start as usize;
        let e = self.toks[b - 1].end as usize;
        self.src.get(s..e).unwrap_or("").to_string()
    }

    fn scan_to_op(&mut self, op: &str) -> R<usize> {
        let mut depth = 0i32;
        loop {
            if self.at_eof() {
                return Err(format!("expected `{op}` before EOF"));
            }
            let t = self.tk().clone();
            if t.kind == TokKind::Op {
                match t.text.as_str() {
                    "(" | "[" | "{" => depth += 1,
                    ")" | "]" | "}" => depth -= 1,
                    _ => {}
                }
                if depth == 0 && t.text == op {
                    return Ok(self.pos);
                }
            }
            if t.kind == TokKind::Newline && depth <= 0 {
                return Err(format!("expected `{op}` before newline"));
            }
            self.advance();
        }
    }

    fn scan_to_kw(&mut self, kw: &str) -> R<usize> {
        let mut depth = 0i32;
        loop {
            if self.at_eof() || (self.tk().kind == TokKind::Newline && depth <= 0) {
                return Err(format!("expected `{kw}`"));
            }
            let t = self.tk().clone();
            if t.kind == TokKind::Op {
                match t.text.as_str() {
                    "(" | "[" | "{" => depth += 1,
                    ")" | "]" | "}" => depth -= 1,
                    _ => {}
                }
            }
            if depth == 0 && t.kind == TokKind::Name && t.text == kw {
                return Ok(self.pos);
            }
            self.advance();
        }
    }

    fn parse_block(&mut self) -> R<PyId> {
        if self.tk().kind == TokKind::Newline {
            self.advance();
            if self.tk().kind != TokKind::Indent {
                return Err("expected an indented block".into());
            }
            let ind = self.advance().text;
            self.indent_stack.push(ind);
            let mut stmts = Vec::new();
            while !self.at_eof() && self.tk().kind != TokKind::Dedent {
                let before = self.pos;
                self.parse_stmt_into(&mut stmts);
                if self.pos == before {
                    self.advance();
                }
            }
            if self.tk().kind == TokKind::Dedent {
                self.advance();
            }
            self.indent_stack.pop();
            Ok(self.ast.block(stmts))
        } else {
            let mut stmts = Vec::new();
            self.parse_simple_line(&mut stmts)?;
            Ok(self.ast.block(stmts))
        }
    }

    fn parse_function(&mut self, is_async: bool, deco_raw: &str) -> R<PyId> {
        self.expect_kw("def")?;
        let name = self.expect_name()?;
        let mut generics_raw = String::new();
        if self.at_op("[") {
            let s = self.pos;
            self.skip_balanced("[", "]")?;
            generics_raw = self.slice_tokens(s, self.pos).trim().to_string();
        }
        let open = self.pos;
        self.skip_balanced("(", ")")?;
        let params_raw = self.inner_parens(open, self.pos);
        let mut ret_raw = String::new();
        if self.eat_op("->") {
            let s = self.pos;
            self.scan_to_op(":")?;
            ret_raw = self.slice_tokens(s, self.pos).trim().to_string();
        }
        self.expect_op(":")?;
        let body = self.parse_block()?;
        Ok(self.ast.function(
            &name,
            &params_raw,
            &ret_raw,
            deco_raw,
            &generics_raw,
            is_async,
            body,
        ))
    }

    fn parse_class(&mut self, deco_raw: &str) -> R<PyId> {
        self.expect_kw("class")?;
        let name = self.expect_name()?;
        let mut header = String::new();
        if self.at_op("[") {
            let s = self.pos;
            self.skip_balanced("[", "]")?;
            header.push_str(&self.slice_tokens(s, self.pos));
        }
        if self.at_op("(") {
            let s = self.pos;
            self.skip_balanced("(", ")")?;
            header.push_str(&self.slice_tokens(s, self.pos));
        }
        self.expect_op(":")?;
        let body = self.parse_block()?;
        Ok(self.ast.class(&name, header.trim(), deco_raw, body))
    }

    fn parse_if_chain(&mut self) -> R<PyId> {
        if !(self.eat_kw("if") || self.eat_kw("elif")) {
            return Err("expected if/elif".into());
        }
        let test = self.parse_named_expr()?;
        self.expect_op(":")?;
        let body = self.parse_block()?;
        let orelse = if self.at_kw("elif") {
            // 约定：If 的第 3 个孩子是 Block；elif 链装成单元素 Block，
            // 打印层据此输出 `elif`（直接放 If 会被当语句列表）。
            let nested = self.parse_if_chain()?;
            Some(self.ast.block(vec![nested]))
        } else if self.eat_kw("else") {
            self.expect_op(":")?;
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(self.ast.if_(test, body, orelse))
    }

    fn parse_while(&mut self) -> R<PyId> {
        self.expect_kw("while")?;
        let test = self.parse_named_expr()?;
        self.expect_op(":")?;
        let body = self.parse_block()?;
        let orelse = if self.eat_kw("else") {
            self.expect_op(":")?;
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(self.ast.while_(test, body, orelse))
    }

    fn parse_for(&mut self, is_async: bool) -> R<PyId> {
        self.expect_kw("for")?;
        let tstart = self.pos;
        let in_pos = self.scan_to_kw("in")?;
        let target_raw = self.slice_tokens(tstart, in_pos).trim().to_string();
        self.advance();
        let iter = self.parse_testlist_star()?;
        self.expect_op(":")?;
        let body = self.parse_block()?;
        let orelse = if self.eat_kw("else") {
            self.expect_op(":")?;
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(self.ast.for_(&target_raw, iter, body, orelse, is_async))
    }

    fn parse_try(&mut self) -> R<PyId> {
        self.expect_kw("try")?;
        self.expect_op(":")?;
        let body = self.parse_block()?;
        let mut handlers = Vec::new();
        let mut star = false;
        while self.at_kw("except") {
            let s = self.pos;
            self.advance();
            if self.at_op("*") {
                star = true;
                self.advance();
            }
            self.scan_to_op(":")?;
            let clause = self.slice_tokens(s, self.pos).trim().to_string();
            self.expect_op(":")?;
            let hbody = self.parse_block()?;
            handlers.push(self.ast.except_handler(&clause, hbody));
        }
        let orelse = if self.eat_kw("else") {
            self.expect_op(":")?;
            Some(self.parse_block()?)
        } else {
            None
        };
        let finally = if self.eat_kw("finally") {
            self.expect_op(":")?;
            Some(self.parse_block()?)
        } else {
            None
        };
        if handlers.is_empty() && finally.is_none() {
            return Err("try without except/finally".into());
        }
        Ok(self.ast.try_(star, body, handlers, orelse, finally))
    }

    fn parse_with(&mut self, is_async: bool) -> R<PyId> {
        self.expect_kw("with")?;
        let s = self.pos;
        self.scan_to_op(":")?;
        let items = self.slice_tokens(s, self.pos).trim().to_string();
        self.expect_op(":")?;
        let body = self.parse_block()?;
        Ok(self.ast.with_(&items, is_async, body))
    }

    /// `match` 是软关键字：`match = re.match(...)`、`match(x)` 是普通名。
    /// 只有头部在深度 0 出现 `:`（且不是 `:=`）才按 match 语句解析。
    fn looks_like_match_stmt(&self) -> bool {
        let mut i = self.pos + 1;
        let mut depth = 0i32;
        while i < self.toks.len() {
            let t = &self.toks[i];
            match t.kind {
                TokKind::Eof | TokKind::Newline if depth <= 0 => return false,
                TokKind::Op => match t.text.as_str() {
                    "(" | "[" | "{" => depth += 1,
                    ")" | "]" | "}" => {
                        depth -= 1;
                        if depth < 0 {
                            return false;
                        }
                    }
                    ":" if depth == 0 => {
                        // `match: T = ...` 是带注解的变量；match 语句的
                        // subject 至少有一个 token（`match w := x,:` 合法）
                        return i > self.pos + 1;
                    }
                    "=" if depth == 0 => return false,
                    _ => {}
                },
                _ => {}
            }
            i += 1;
        }
        false
    }

    fn parse_match(&mut self) -> R<PyId> {
        self.expect_kw("match")?;
        let subject = self.parse_testlist_star()?;
        self.expect_op(":")?;
        let mut cases = Vec::new();
        if self.tk().kind == TokKind::Newline {
            self.advance();
        }
        if self.tk().kind != TokKind::Indent {
            return Err("expected match cases".into());
        }
        let ind = self.advance().text;
        self.indent_stack.push(ind);
        while !self.at_eof() && self.tk().kind != TokKind::Dedent {
            if !self.at_kw("case") {
                return Err("expected case".into());
            }
            cases.push(self.parse_case()?);
        }
        if self.tk().kind == TokKind::Dedent {
            self.advance();
        }
        self.indent_stack.pop();
        Ok(self.ast.match_(subject, cases))
    }

    fn parse_case(&mut self) -> R<PyId> {
        self.expect_kw("case")?;
        let s = self.pos;
        let mut depth = 0i32;
        let mut guard_pos = None;
        loop {
            if self.at_eof() {
                return Err("unterminated case".into());
            }
            let t = self.tk().clone();
            if t.kind == TokKind::Op {
                match t.text.as_str() {
                    "(" | "[" | "{" => depth += 1,
                    ")" | "]" | "}" => depth -= 1,
                    ":" if depth == 0 => break,
                    _ => {}
                }
            }
            if depth == 0 && t.kind == TokKind::Name && t.text == "if" {
                guard_pos = Some(self.pos);
                break;
            }
            self.advance();
        }
        let pend = guard_pos.unwrap_or(self.pos);
        let pattern = self.slice_tokens(s, pend).trim().to_string();
        let guard = if guard_pos.is_some() {
            self.expect_kw("if")?;
            Some(self.parse_named_expr()?)
        } else {
            None
        };
        self.expect_op(":")?;
        let body = self.parse_block()?;
        Ok(self.ast.case(&pattern, guard, body))
    }

    fn parse_testlist_star(&mut self) -> R<PyId> {
        let mut v = vec![self.parse_starred_test()?];
        let mut trailing_comma = false;
        while self.eat_op(",") {
            trailing_comma = true;
            if self.at_stmt_end() || self.at_op(",") {
                break;
            }
            v.push(self.parse_starred_test()?);
            trailing_comma = false;
        }
        if v.len() == 1 && !trailing_comma {
            Ok(v.pop().unwrap())
        } else {
            Ok(self.ast.push(NodeData::Tuple, v))
        }
    }

    fn parse_starred_test(&mut self) -> R<PyId> {
        if self.eat_op("*") {
            let e = self.parse_bit_or()?;
            return Ok(self.ast.push(NodeData::Starred, vec![e]));
        }
        self.parse_named_expr()
    }

    fn parse_named_expr(&mut self) -> R<PyId> {
        let t = self.parse_test()?;
        if self.eat_op(":=") {
            let name = match self.ast.data(t) {
                NodeData::Name(n) => n.clone(),
                _ => return Err("walrus target must be a name".into()),
            };
            let v = self.parse_test()?;
            return Ok(self.ast.push(NodeData::NamedExpr { name }, vec![v]));
        }
        Ok(t)
    }

    fn parse_test(&mut self) -> R<PyId> {
        self.depth += 1;
        if self.depth > cure_python_ast::MAX_NEST {
            self.depth -= 1;
            return Err("nesting too deep".into());
        }
        let out = self.parse_test_inner();
        self.depth -= 1;
        out
    }

    fn parse_test_inner(&mut self) -> R<PyId> {
        if self.at_kw("lambda") {
            self.advance();
            let s = self.pos;
            // 默认值里可以再嵌 lambda（`lambda x=lambda y: y: x()`）：
            // 不能扫到第一个 `:` 就停，必须按参数表结构解析。
            loop {
                if self.at_op(":") {
                    break;
                }
                if self.eat_op("*") {
                    if self.tk().kind == TokKind::Name {
                        self.advance();
                    }
                } else if self.eat_op("**") {
                    self.expect_name()?;
                } else if self.eat_op("/") {
                    // positional-only marker
                } else if self.at_op("(") {
                    // Python 2 tuple parameter: `lambda (a, b): ...`
                    self.skip_balanced("(", ")")?;
                    if self.eat_op("=") {
                        let _ = self.parse_test()?;
                    }
                } else {
                    self.expect_name()?;
                    if self.eat_op("=") {
                        let _ = self.parse_test()?;
                    }
                }
                if self.eat_op(",") {
                    continue;
                }
                break;
            }
            let params = self.slice_tokens(s, self.pos).trim().to_string();
            self.expect_op(":")?;
            let no_cond = std::mem::take(&mut self.no_cond_lambda);
            let body = if no_cond {
                self.parse_or()?
            } else {
                self.parse_test()?
            };
            return Ok(self
                .ast
                .push(NodeData::Lambda { params_raw: params }, vec![body]));
        }
        if self.at_kw("yield") {
            return self.parse_yield();
        }
        let body = self.parse_or()?;
        if self.eat_kw("if") {
            let cond = self.parse_or()?;
            self.expect_kw("else")?;
            let orelse = self.parse_test()?;
            return Ok(self.ast.if_exp(body, cond, orelse));
        }
        Ok(body)
    }

    fn parse_comp_iter_item(&mut self) -> R<PyId> {
        if self.at_kw("lambda") {
            self.no_cond_lambda = true;
            let r = self.parse_test();
            self.no_cond_lambda = false;
            r
        } else {
            self.parse_or()
        }
    }

    fn parse_comp_ifs(&mut self) -> R<Vec<PyId>> {
        let mut v = Vec::new();
        while self.eat_kw("if") {
            v.push(self.parse_or()?);
        }
        Ok(v)
    }

    fn parse_or(&mut self) -> R<PyId> {
        let first = self.parse_and()?;
        if !self.at_kw("or") {
            return Ok(first);
        }
        let mut vals = vec![first];
        while self.eat_kw("or") {
            vals.push(self.parse_and()?);
        }
        Ok(self.ast.bool_op(PyBoolOp::Or, vals))
    }

    fn parse_and(&mut self) -> R<PyId> {
        let first = self.parse_not()?;
        if !self.at_kw("and") {
            return Ok(first);
        }
        let mut vals = vec![first];
        while self.eat_kw("and") {
            vals.push(self.parse_not()?);
        }
        Ok(self.ast.bool_op(PyBoolOp::And, vals))
    }

    fn parse_not(&mut self) -> R<PyId> {
        if self.eat_kw("not") {
            let e = self.parse_not()?;
            return Ok(self.ast.unary(PyUn::Not, e));
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> R<PyId> {
        let first = self.parse_bit_or()?;
        let mut operands = vec![first];
        let mut ops = Vec::new();
        loop {
            let op = if self.eat_op("==") {
                PyCmp::Eq
            } else if self.eat_op("!=") {
                PyCmp::NotEq
            } else if self.eat_op("<>") {
                PyCmp::LtGt
            } else if self.eat_op("<=") {
                PyCmp::LtE
            } else if self.eat_op(">=") {
                PyCmp::GtE
            } else if self.eat_op("<") {
                PyCmp::Lt
            } else if self.eat_op(">") {
                PyCmp::Gt
            } else if self.at_kw("is") {
                self.advance();
                if self.eat_kw("not") {
                    PyCmp::IsNot
                } else {
                    PyCmp::Is
                }
            } else if self.at_kw("in") {
                self.advance();
                PyCmp::In
            } else if self.at_kw("not") && self.at_kw_at(1, "in") {
                self.advance();
                self.advance();
                PyCmp::NotIn
            } else {
                break;
            };
            ops.push(op);
            operands.push(self.parse_bit_or()?);
        }
        if ops.is_empty() {
            Ok(operands.pop().unwrap())
        } else {
            Ok(self.ast.compare(operands, ops))
        }
    }

    fn parse_bit_or(&mut self) -> R<PyId> {
        self.parse_left_assoc(Self::parse_bit_xor, &[("|", PyBin::BitOr)])
    }
    fn parse_bit_xor(&mut self) -> R<PyId> {
        self.parse_left_assoc(Self::parse_bit_and, &[("^", PyBin::BitXor)])
    }
    fn parse_bit_and(&mut self) -> R<PyId> {
        self.parse_left_assoc(Self::parse_shift, &[("&", PyBin::BitAnd)])
    }
    fn parse_shift(&mut self) -> R<PyId> {
        self.parse_left_assoc(
            Self::parse_add,
            &[("<<", PyBin::LShift), (">>", PyBin::RShift)],
        )
    }
    fn parse_add(&mut self) -> R<PyId> {
        self.parse_left_assoc(Self::parse_mul, &[("+", PyBin::Add), ("-", PyBin::Sub)])
    }
    fn parse_mul(&mut self) -> R<PyId> {
        self.parse_left_assoc(
            Self::parse_factor,
            &[
                ("*", PyBin::Mul),
                ("@", PyBin::MatMul),
                ("/", PyBin::Div),
                ("//", PyBin::FloorDiv),
                ("%", PyBin::Mod),
            ],
        )
    }

    fn parse_left_assoc(
        &mut self,
        next: fn(&mut Self) -> R<PyId>,
        ops: &[(&str, PyBin)],
    ) -> R<PyId> {
        let mut left = next(self)?;
        loop {
            let mut hit = None;
            for &(tok, op) in ops {
                if self.at_op(tok) {
                    hit = Some(op);
                    break;
                }
            }
            let Some(op) = hit else { break };
            self.advance();
            let right = next(self)?;
            left = self.ast.binary(op, left, right);
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> R<PyId> {
        if self.eat_op("+") {
            let e = self.parse_factor()?;
            return Ok(self.ast.unary(PyUn::Plus, e));
        }
        if self.eat_op("-") {
            let e = self.parse_factor()?;
            return Ok(self.ast.unary(PyUn::Minus, e));
        }
        if self.eat_op("~") {
            let e = self.parse_factor()?;
            return Ok(self.ast.unary(PyUn::Invert, e));
        }
        self.parse_power()
    }

    fn parse_power(&mut self) -> R<PyId> {
        let base = self.parse_await_primary()?;
        if self.eat_op("**") {
            let exp = self.parse_factor()?;
            return Ok(self.ast.binary(PyBin::Pow, base, exp));
        }
        Ok(base)
    }

    fn parse_await_primary(&mut self) -> R<PyId> {
        if self.eat_kw("await") {
            let e = self.parse_await_primary()?;
            return Ok(self.ast.push(NodeData::Await, vec![e]));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> R<PyId> {
        self.depth += 1;
        if self.depth > cure_python_ast::MAX_NEST {
            self.depth -= 1;
            return Err("nesting too deep".into());
        }
        let mut e = self.parse_atom()?;
        loop {
            if self.at_op("(") {
                e = self.parse_call(e)?;
            } else if self.eat_op(".") {
                let name = self.expect_name()?;
                e = self.ast.attribute(e, &name);
            } else if self.at_op("[") {
                e = self.parse_subscript(e)?;
            } else {
                break;
            }
        }
        self.depth -= 1;
        Ok(e)
    }

    fn parse_call(&mut self, func: PyId) -> R<PyId> {
        self.expect_op("(")?;
        let mut args = Vec::new();
        let mut kinds = Vec::new();
        while !self.at_op(")") {
            if self.at_eof() {
                return Err("unterminated call".into());
            }
            if self.eat_op("**") {
                let e = self.parse_test()?;
                args.push(self.ast.push(NodeData::DoubleStar, vec![e]));
                kinds.push(CallArg::kwstar());
            } else if self.eat_op("*") {
                let e = self.parse_test()?;
                args.push(self.ast.push(NodeData::Starred, vec![e]));
                kinds.push(CallArg::star());
            } else if self.tk().kind == TokKind::Name
                && self.tk_at(1).kind == TokKind::Op
                && self.tk_at(1).text == "="
            {
                let name = self.advance().text;
                self.advance();
                let e = self.parse_test()?;
                args.push(e);
                kinds.push(CallArg::kw(name));
            } else {
                let e = self.parse_named_expr()?;
                if self.at_kw("for") || (self.at_kw("async") && self.at_kw_at(1, "for")) {
                    let comp = self.parse_comp(e, CompKind::Gen, None)?;
                    args.push(comp);
                } else {
                    args.push(e);
                }
                kinds.push(CallArg::pos());
            }
            if !self.eat_op(",") {
                break;
            }
        }
        self.expect_op(")")?;
        Ok(self.ast.call(func, args, kinds))
    }

    fn parse_subscript(&mut self, base: PyId) -> R<PyId> {
        self.expect_op("[")?;
        let mut items = Vec::new();
        let mut trailing_comma = false;
        loop {
            if self.at_op("]") {
                break;
            }
            let lower = if self.at_op(":") || self.at_op(",") {
                None
            } else if self.eat_op("*") {
                let e = self.parse_named_expr()?;
                Some(self.ast.push(NodeData::Starred, vec![e]))
            } else {
                Some(self.parse_named_expr()?)
            };
            if self.eat_op(":") {
                let upper = if self.at_op(":") || self.at_op(",") || self.at_op("]") {
                    None
                } else {
                    Some(self.parse_named_expr()?)
                };
                let step_colon = self.eat_op(":");
                let step = if step_colon {
                    if self.at_op(",") || self.at_op("]") {
                        None
                    } else {
                        Some(self.parse_named_expr()?)
                    }
                } else {
                    None
                };
                let s = self.ast.slice(lower, upper, step, step_colon);
                items.push(s);
            } else if let Some(l) = lower {
                items.push(l);
            } else {
                return Err("empty subscript item".into());
            }
            if !self.eat_op(",") {
                break;
            }
            trailing_comma = true;
            if self.at_op("]") {
                break;
            }
            trailing_comma = false;
        }
        self.expect_op("]")?;
        let idx = if items.len() == 1 && !trailing_comma {
            items.pop().unwrap()
        } else {
            self.ast.push(NodeData::Tuple, items)
        };
        Ok(self.ast.subscript(base, idx))
    }

    fn parse_comp(&mut self, elt: PyId, kind: CompKind, value: Option<PyId>) -> R<PyId> {
        let mut compfors = Vec::new();
        loop {
            let is_async = if self.at_kw("async") && self.at_kw_at(1, "for") {
                self.advance();
                true
            } else {
                false
            };
            if !self.at_kw("for") {
                break;
            }
            self.advance();
            let tstart = self.pos;
            let in_pos = self.scan_to_kw("in")?;
            let target_raw = self.slice_tokens(tstart, in_pos).trim().to_string();
            self.advance();
            // Python 2 允许推导式 iterable 是未加括号的 tuple：
            // `[x for x in a, b]`（Python 3 已移除）。
            let first = self.parse_comp_iter_item()?;
            let mut items = vec![first];
            while self.eat_op(",") {
                if self.at_kw("for")
                    || self.at_kw("if")
                    || self.at_op("]")
                    || self.at_op(")")
                    || self.at_op("}")
                    || self.tk().kind == TokKind::Newline
                {
                    break;
                }
                items.push(self.parse_comp_iter_item()?);
            }
            let iter = if items.len() == 1 {
                items.pop().unwrap()
            } else {
                self.ast.push(NodeData::Tuple, items)
            };
            let ifs = self.parse_comp_ifs()?;
            let if_count = ifs.len();
            let mut ch = vec![iter];
            ch.extend(ifs);
            compfors.push(self.ast.push(
                NodeData::CompFor {
                    target_raw,
                    is_async,
                    if_count,
                },
                ch,
            ));
        }
        if compfors.is_empty() {
            return Err("expected comprehension for".into());
        }
        let mut ch = Vec::new();
        if let Some(v) = value {
            ch.push(elt);
            ch.push(v);
        } else {
            ch.push(elt);
        }
        ch.extend(compfors);
        Ok(self.ast.push(NodeData::Comp { kind }, ch))
    }

    fn parse_atom(&mut self) -> R<PyId> {
        let t = self.tk().clone();
        match t.kind {
            TokKind::Name => {
                self.advance();
                match t.text.as_str() {
                    "True" if !self.bool_names => Ok(self.ast.bool(true)),
                    "False" if !self.bool_names => Ok(self.ast.bool(false)),
                    "None" => Ok(self.ast.none()),
                    _ => Ok(self.ast.name(&t.text)),
                }
            }
            TokKind::Number => {
                self.advance();
                Ok(self.ast.literal(number_literal(&t.text)))
            }
            TokKind::Str => {
                let s = self.pos;
                let mut any_f = false;
                let mut any_b = false;
                while self.tk().kind == TokKind::Str {
                    let p = string_prefix(&self.tk().text);
                    any_f |= p.contains(['f', 'F', 't', 'T']);
                    any_b |= p.contains(['b', 'B']);
                    self.advance();
                }
                let raw = self.slice_tokens(s, self.pos);
                if any_f {
                    Ok(self.ast.raw(&raw))
                } else {
                    let kind = if any_b { LitKind::Bytes } else { LitKind::Str };
                    Ok(self.ast.literal(Literal::raw_str(raw, kind)))
                }
            }
            TokKind::Op => match t.text.as_str() {
                "..." => {
                    self.advance();
                    Ok(self.ast.literal(Literal::ellipsis()))
                }
                "(" => self.parse_paren(),
                "[" => self.parse_list(),
                "{" => self.parse_brace(),
                "`" => {
                    self.advance();
                    let e = self.parse_testlist_star()?;
                    self.expect_op("`")?;
                    Ok(self.ast.repr_(e))
                }
                _ => Err(format!("unexpected `{}`", t.text)),
            },
            _ => Err(format!("unexpected token `{}`", t.text)),
        }
    }

    fn parse_paren(&mut self) -> R<PyId> {
        self.expect_op("(")?;
        if self.eat_op(")") {
            return Ok(self.ast.push(NodeData::Tuple, vec![]));
        }
        if self.at_kw("yield") {
            let y = self.parse_yield()?;
            self.expect_op(")")?;
            return Ok(self.ast.paren(y));
        }
        let first = self.parse_starred_test()?;
        if self.at_kw("for") || (self.at_kw("async") && self.at_kw_at(1, "for")) {
            let comp = self.parse_comp(first, CompKind::Gen, None)?;
            self.expect_op(")")?;
            return Ok(comp);
        }
        let inner = from_paren_list(self, first, ")")?;
        if matches!(self.ast.data(inner), NodeData::Tuple) {
            Ok(inner)
        } else {
            Ok(self.ast.paren(inner))
        }
    }

    fn parse_list(&mut self) -> R<PyId> {
        self.expect_op("[")?;
        if self.eat_op("]") {
            return Ok(self.ast.push(NodeData::List, vec![]));
        }
        let first = self.parse_starred_test()?;
        if self.at_kw("for") || (self.at_kw("async") && self.at_kw_at(1, "for")) {
            let comp = self.parse_comp(first, CompKind::List, None)?;
            self.expect_op("]")?;
            return Ok(comp);
        }
        let mut items = vec![first];
        while self.eat_op(",") {
            if self.at_op("]") {
                break;
            }
            items.push(self.parse_starred_test()?);
        }
        self.expect_op("]")?;
        Ok(self.ast.push(NodeData::List, items))
    }

    fn parse_brace(&mut self) -> R<PyId> {
        self.expect_op("{")?;
        if self.eat_op("}") {
            return Ok(self.ast.push(NodeData::Dict, vec![]));
        }
        if self.eat_op("**") {
            let v = self.parse_test()?;
            let mut items = vec![self.ast.push(NodeData::DoubleStar, vec![v])];
            while self.eat_op(",") {
                if self.at_op("}") {
                    break;
                }
                if self.eat_op("**") {
                    let v = self.parse_test()?;
                    items.push(self.ast.push(NodeData::DoubleStar, vec![v]));
                } else {
                    let k = self.parse_test()?;
                    self.expect_op(":")?;
                    let v = self.parse_test()?;
                    items.push(k);
                    items.push(v);
                }
            }
            self.expect_op("}")?;
            return Ok(self.ast.push(NodeData::Dict, items));
        }
        let first = self.parse_starred_test()?;
        if self.eat_op(":") {
            let value = self.parse_test()?;
            if self.at_kw("for") || (self.at_kw("async") && self.at_kw_at(1, "for")) {
                let comp = self.parse_comp(first, CompKind::Dict, Some(value))?;
                self.expect_op("}")?;
                return Ok(comp);
            }
            let mut items = vec![first, value];
            while self.eat_op(",") {
                if self.at_op("}") {
                    break;
                }
                if self.eat_op("**") {
                    let v = self.parse_test()?;
                    items.push(self.ast.push(NodeData::DoubleStar, vec![v]));
                } else {
                    let k = self.parse_test()?;
                    self.expect_op(":")?;
                    let v = self.parse_test()?;
                    items.push(k);
                    items.push(v);
                }
            }
            self.expect_op("}")?;
            Ok(self.ast.push(NodeData::Dict, items))
        } else if self.at_kw("for") || (self.at_kw("async") && self.at_kw_at(1, "for")) {
            let comp = self.parse_comp(first, CompKind::Set, None)?;
            self.expect_op("}")?;
            Ok(comp)
        } else {
            let mut items = vec![first];
            while self.eat_op(",") {
                if self.at_op("}") {
                    break;
                }
                items.push(self.parse_starred_test()?);
            }
            self.expect_op("}")?;
            Ok(self.ast.push(NodeData::Set, items))
        }
    }

    fn parse_yield(&mut self) -> R<PyId> {
        self.expect_kw("yield")?;
        if self.eat_kw("from") {
            let value = self.parse_test()?;
            return Ok(self.ast.push(NodeData::Yield { from: true }, vec![value]));
        }
        if self.at_stmt_end() || self.at_kw("else") {
            return Ok(self.ast.push(NodeData::Yield { from: false }, vec![]));
        }
        let value = self.parse_testlist_star()?;
        Ok(self.ast.push(NodeData::Yield { from: false }, vec![value]))
    }

    fn skip_balanced(&mut self, open: &str, close: &str) -> R<()> {
        self.expect_op(open)?;
        let mut depth = 1i32;
        while depth > 0 {
            if self.at_eof() {
                return Err(format!("unterminated `{open}`"));
            }
            let t = self.advance();
            if t.kind == TokKind::Op {
                if t.text == open {
                    depth += 1;
                } else if t.text == close {
                    depth -= 1;
                }
            }
        }
        Ok(())
    }

    fn inner_parens(&self, open: usize, end: usize) -> String {
        if end <= open + 1 {
            return String::new();
        }
        let s = self.toks[open].end as usize;
        let e = self.toks[end - 1].start as usize;
        self.src.get(s..e).unwrap_or("").to_string()
    }

    /// Python 2 print 是语句；`print(...)`/`print[0]` 这类在 Python 3
    /// 合法（或打印对象相同）的形态仍按表达式解析，打印结果不变。
    fn looks_like_print_stmt(&self) -> bool {
        let t = self.tk_at(1);
        match t.kind {
            TokKind::Newline | TokKind::Eof | TokKind::Name | TokKind::Number | TokKind::Str => {
                true
            }
            TokKind::Op => match t.text.as_str() {
                ">>" | ";" | "-" | "+" | "~" | "`" => true,
                // `print [1]`（有空格）是 print 语句；紧邻的 `print[1]`
                // 已在 parse_small_stmt 里按原文保全。
                "[" | "{" => !self.next_token_adjacent(),
                _ => false,
            },
            _ => false,
        }
    }

    /// `print(...)` 之后是否立刻结束当前简单语句（只有这种形态在
    /// Python 2/3 下语义一致，可以安全地当调用处理）。
    fn print_call_is_whole_stmt(&self) -> bool {
        let mut i = self.pos + 1;
        let mut depth = 0i32;
        while i < self.toks.len() {
            let t = &self.toks[i];
            match t.kind {
                TokKind::Eof => return false,
                TokKind::Op => match t.text.as_str() {
                    "(" => depth += 1,
                    ")" => {
                        depth -= 1;
                        if depth == 0 {
                            let Some(next) = self.toks.get(i + 1) else {
                                return true;
                            };
                            return next.kind == TokKind::Newline
                                || next.kind == TokKind::Eof
                                || next.kind == TokKind::Dedent
                                || (next.kind == TokKind::Op && next.text == ";");
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
            i += 1;
        }
        false
    }

    /// 当前 token 与下一个 token 之间没有空白（用于区分 Python 2 语句
    /// 与 Python 3 的下标/调用形态）。
    fn next_token_adjacent(&self) -> bool {
        self.pos + 1 < self.toks.len()
            && self.toks[self.pos].end == self.toks[self.pos + 1].start
    }

    fn parse_print_stmt(&mut self) -> R<PyId> {
        self.expect_kw("print")?;
        let mut dest = None;
        if self.eat_op(">>") {
            dest = Some(self.parse_test()?);
        }
        let mut values = Vec::new();
        let mut nl = true;
        if dest.is_some() {
            if self.eat_op(",") {
                nl = false;
                if !self.at_stmt_end() {
                    values.push(self.parse_test()?);
                    nl = true;
                }
                while !self.at_stmt_end() {
                    if !self.eat_op(",") {
                        break;
                    }
                    nl = false;
                    if self.at_stmt_end() {
                        break;
                    }
                    values.push(self.parse_test()?);
                    nl = true;
                }
            }
        } else if !self.at_stmt_end() {
            values.push(self.parse_test()?);
            while !self.at_stmt_end() {
                if !self.eat_op(",") {
                    break;
                }
                nl = false;
                if self.at_stmt_end() {
                    break;
                }
                values.push(self.parse_test()?);
                nl = true;
            }
        }
        Ok(self.ast.print_stmt(dest, values, nl))
    }

    /// 捕获到当前逻辑行结束（深度 0 的换行或分号），不吞缩进块。
    fn capture_line_from(&mut self, start: usize) -> PyId {
        if start >= self.toks.len() {
            return self.ast.raw_stmt("");
        }
        let byte_start = self.toks[start].start as usize;
        let mut depth = 0i32;
        let mut i = start;
        while i < self.toks.len() {
            let t = &self.toks[i];
            if t.kind == TokKind::Eof {
                break;
            }
            if t.kind == TokKind::Op {
                match t.text.as_str() {
                    "(" | "[" | "{" => depth += 1,
                    ")" | "]" | "}" => depth -= 1,
                    ";" if depth <= 0 => break,
                    _ => {}
                }
            }
            if t.kind == TokKind::Newline && depth <= 0 {
                break;
            }
            i += 1;
        }
        let byte_end = if i > start {
            self.toks[i - 1].end as usize
        } else {
            byte_start
        };
        self.pos = i;
        let raw = self.src.get(byte_start..byte_end).unwrap_or("");
        self.ast.raw_stmt(raw.trim_end())
    }

    fn capture_raw_from(&mut self, start: usize) -> PyId {
        let byte_start = self.toks[start].start as usize;
        let mut i = start;
        // 装饰器行可能有多条：先逐条吃到各自换行
        while i < self.toks.len() && self.toks[i].kind == TokKind::Op && self.toks[i].text == "@" {
            while i < self.toks.len() && self.toks[i].kind != TokKind::Newline {
                i += 1;
            }
            if i < self.toks.len() {
                i += 1;
            }
            while i < self.toks.len() && self.toks[i].kind == TokKind::Newline {
                i += 1;
            }
        }
        let mut depth = 0i32;
        while i < self.toks.len() {
            let t = &self.toks[i];
            if t.kind == TokKind::Eof {
                break;
            }
            if t.kind == TokKind::Op {
                match t.text.as_str() {
                    "(" | "[" | "{" => depth += 1,
                    ")" | "]" | "}" => depth -= 1,
                    _ => {}
                }
            }
            if t.kind == TokKind::Newline && depth <= 0 {
                i += 1;
                break;
            }
            i += 1;
        }
        loop {
            if i >= self.toks.len() || self.toks[i].kind != TokKind::Indent {
                break;
            }
            let mut lvl = 1i32;
            i += 1;
            while i < self.toks.len() && lvl > 0 {
                match self.toks[i].kind {
                    TokKind::Indent => lvl += 1,
                    TokKind::Dedent => lvl -= 1,
                    TokKind::Eof => break,
                    _ => {}
                }
                i += 1;
            }
        }
        let byte_end = if i > start {
            self.toks[i - 1].end as usize
        } else {
            byte_start
        };
        self.pos = i;
        let raw = self.src.get(byte_start..byte_end).unwrap_or("");
        self.ast.raw_stmt(raw.trim_end())
    }
}

fn from_paren_list(p: &mut Parser<'_>, first: PyId, close: &str) -> R<PyId> {
    let mut items = vec![first];
    let mut trailing_comma = false;
    while p.eat_op(",") {
        trailing_comma = true;
        if p.at_op(close) {
            break;
        }
        items.push(p.parse_starred_test()?);
        trailing_comma = false;
    }
    p.expect_op(close)?;
    if items.len() == 1 && !trailing_comma {
        Ok(items.pop().unwrap())
    } else {
        Ok(p.ast.push(NodeData::Tuple, items))
    }
}

fn string_prefix(raw: &str) -> &str {
    let n = raw
        .char_indices()
        .find(|&(_, c)| c == '\'' || c == '"')
        .map(|(i, _)| i)
        .unwrap_or(0);
    &raw[..n]
}

/// 把装饰器续行的代码缩进从源文件基准缩进改写成打印器的 4 空格层级。
/// 三引号/多行字符串内部的物理行保持原样（空格是字符串值的一部分）。
fn reindent_decorator(
    raw: &str,
    str_spans: &[(usize, usize)],
    base: &str,
    target: &str,
) -> String {
    if raw.contains('\n') == false {
        return raw.to_string();
    }
    let mut out = String::with_capacity(raw.len());
    let mut off = 0usize;
    for (i, line) in raw.lines().enumerate() {
        if i > 0 {
            out.push('\n');
            let in_string = str_spans
                .iter()
                .any(|&(s, e)| s < off && off < e);
            if in_string || line.trim().is_empty() {
                out.push_str(line);
            } else if !base.is_empty() {
                if let Some(rest) = line.strip_prefix(base) {
                    out.push_str(target);
                    out.push_str(rest);
                } else {
                    out.push_str(line);
                }
            } else {
                out.push_str(line);
            }
        } else {
            out.push_str(line);
        }
        off += line.len() + 1;
    }
    out
}

/// Python 2 的 True/False 可以被赋值；检测到绑定后就不要再把它们
/// 当字面量折叠（Python 3 里这种绑定本身是语法错误，按名字处理也无害）。
fn bool_literals_are_shadowed(toks: &[Token]) -> bool {
    // Python 2 的 exec 可以在运行时改写 globals 里的 True/False；保守地
    // 关闭该模块的布尔常量折叠。
    if toks
        .iter()
        .any(|t| t.kind == TokKind::Name && t.text == "exec")
    {
        return true;
    }
    for (i, t) in toks.iter().enumerate() {
        if t.kind != TokKind::Name || !matches!(t.text.as_str(), "True" | "False") {
            continue;
        }
        let mut stmt_start = i;
        while stmt_start > 0 {
            let p = &toks[stmt_start - 1];
            if matches!(
                p.kind,
                TokKind::Newline | TokKind::Indent | TokKind::Dedent
            ) || (p.kind == TokKind::Op && p.text == ";")
            {
                break;
            }
            stmt_start -= 1;
        }
        let mut base_depth = 0i32;
        for n in &toks[stmt_start..i] {
            if n.kind == TokKind::Op {
                match n.text.as_str() {
                    "(" | "[" | "{" => base_depth += 1,
                    ")" | "]" | "}" => base_depth -= 1,
                    _ => {}
                }
            }
        }
        // 1) True/False 之后、同一逻辑语句内出现赋值号（且不是调用里的
        //    关键字参数）。
        let mut depth = base_depth;
        let mut j = i + 1;
        while j < toks.len() {
            let n = &toks[j];
            match n.kind {
                TokKind::Newline | TokKind::Eof => break,
                TokKind::Op => match n.text.as_str() {
                    "(" | "[" | "{" => depth += 1,
                    ")" | "]" | "}" => {
                        depth -= 1;
                        if depth < 0 {
                            break;
                        }
                    }
                    ";" if depth <= base_depth => break,
                    // `if True: x = 1` 的 `=` 在 suite 里；`f(True):` 的冒号
                    // 在调用之外，都不是 True 的赋值。
                    ":" if depth <= base_depth => break,
                    // 只有绝对深度 0 的 `=` 是语句级赋值；调用里的
                    // `f(True, x=1)` / 外层调用的关键字参数不算。
                    "=" if depth == 0 => return true,
                    "+=" | "-=" | "*=" | "/=" | "//=" | "%=" | "**=" | "&=" | "|=" | "^="
                    | "<<=" | ">>="
                        if depth == 0 =>
                    {
                        return true
                    }
                    _ => {}
                },
                _ => {}
            }
            j += 1;
        }
        // `for True in ...` / `for a, True in ...`：向回找最近的 for/in，
        // 先遇到 for 说明 True 在目标列表里；先遇到 in 说明它在 iterable。
        {
            let mut k = i;
            while k > 0 {
                k -= 1;
                let n = &toks[k];
                if matches!(
                    n.kind,
                    TokKind::Newline | TokKind::Indent | TokKind::Dedent | TokKind::Eof
                ) || (n.kind == TokKind::Op && n.text == ";")
                {
                    break;
                }
                if n.kind == TokKind::Name && n.text == "in" {
                    // True 在 iterable 里，不是 for 目标。
                    break;
                }
                if n.kind == TokKind::Name && n.text == "for" {
                    return true;
                }
            }
        }
        // 2) 前置绑定关键字：`as True`、`global True`、`del True`、`def True`。
        let prev = toks[..i]
            .iter()
            .rev()
            .find(|p| {
                !matches!(
                    p.kind,
                    TokKind::Newline | TokKind::Indent | TokKind::Dedent | TokKind::Eof
                )
            });
        if let Some(prev) = prev {
            if prev.kind == TokKind::Name
                && matches!(
                    prev.text.as_str(),
                    "as" | "global" | "nonlocal" | "del" | "def" | "class"
                )
            {
                return true;
            }
        }
        // 3) 同一逻辑语句里位于 def/lambda/except 的头部（冒号之前）：
        //    `def f(True)`、`lambda True:`、`except E, True:`；import/from
        //    里的名字也都是绑定。
        let start = stmt_start;
        // `with x as (True, y):` / `import x as True` 等 as 绑定。
        if (start..i).any(|k| {
            toks[k].kind == TokKind::Name && toks[k].text == "as"
        }) {
            let mut d = 0i32;
            let mut colon = None;
            for k in start..toks.len() {
                let n = &toks[k];
                if matches!(n.kind, TokKind::Newline | TokKind::Eof) {
                    break;
                }
                if n.kind == TokKind::Op {
                    match n.text.as_str() {
                        "(" | "[" | "{" => d += 1,
                        ")" | "]" | "}" => d -= 1,
                        ":" if d <= 0 => {
                            colon = Some(k);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            if colon.map_or(true, |c| i < c) {
                return true;
            }
        }
        let mut is_def = false;
        let mut is_lambda = false;
        let mut is_except = false;
        let mut has_import = false;
        for n in &toks[start..=i] {
            if n.kind == TokKind::Name {
                match n.text.as_str() {
                    "def" => is_def = true,
                    "lambda" => is_lambda = true,
                    "except" => is_except = true,
                    "import" | "from" => has_import = true,
                    _ => {}
                }
            }
        }
        if has_import {
            return true;
        }
        let header = is_def || is_lambda || is_except;
        if header {
            // 头部第一个深度 0 的冒号；它之后是函数体/lambda 体，不是绑定区。
            let mut d = 0i32;
            let mut colon = None;
            for k in start..toks.len() {
                let n = &toks[k];
                if matches!(n.kind, TokKind::Newline | TokKind::Eof) {
                    break;
                }
                if n.kind == TokKind::Op {
                    match n.text.as_str() {
                        "(" | "[" | "{" => d += 1,
                        ")" | "]" | "}" => d -= 1,
                        ":" if d <= 0 => {
                            colon = Some(k);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            if colon.is_some_and(|c| i >= c) {
                continue;
            }
            if is_def || is_lambda {
                // 形参名出现在 `(`/`,`/`lambda`/`*`/`**` 之后；
                // 默认值在 `=` 之后，不是绑定。
                let bind_prev = toks[..i].iter().rev().find(|p| {
                    !matches!(
                        p.kind,
                        TokKind::Newline | TokKind::Indent | TokKind::Dedent | TokKind::Eof
                    )
                });
                if let Some(prev) = bind_prev {
                    if prev.kind == TokKind::Op
                        && matches!(prev.text.as_str(), "(" | "," | "*" | "**")
                    {
                        return true;
                    }
                    if prev.kind == TokKind::Name && prev.text == "lambda" {
                        return true;
                    }
                }
            } else {
                // `except E, True:`：深度 0 的逗号之后是绑定目标；
                // `except (E, True):` 里的逗号在括号内，是类型值。
                let mut d = 0i32;
                let mut depth0_comma = false;
                for k in start..i {
                    let n = &toks[k];
                    if n.kind == TokKind::Op {
                        match n.text.as_str() {
                            "(" | "[" | "{" => d += 1,
                            ")" | "]" | "}" => d -= 1,
                            "," if d <= 0 => depth0_comma = true,
                            _ => {}
                        }
                    }
                }
                if depth0_comma {
                    return true;
                }
            }
        }
    }
    false
}

/// 模块开头的 shebang 与编码声明（最多前两行）。
fn leading_header(src: &str) -> Option<String> {
    let lines: Vec<&str> = src.lines().take(2).collect();
    let mut keep = 0usize;
    if let Some(first) = lines.first() {
        let t = first.trim_start_matches([' ', '\t', '\x0c']);
        if t.starts_with("#!") {
            keep = 1;
            if lines.get(1).is_some_and(|l| is_coding_line(l)) {
                keep = 2;
            }
        } else if is_coding_line(first) {
            keep = 1;
        } else if (t.is_empty() || t.starts_with('#'))
            && lines.get(1).is_some_and(|l| is_coding_line(l))
        {
            // PEP 263 允许第一行为空行或注释、第二行写编码声明。
            keep = 2;
        }
    }
    if keep == 0 {
        return None;
    }
    Some(
        lines[..keep]
            .iter()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn is_coding_line(line: &str) -> bool {
    let t = line.trim_start_matches([' ', '\t', '\x0c']);
    t.starts_with('#') && (t.contains("coding:") || t.contains("coding="))
}

fn number_literal(raw: &str) -> Literal {
    let clean: String = raw.chars().filter(|&c| c != '_').collect();
    // Python 2 long 后缀（`10L` / `0xFFL`）：打印保留原文，值去掉后缀。
    let body = if clean.len() > 1 && clean.ends_with(['l', 'L']) {
        &clean[..clean.len() - 1]
    } else {
        clean.as_str()
    };
    let lower = body.to_ascii_lowercase();
    if lower.ends_with('j') {
        return Literal::raw_str(raw, LitKind::Complex);
    }
    let prefixed = lower.starts_with("0x")
        || lower.starts_with("0o")
        || lower.starts_with("0b");
    if !prefixed && (lower.contains('.') || lower.contains('e')) {
        return Literal::raw_str(raw, LitKind::Float);
    }
    let v = if let Some(h) = lower.strip_prefix("0x") {
        i64::from_str_radix(h, 16).ok()
    } else if let Some(o) = lower.strip_prefix("0o") {
        i64::from_str_radix(o, 8).ok()
    } else if let Some(b) = lower.strip_prefix("0b") {
        i64::from_str_radix(b, 2).ok()
    } else if lower.len() > 1
        && lower.starts_with('0')
        && lower.bytes().all(|b| (b'0'..=b'7').contains(&b))
    {
        // Python 2 旧式八进制 `0777`
        i64::from_str_radix(&lower[1..], 8).ok()
    } else {
        lower.parse::<i64>().ok()
    };
    match v {
        Some(v) => Literal::int_raw(raw, v),
        None => Literal::raw_str(raw, LitKind::Int),
    }
}
