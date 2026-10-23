//! # cure-ts-print
//!
//! TS/JS AST → 源码打印机（与 cure-java-print 同规范：语义保真、
//! 幂等、4 空格缩进）。
//!
//! 关键点：
//! - **ECMA Number→String**（[`format_js_number`]）：最短往返数字
//!   （Rust `{:e}` 与 JS 同源算法）+ ECMA 指数阈值（|v| ≥ 1e21 或
//!   < 1e-6 走指数形态；±0/NaN/Infinity 的 JS 字面形态）。
//! - **优先级驱动加括号**：只加语义必需的括号（AST 的 Paren 在输出
//!   时视同透明层——括号由打印器按优先级重打）。
//! - **原文保真区**：Raw/JSX/类表达式/解构模式/签名/注解原样输出。
//! - **幂等**：cure(cure(x)) == cure(x)。

use cure_engine::kind::{BinOp, NodeKind, UnOp};
use cure_engine::Lang;
use cure_ts_ast::{ClassDecl, ClassMember, JsLit, ModuleUnit, NodeData, NumVal, TopItem, TsAst, TsId, VarKind};

/// 打印模块单元。
pub fn print_module(ast: &TsAst, unit: &ModuleUnit) -> String {
    let mut p = Printer { ast, out: String::new(), indent: 0 };
    if !unit.shebang.is_empty() {
        p.out.push_str(&unit.shebang);
        p.out.push('\n');
    }
    for item in &unit.items {
        p.print_top_item(item);
        p.out.push('\n');
    }
    if !p.out.ends_with('\n') {
        p.out.push('\n');
    }
    p.out
}

struct Printer<'a> {
    ast: &'a TsAst,
    out: String,
    indent: usize,
}

/// 表达式优先级（打印决策）。数值越大越紧密。
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
#[repr(i32)]
enum Prec {
    Seq = 1,
    Assign = 2,
    Cond = 3,
    NullishOr = 4,
    And = 5,
    BitOr = 6,
    BitXor = 7,
    BitAnd = 8,
    Equality = 9,
    Relational = 10,
    Shift = 11,
    Additive = 12,
    Mul = 13,
    Pow = 14,
    Unary = 15,
    Update = 16,
    Member = 17,
}

fn bin_prec_of(op: BinOp) -> Prec {
    use BinOp::*;
    match op {
        Nullish | Or => Prec::NullishOr,
        And => Prec::And,
        BitOr => Prec::BitOr,
        BitXor => Prec::BitXor,
        BitAnd => Prec::BitAnd,
        Eq | Ne | StrictEq | StrictNe => Prec::Equality,
        Lt | Le | Gt | Ge | In | Instanceof => Prec::Relational,
        Shl | Shr | UShr => Prec::Shift,
        Add | Sub => Prec::Additive,
        Mul | Div | Rem => Prec::Mul,
        Pow => Prec::Pow,
    }
}

fn succ(p: Prec) -> Prec {
    let v = p as i32 + 1;
    // 枚举区间内安全（Member=17 不会被 succ——右操作数最高 Mul→Pow…）
    match v {
        1 => Prec::Seq,
        2 => Prec::Assign,
        3 => Prec::Cond,
        4 => Prec::NullishOr,
        5 => Prec::And,
        6 => Prec::BitOr,
        7 => Prec::BitXor,
        8 => Prec::BitAnd,
        9 => Prec::Equality,
        10 => Prec::Relational,
        11 => Prec::Shift,
        12 => Prec::Additive,
        13 => Prec::Mul,
        14 => Prec::Pow,
        15 => Prec::Unary,
        16 => Prec::Update,
        _ => Prec::Member,
    }
}

impl<'a> Printer<'a> {
    /// 尾接续子句（`} else` / `} while` / `} catch`）：补一个空格。
    fn attach_tail(&mut self) {
        if !self.out.ends_with(' ') && !self.out.ends_with('{') {
            self.out.push(' ');
        }
    }

    fn nl_indent(&mut self) {
        self.out.push('\n');
        for _ in 0..self.indent {
            self.out.push_str("    ");
        }
    }
    fn ind(&mut self) {
        for _ in 0..self.indent {
            self.out.push_str("    ");
        }
    }

    // ---- 模块层 ----

    fn print_top_item(&mut self, item: &TopItem) {
        match item {
            TopItem::Stmt(s) => self.print_stmt(*s),
            TopItem::Func { sig_raw, body, .. } => {
                self.out.push_str(sig_raw.trim_end());
                self.out.push(' ');
                match body {
                    Some(b) => self.print_block_like(*b),
                    None => self.out.push_str(";\n"),
                }
            }
            TopItem::Class(c) => self.print_class(c),
            TopItem::ExportStmt(stmt) => {
                self.out.push_str("export ");
                self.print_stmt_at(*stmt);
            }
            TopItem::Import(raw) | TopItem::ExportRaw(raw) | TopItem::Raw(raw) => {
                let r = raw.trim_end();
                self.out.push_str(r);
                if !r.ends_with(';') && !r.ends_with('}') && !r.ends_with('{') {
                    self.out.push(';');
                }
                self.out.push('\n');
            }
        }
    }

    fn print_class(&mut self, c: &ClassDecl) {
        let head = c.head_raw.trim();
        self.out.push_str(&c.kind_kw);
        if !c.name.is_empty() {
            self.out.push(' ');
            self.out.push_str(&c.name);
        }
        if !head.is_empty() {
            self.out.push(' ');
            self.out.push_str(head);
        }
        self.out.push_str(" {");
        if c.members.is_empty() {
            self.out.push('}');
            return;
        }
        self.indent += 1;
        for m in &c.members {
            self.print_class_member(m);
        }
        self.indent -= 1;
        self.ind();
        self.out.push('}');
    }

    fn print_class_member(&mut self, m: &ClassMember) {
        self.nl_indent();
        match m {
            ClassMember::Method { sig_raw, body } => {
                let sig = sig_raw.trim_end().trim_end_matches(';');
                self.out.push_str(sig);
                self.out.push(' ');
                match body {
                    Some(b) => self.print_block_like(*b),
                    None => self.out.push(';'),
                }
            }
            ClassMember::Field { sig_raw, init, .. } => {
                let sig = sig_raw.trim().trim_end();
                self.out.push_str(sig);
                if let Some(i) = init {
                    if !sig.ends_with('=') {
                        self.out.push_str(" = ");
                    } else {
                        self.out.push(' ');
                    }
                    self.print_expr(*i, Prec::Assign);
                }
                self.out.push(';');
            }
            ClassMember::StaticBlock { body } => {
                self.out.push_str("static ");
                self.print_block_like(*body);
            }
            ClassMember::Raw(raw) => {
                let r = raw.trim_end();
                self.out.push_str(r);
                if !r.ends_with(';') && !r.ends_with('}') {
                    self.out.push(';');
                }
                self.out.push('\n');
            }
        }
    }

    // ---- 语句 ----

    /// 语句打印（行首缩进由本层负责）。
    fn print_stmt(&mut self, s: TsId) {
        self.ind();
        self.print_stmt_at(s);
    }

    /// 语句内容打印（无行首缩进；末尾带 `\n` 或块形态）。
    fn print_stmt_at(&mut self, s: TsId) {
        match self.ast.data(s) {
            NodeData::Block => self.print_block_like(s),
            NodeData::Group => {
                // 合成分组：同缩进多语句（无括号——var 多声明符拆行）
                for (i, &c) in self.ast.children(s).iter().enumerate() {
                    if i > 0 {
                        self.nl_indent();
                    }
                    self.print_stmt_at(c);
                }
            }
            NodeData::Empty => {
                self.out.push(';');
            }
            NodeData::ExprStmt => {
                let e = self.ast.children(s)[0];
                self.print_expr(e, Prec::Seq);
                self.out.push(';');
            }
            NodeData::VarDecl { name, kind, ty_raw } => {
                self.print_var_kw(*kind);
                self.out.push(' ');
                self.out.push_str(name);
                if let Some(t) = ty_raw {
                    self.out.push_str(t);
                }
                if let Some(&init) = self.ast.children(s).first() {
                    self.out.push_str(" = ");
                    self.print_expr(init, Prec::Assign);
                }
                self.out.push(';');
            }
            NodeData::DestructStmt { kw, pattern } => {
                if !kw.is_empty() {
                    self.out.push_str(kw);
                    self.out.push(' ');
                }
                self.out.push_str(pattern.trim());
                self.out.push_str(" = ");
                let value = self.ast.children(s)[0];
                self.print_expr(value, Prec::Assign);
                self.out.push(';');
            }
            NodeData::FuncStmt { sig_raw, .. } => {
                self.out.push_str(sig_raw.trim_end());
                self.out.push(' ');
                let body = self.ast.children(s)[0];
                self.print_block_like(body);
            }
            NodeData::If => {
                let ch = self.ast.children(s).to_vec();
                self.out.push_str("if (");
                self.print_expr(ch[0], Prec::Seq);
                self.out.push_str(") ");
                self.print_stmt_or_inline(ch[1]);
                if let Some(&els) = ch.get(2) {
                    self.attach_tail();
                    self.out.push_str("else ");
                    self.print_stmt_or_inline(els);
                }
            }
            NodeData::While => {
                let ch = self.ast.children(s).to_vec();
                self.out.push_str("while (");
                self.print_expr(ch[0], Prec::Seq);
                self.out.push_str(") ");
                self.print_stmt_or_inline(ch[1]);
            }
            NodeData::DoWhile => {
                let ch = self.ast.children(s).to_vec();
                self.out.push_str("do ");
                self.print_stmt_or_inline(ch[0]);
                self.attach_tail();
                self.out.push_str("while (");
                self.print_expr(ch[1], Prec::Seq);
                self.out.push_str(");\n");
            }
            NodeData::For { inits, has_cond, steps } => {
                let ch = self.ast.children(s).to_vec();
                let ni = *inits as usize;
                let nc = if *has_cond { 1 } else { 0 };
                let ns = *steps as usize;
                self.out.push_str("for (");
                // 多声明符共享一个 let/const/var（`for (let a = 0, b = 1; …)`
                // ——逐个打印会产出 `let a = 0, let b = 1` 非法源）
                let all_var_decls = ch.iter().take(ni).all(|&c| {
                    matches!(self.ast.data(c), NodeData::VarDecl { .. })
                }) && ni > 0;
                if all_var_decls {
                    if let NodeData::VarDecl { kind, .. } = self.ast.data(ch[0]) {
                        self.print_var_kw(*kind);
                        self.out.push(' ');
                    }
                    for (i, &c) in ch.iter().take(ni).enumerate() {
                        if i > 0 {
                            self.out.push_str(", ");
                        }
                        self.print_declarator(c);
                    }
                } else {
                    for (i, &c) in ch.iter().take(ni).enumerate() {
                        if i > 0 {
                            self.out.push_str(", ");
                        }
                        self.print_for_head_stmt(c);
                    }
                }
                self.out.push(';');
                if nc == 1 {
                    self.out.push(' ');
                    self.print_expr(ch[ni], Prec::Seq);
                }
                self.out.push(';');
                for (i, &c) in ch.iter().skip(ni + nc).take(ns).enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    } else {
                        self.out.push(' ');
                    }
                    self.print_for_head_stmt(c);
                }
                self.out.push_str(") ");
                self.print_stmt_or_inline(ch[ni + nc + ns]);
            }
            NodeData::ForEach { name, of, decl, ty_raw, await_ } => {
                let ch = self.ast.children(s).to_vec();
                self.out.push_str("for ");
                if *await_ {
                    self.out.push_str("await ");
                }
                self.out.push('(');
                if let Some(k) = decl {
                    self.print_var_kw(*k);
                    self.out.push(' ');
                }
                self.out.push_str(name);
                if let Some(t) = ty_raw {
                    self.out.push_str(t);
                }
                self.out.push_str(if *of { " of " } else { " in " });
                self.print_expr(ch[0], Prec::Seq);
                self.out.push_str(") ");
                self.print_stmt_or_inline(ch[1]);
            }
            NodeData::Return => {
                self.out.push_str("return");
                if let Some(&v) = self.ast.children(s).first() {
                    self.out.push(' ');
                    self.print_expr(v, Prec::Seq);
                }
                self.out.push(';');
            }
            NodeData::Break { label } => {
                self.out.push_str("break");
                if let Some(l) = label {
                    self.out.push(' ');
                    self.out.push_str(l);
                }
                self.out.push(';');
            }
            NodeData::Continue { label } => {
                self.out.push_str("continue");
                if let Some(l) = label {
                    self.out.push(' ');
                    self.out.push_str(l);
                }
                self.out.push(';');
            }
            NodeData::Throw => {
                self.out.push_str("throw ");
                let e = self.ast.children(s)[0];
                self.print_expr(e, Prec::Seq);
                self.out.push(';');
            }
            NodeData::Try => {
                let ch = self.ast.children(s).to_vec();
                self.out.push_str("try ");
                self.print_block_like(ch[0]);
                for &c in &ch[1..] {
                    if self.ast.kind(c) == NodeKind::Catch {
                        if let NodeData::Catch { param_raw } = self.ast.data(c) {
                            self.attach_tail();
                            self.out.push_str("catch ");
                            if let Some(p) = param_raw {
                                self.out.push('(');
                                self.out.push_str(p.trim());
                                self.out.push_str(") ");
                            }
                            let body = self.ast.children(c)[0];
                            self.print_block_like(body);
                        }
                    } else {
                        self.attach_tail();
                        self.out.push_str("finally ");
                        self.print_block_like(c);
                    }
                }
            }
            NodeData::Switch => {
                let ch = self.ast.children(s).to_vec();
                self.out.push_str("switch (");
                self.print_expr(ch[0], Prec::Seq);
                self.out.push_str(") {");
                self.indent += 1;
                for &c in &ch[1..] {
                    self.print_case(c);
                }
                self.indent -= 1;
                self.ind();
                self.out.push('}');
            }
            NodeData::Label { name } => {
                self.out.push_str(name);
                self.out.push_str(": ");
                let stmt = self.ast.children(s)[0];
                self.print_stmt_or_inline(stmt);
            }
            NodeData::Raw { text } => {
                let t = text.trim_end();
                self.out.push_str(t);
                if !t.ends_with(';') && !t.ends_with('}') && !t.is_empty() {
                    self.out.push(';');
                }
            }
            _ => {
                // 表达式直达（防御）
                self.print_expr(s, Prec::Seq);
                self.out.push(';');
            }
        }
    }

    fn print_var_kw(&mut self, k: VarKind) {
        self.out.push_str(match k {
            VarKind::Var => "var",
            VarKind::Let => "let",
            VarKind::Const => "const",
        });
    }

    /// 声明体（无关键字）：`name [: ty] [= init]`。
    fn print_declarator(&mut self, s: TsId) {
        if let NodeData::VarDecl { name, ty_raw, .. } = self.ast.data(s) {
            self.out.push_str(name);
            if let Some(t) = ty_raw {
                self.out.push_str(t);
            }
            if let Some(&init) = self.ast.children(s).first() {
                self.out.push_str(" = ");
                self.print_expr(init, Prec::Assign);
            }
        }
    }

    /// for 头部的「语句」（去分号形式）。
    fn print_for_head_stmt(&mut self, s: TsId) {
        match self.ast.data(s) {
            NodeData::ExprStmt => {
                let e = self.ast.children(s)[0];
                self.print_expr(e, Prec::Seq);
            }
            NodeData::VarDecl { name, kind, ty_raw } => {
                self.print_var_kw(*kind);
                self.out.push(' ');
                self.out.push_str(name);
                if let Some(t) = ty_raw {
                    self.out.push_str(t);
                }
                if let Some(&init) = self.ast.children(s).first() {
                    self.out.push_str(" = ");
                    self.print_expr(init, Prec::Assign);
                }
            }
            _ => self.print_expr(s, Prec::Seq),
        }
    }

    /// case 子句。
    fn print_case(&mut self, c: TsId) {
        if let NodeData::Case { labels, is_default, arrow } = self.ast.data(c) {
            self.nl_indent();
            if *is_default {
                self.out.push_str("default:");
                let _ = arrow;
            } else {
                self.out.push_str("case ");
                let ch = self.ast.children(c).to_vec();
                for (i, &l) in ch.iter().take(*labels as usize).enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    self.print_expr(l, Prec::Seq);
                }
                if *arrow {
                    self.out.push_str(" =>");
                } else {
                    self.out.push(':');
                }
            }
            self.indent += 1;
            for &st in self.ast.children(c).iter().skip(*labels as usize) {
                self.nl_indent();
                self.print_stmt_at(st);
            }
            self.indent -= 1;
        } else if let NodeData::Raw { text } = self.ast.data(c) {
            self.nl_indent();
            self.out.push_str(text.trim_end());
        }
    }

    /// 块形态打印（`{ … }` 换行缩进；带尾换行）。
    fn print_block_like(&mut self, b: TsId) {
        self.out.push('{');
        let stmts = self.ast.children(b).to_vec();
        if stmts.is_empty() {
            self.out.push('}');
            return;
        }
        self.indent += 1;
        for &s in &stmts {
            self.nl_indent();
            self.print_stmt_at(s);
        }
        self.indent -= 1;
        self.nl_indent();
        self.out.push('}');
    }

    /// 语句位（if/while/for 体）：块 → 块形态；单语句 → 单行（同缩进）。
    fn print_stmt_or_inline(&mut self, s: TsId) {
        if self.ast.kind(s) == NodeKind::Block {
            self.print_block_like(s);
            return;
        }
        self.print_stmt_at(s);
    }

    // ---- 表达式 ----

    /// 表达式打印（`prec` = 父语境要求的最低优先级）。
    fn print_expr(&mut self, e: TsId, prec: Prec) {
        let my_prec = self.expr_prec(e);
        // 语法级必需括号：引擎 paren_removal 删除 AST 括号、由本层优先级
        // 模型重建——但优先级序**表达不了语法产生式约束**（JS 里
        // `a ?? (() => true)` 的箭头/`(yield x)`/对象字面量在二元/一元
        // 运算数位必须带括号——产生式只接受 AssignmentExpression 以下
        // 的形态位）。赋值级及更松位置（实参/序列/三元臂）这些形态合法。
        let grammar_paren = prec > Prec::Assign && self.ast.grammar_paren_required(e);
        if my_prec < prec || grammar_paren {
            self.out.push('(');
            self.print_expr_inner(e, my_prec);
            self.out.push(')');
            return;
        }
        self.print_expr_inner(e, my_prec);
    }

    fn expr_prec(&self, e: TsId) -> Prec {
        match self.ast.data(e) {
            NodeData::Seq => Prec::Seq,
            NodeData::Assign { .. } => Prec::Assign,
            NodeData::Ternary => Prec::Cond,
            NodeData::Binary { op } => bin_prec_of(*op),
            NodeData::Unary { op } => match op {
                UnOp::PreInc | UnOp::PreDec | UnOp::PostInc | UnOp::PostDec => Prec::Update,
                _ => Prec::Unary,
            },
            _ => Prec::Member,
        }
    }

    fn print_expr_inner(&mut self, e: TsId, my_prec: Prec) {
        match self.ast.data(e) {
            NodeData::Literal(l) => self.print_lit(l),
            NodeData::VarRef { name } => {
                let n = self.ast.sn(*name).to_string();
                self.out.push_str(&n);
            }
            NodeData::This => self.out.push_str("this"),
            NodeData::Paren => {
                // 保守保留括号：括号可能承载语法必需性（yield/逗号序列
                // 等 Raw 形态的优先级不可判）。冗余括号由引擎
                // paren_removal 规则在 AST 层消除——打印层不承担判定
                let inner = self.ast.children(e)[0];
                self.out.push('(');
                self.print_expr(inner, Prec::Seq);
                self.out.push(')');
            }
            NodeData::Binary { op } => self.print_binary(e, *op, my_prec),
            NodeData::Unary { op } => self.print_unary(e, *op),
            NodeData::Ternary => {
                let ch = self.ast.children(e).to_vec();
                self.print_expr(ch[0], Prec::NullishOr);
                self.out.push_str(" ? ");
                self.print_expr(ch[1], Prec::Assign);
                self.out.push_str(" : ");
                self.print_expr(ch[2], Prec::Assign);
            }
            NodeData::Assign { op } => {
                let ch = self.ast.children(e).to_vec();
                self.print_expr(ch[0], Prec::Assign);
                self.out.push(' ');
                match op {
                    Some(BinOp::And) => self.out.push_str("&&="),
                    Some(BinOp::Or) => self.out.push_str("||="),
                    Some(BinOp::Nullish) => self.out.push_str("??="),
                    Some(o) => {
                        self.out.push_str(bin_symbol(*o));
                        self.out.push('=');
                    }
                    None => self.out.push('='),
                }
                self.out.push(' ');
                self.print_expr(ch[1], Prec::Assign);
            }
            NodeData::Call { optional } => {
                let ch = self.ast.children(e).to_vec();
                let callee = ch[0];
                if matches!(
                    self.ast.data(callee),
                    NodeData::New | NodeData::Lambda { arrow: false, .. }
                ) {
                    self.out.push('(');
                    self.print_expr_inner(callee, self.expr_prec(callee));
                    self.out.push_str(")(");
                } else if *optional {
                    self.print_expr(callee, Prec::Member);
                    self.out.push_str("?.(");
                } else {
                    self.print_expr(callee, Prec::Member);
                    self.out.push('(');
                }
                self.print_args(&ch[1..]);
                self.out.push(')');
            }
            NodeData::New => {
                let ch = self.ast.children(e).to_vec();
                self.out.push_str("new ");
                self.print_expr(ch[0], Prec::Member);
                self.out.push('(');
                self.print_args(&ch[1..]);
                self.out.push(')');
            }
            NodeData::Member { name, optional } => {
                let ch = self.ast.children(e).to_vec();
                self.print_expr(ch[0], Prec::Member);
                if *optional {
                    self.out.push_str("?.");
                } else {
                    self.out.push('.');
                }
                self.out.push_str(self.ast.sn(*name));
            }
            NodeData::Index { optional } => {
                let ch = self.ast.children(e).to_vec();
                self.print_expr(ch[0], Prec::Member);
                if *optional {
                    self.out.push_str("?.[");
                } else {
                    self.out.push('[');
                }
                self.print_expr(ch[1], Prec::Seq);
                self.out.push(']');
            }
            NodeData::ArrayLit => {
                self.out.push('[');
                let ch = self.ast.children(e).to_vec();
                for (i, &c) in ch.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    self.print_expr(c, Prec::Assign);
                }
                self.out.push(']');
            }
            NodeData::ObjLit { props } => {
                let ch = self.ast.children(e).to_vec();
                self.out.push('{');
                for (i, (p, v)) in props.iter().zip(ch.iter()).enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    if p == "..." {
                        self.out.push_str("...");
                        self.print_expr(*v, Prec::Assign);
                        continue;
                    }
                    if self.is_shorthand(*v, p) {
                        self.out.push_str(p.trim());
                        continue;
                    }
                    self.out.push_str(p.trim());
                    if p.contains('(') {
                        // 方法简写：值是 Lambda——prop 已带 (params)
                        if let Some(&b) = self.ast.children(*v).first() {
                            self.out.push(' ');
                            self.print_block_like_expr(b);
                        }
                        continue;
                    }
                    self.out.push_str(": ");
                    self.print_expr(*v, Prec::Assign);
                }
                self.out.push('}');
            }
            NodeData::Lambda { params_raw, arrow, is_async, is_generator } => {
                if *is_async {
                    self.out.push_str("async ");
                }
                if *arrow {
                    let ps = params_raw.trim();
                    if is_plain_ident(ps) {
                        self.out.push_str(ps);
                    } else {
                        // 多参/模式/默认值/类型注解——括号必需
                        if ps.starts_with('(') && ps.ends_with(')') {
                            self.out.push_str(ps);
                        } else {
                            self.out.push('(');
                            self.out.push_str(ps);
                            self.out.push(')');
                        }
                    }
                    self.out.push_str(" => ");
                } else {
                    self.out.push_str("function");
                    if *is_generator {
                        self.out.push('*');
                    }
                    self.out.push('(');
                    self.out.push_str(params_raw.trim());
                    self.out.push_str(") ");
                }
                let body = self.ast.children(e)[0];
                self.print_lambda_body(body);
            }
            NodeData::Template { quasis, tag } => {
                let ch = self.ast.children(e).to_vec();
                let interps: &[TsId] = if *tag { &ch[1..] } else { &ch[..] };
                if *tag {
                    self.print_expr(ch[0], Prec::Member);
                }
                self.out.push('`');
                for (i, q) in quasis.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str("${");
                        self.print_expr(interps[i - 1], Prec::Seq);
                        self.out.push('}');
                    }
                    self.out.push_str(&print_quasi(q));
                }
                self.out.push('`');
            }
            NodeData::Spread => {
                self.out.push_str("...");
                let v = self.ast.children(e)[0];
                self.print_expr(v, Prec::Assign);
            }
            NodeData::Destruct { pattern_raw } => {
                self.out.push_str(pattern_raw.trim());
                self.out.push_str(" = ");
                let v = self.ast.children(e)[0];
                self.print_expr(v, Prec::Assign);
            }
            NodeData::TsAs { ty_raw } => {
                let v = self.ast.children(e)[0];
                self.print_expr(v, Prec::Shift);
                self.out.push_str(" as ");
                self.out.push_str(ty_raw.trim());
            }
            NodeData::TsSatisfies { ty_raw } => {
                let v = self.ast.children(e)[0];
                self.print_expr(v, Prec::Shift);
                self.out.push_str(" satisfies ");
                self.out.push_str(ty_raw.trim());
            }
            NodeData::TsNonNull => {
                let v = self.ast.children(e)[0];
                self.print_expr(v, Prec::Member);
                self.out.push('!');
            }
            NodeData::Seq => {
                let ch = self.ast.children(e).to_vec();
                for (i, &c) in ch.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    self.print_expr(c, Prec::Assign);
                }
            }
            NodeData::Raw { text } => self.out.push_str(text.trim()),
            NodeData::ClassExpr { raw } => self.out.push_str(raw.trim()),
            NodeData::FuncStmt { sig_raw, .. } => self.out.push_str(sig_raw.trim()),
            _ => self.out.push_str("/*?*/"),
        }
    }

    /// `{ x: x }` → `{ x }` 简写判定。
    fn is_shorthand(&self, v: TsId, key: &str) -> bool {
        if let NodeData::VarRef { name } = self.ast.data(v) {
            let k = key.trim();
            return self.ast.sn(*name) == k && is_plain_ident(k);
        }
        false
    }

    fn print_args(&mut self, args: &[TsId]) {
        for (i, &a) in args.iter().enumerate() {
            if i > 0 {
                self.out.push_str(", ");
            }
            self.print_expr(a, Prec::Assign);
        }
    }

    fn print_binary(&mut self, e: TsId, op: BinOp, my_prec: Prec) {
        let ch = self.ast.children(e).to_vec();
        let (l, r) = (ch[0], ch[1]);
        // ** 右结合：左操作数同层加括号；左结合：右操作数同层加括号。
        // `??` 与 &&/|| 的 JS 混合禁令：右操作数是 &&/||/?? 时加括号。
        // 左操作数：同层平铺（** 的左结合限制由右侧括号规则 +
        // expr_prec < 检查覆盖——(a**b)**c 的左 Binary(Pow) 层相等走
        // print_expr 的 my_prec < prec 判定加括号）
        let lp = my_prec;
        self.print_expr(l, lp);
        self.out.push(' ');
        self.out.push_str(bin_symbol(op));
        self.out.push(' ');
        let rp = if op == BinOp::Pow {
            my_prec // 右结合：右侧同层平铺
        } else {
            succ(my_prec) // 左结合：右侧同层括号
        };
        // 混合禁令覆盖：?? 的右操作数 &&/||（层更高但语法禁括号省）
        let rp_final = if op == BinOp::Nullish {
            if let NodeData::Binary { op: ro } = self.ast.data(r) {
                if matches!(ro, BinOp::And | BinOp::Or | BinOp::Nullish) {
                    Prec::Assign // 强制括号（低于一切二元）
                } else {
                    rp
                }
            } else {
                rp
            }
        } else if matches!(op, BinOp::And | BinOp::Or) {
            // a && (b ?? c) 合法但 a && b ?? c 非法——右操作数 ?? 需括号
            if let NodeData::Binary { op: ro } = self.ast.data(r) {
                if matches!(ro, BinOp::Nullish) {
                    Prec::Assign
                } else {
                    rp
                }
            } else {
                rp
            }
        } else {
            rp
        };
        self.print_expr(r, rp_final);
    }

    fn print_unary(&mut self, e: TsId, op: UnOp) {
        let operand = self.ast.children(e)[0];
        match op {
            UnOp::Not => {
                self.out.push('!');
                self.print_expr(operand, Prec::Unary);
            }
            UnOp::BitNot => {
                self.out.push('~');
                self.print_expr(operand, Prec::Unary);
            }
            UnOp::Neg => {
                self.out.push('-');
                if matches!(
                    self.ast.data(operand),
                    NodeData::Unary { op: UnOp::Neg | UnOp::PreDec | UnOp::PreInc }
                ) {
                    self.out.push(' ');
                }
                self.print_expr(operand, Prec::Unary);
            }
            UnOp::Plus => {
                self.out.push('+');
                if matches!(
                    self.ast.data(operand),
                    NodeData::Unary { op: UnOp::Plus | UnOp::PreInc | UnOp::PreDec }
                ) {
                    self.out.push(' ');
                }
                self.print_expr(operand, Prec::Unary);
            }
            UnOp::TypeOf => {
                self.out.push_str("typeof ");
                self.print_expr(operand, Prec::Unary);
            }
            UnOp::Void => {
                self.out.push_str("void ");
                self.print_expr(operand, Prec::Unary);
            }
            UnOp::Delete => {
                self.out.push_str("delete ");
                self.print_expr(operand, Prec::Unary);
            }
            UnOp::Await => {
                self.out.push_str("await ");
                self.print_expr(operand, Prec::Unary);
            }
            UnOp::PreInc => {
                self.out.push_str("++");
                self.print_expr(operand, Prec::Unary);
            }
            UnOp::PreDec => {
                self.out.push_str("--");
                self.print_expr(operand, Prec::Unary);
            }
            UnOp::PostInc => {
                self.print_expr(operand, Prec::Update);
                self.out.push_str("++");
            }
            UnOp::PostDec => {
                self.print_expr(operand, Prec::Update);
                self.out.push_str("--");
            }
        }
    }

    /// 箭头体：块 → 块；表达式 → 直接（ObjLit/Seq 加括号）。
    fn print_lambda_body(&mut self, body: TsId) {
        if self.ast.kind(body) == NodeKind::Block {
            self.print_block_like(body);
            return;
        }
        if matches!(self.ast.data(body), NodeData::ObjLit { .. } | NodeData::Seq) {
            self.out.push('(');
            self.print_expr_inner(body, self.expr_prec(body));
            self.out.push(')');
            return;
        }
        self.print_expr(body, Prec::Assign);
    }

    /// 表达式位块（对象方法体）：紧凑单行。
    fn print_block_like_expr(&mut self, b: TsId) {
        self.out.push('{');
        let stmts = self.ast.children(b).to_vec();
        if !stmts.is_empty() {
            self.out.push(' ');
            for (i, &s) in stmts.iter().enumerate() {
                if i > 0 {
                    self.out.push(' ');
                }
                let save = self.out.len();
                self.print_stmt_at(s);
                let tail = self.out[save..].to_string();
                self.out.truncate(save);
                self.out.push_str(tail.trim());
                self.out.push(' ');
            }
            self.out.pop(); // 末尾多余空格
        }
        self.out.push('}');
    }

    // ---- 字面量 ----

    fn print_lit(&mut self, l: &JsLit) {
        match l {
            JsLit::Bool(b) => self.out.push_str(if *b { "true" } else { "false" }),
            JsLit::Int(v) => self.out.push_str(&v.to_string()),
            JsLit::Double(v) => self.out.push_str(&format_js_number(*v)),
            JsLit::NumRaw { text, val } => match val {
                // 十六/八/二进制原文**归一化**为十进制（反混淆可读性
                //——0x1f 与 31 语义恒等；混淆器偏爱 radix 形态）
                NumVal::Int(v) => {
                    let _ = text;
                    self.out.push_str(&v.to_string());
                }
                NumVal::Double(v) => {
                    if v.is_nan() || v.is_infinite() {
                        self.out.push_str(text);
                    } else {
                        self.out.push_str(&format_js_number(*v));
                    }
                }
            },
            JsLit::BigInt(raw) => {
                let clean: String = raw.chars().filter(|c| *c != '_').collect();
                self.out.push_str(&clean);
            }
            JsLit::Str(v) => self.print_str(v),
            JsLit::Regex(raw) => self.out.push_str(raw),
            JsLit::Null => self.out.push_str("null"),
        }
    }

    /// 字符串：单引号策略 + 最小转义。
    fn print_str(&mut self, v: &str) {
        self.out.push('\'');
        for c in v.chars() {
            match c {
                '\'' => self.out.push_str("\\'"),
                '\\' => self.out.push_str("\\\\"),
                '\n' => self.out.push_str("\\n"),
                '\r' => self.out.push_str("\\r"),
                '\t' => self.out.push_str("\\t"),
                '\u{0008}' => self.out.push_str("\\b"),
                '\u{000c}' => self.out.push_str("\\f"),
                '\u{000b}' => self.out.push_str("\\v"),
                c if (c as u32) < 0x20 => {
                    self.out.push_str(&format!("\\u{:04x}", c as u32));
                }
                c => self.out.push(c),
            }
        }
        self.out.push('\'');
    }
}

fn is_plain_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_' || c == '$')
        && s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

fn bin_symbol(op: BinOp) -> &'static str {
    use BinOp::*;
    match op {
        Add => "+",
        Sub => "-",
        Mul => "*",
        Div => "/",
        Rem => "%",
        Pow => "**",
        Shl => "<<",
        Shr => ">>",
        UShr => ">>>",
        Lt => "<",
        Le => "<=",
        Gt => ">",
        Ge => ">=",
        Eq => "==",
        Ne => "!=",
        StrictEq => "===",
        StrictNe => "!==",
        BitAnd => "&",
        BitXor => "^",
        BitOr => "|",
        And => "&&",
        Or => "||",
        Nullish => "??",
        In => "in",
        Instanceof => "instanceof",
    }
}

/// quasi 文本打印：**原样透传**。quasi 由 lexer 从源扫描——源里的
/// 转义序列（\n \` \${）以源形态存储（反斜杠+字母两字符），
/// 字面反引号不可能未转义出现在 quasi（扫描会在其处终止）。
/// 再转义会破坏转义序列（差分抓获：`a\nb` 曾打印成 `a\\nb`——
/// 新义为字面反斜杠+n）。
fn print_quasi(q: &str) -> String {
    q.to_string()
}

/// ECMA Number::toString（JS 数字字面形态）。
pub fn format_js_number(v: f64) -> String {
    if v == 0.0 {
        return if v.is_sign_negative() { "-0".to_string() } else { "0".to_string() };
    }
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v < 0.0 { "-Infinity".to_string() } else { "Infinity".to_string() };
    }
    let s = format!("{:e}", v);
    let (mant, exp_s) = s.split_once('e').expect("exponent form");
    let exp: i32 = exp_s.parse().expect("exponent");
    let neg = mant.starts_with('-');
    let mant = mant.trim_start_matches('-');
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    let k = digits.len() as i32;
    let n = exp + 1;
    let body = if k <= n && n <= 21 {
        format!("{}{}", digits, "0".repeat((n - k) as usize))
    } else if 0 < n && n <= 21 {
        format!("{}.{}", &digits[..n as usize], &digits[n as usize..])
    } else if -6 < n && n <= 0 {
        format!("0.{}{}", "0".repeat((-n) as usize), digits)
    } else if k == 1 {
        format!("{}e{}{}", digits, sign(exp), exp.abs())
    } else {
        format!("{}.{}e{}{}", &digits[..1], &digits[1..], sign(exp), exp.abs())
    };
    if neg {
        format!("-{body}")
    } else {
        body
    }
}

fn sign(exp: i32) -> &'static str {
    if exp >= 0 {
        "+"
    } else {
        "-"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cure_ts_parser::parse;

    fn roundtrip(src: &str) -> String {
        let out = parse(src);
        assert!(out.errors.is_empty(), "{:#?}", out.errors);
        print_module(&out.ast, &out.unit)
    }

    #[test]
    fn js_number_format() {
        assert_eq!(format_js_number(0.1), "0.1");
        assert_eq!(format_js_number(100.0), "100");
        assert_eq!(format_js_number(123.456), "123.456");
        assert_eq!(format_js_number(1e21), "1e+21");
        assert_eq!(format_js_number(1e-7), "1e-7");
        assert_eq!(format_js_number(5e-7), "5e-7");
        assert_eq!(format_js_number(9007199254740992.0), "9007199254740992");
        assert_eq!(format_js_number(-0.0), "-0");
        assert_eq!(format_js_number(0.5), "0.5");
        assert_eq!(format_js_number(-2.5), "-2.5");
    }

    #[test]
    fn print_basic() {
        let out = roundtrip("const x = 1;\nfunction f(a) { return a + x; }\n");
        assert!(out.contains("const x = 1;"));
        assert!(out.contains("function f(a) {"));
        assert!(out.contains("return a + x;"));
    }

    #[test]
    fn print_control_flow() {
        let out = roundtrip(
            "function f(n) { if (n > 1) { return 1; } else { return 2; } for (let i = 0; i < n; i++) { f(i); } }",
        );
        assert!(out.contains("if (n > 1) {"));
        assert!(out.contains("} else {"));
        assert!(out.contains("for (let i = 0; i < n; i++) {"));
    }

    #[test]
    fn print_idempotent() {
        let src = "const f = (a, b) => a + b;\nfunction g(n) { let s = 0; for (const x of [1, 2, 3]) { s += x * 2; } return s ?? 0; }\n";
        let once = roundtrip(src);
        let twice = roundtrip(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn print_precedence_parens() {
        let out = roundtrip("const y = (a + b) * c;\n");
        assert!(out.contains("(a + b) * c"));
        let out = roundtrip("const y = a - (b - c);\n");
        assert!(out.contains("a - (b - c)"));
        let out = roundtrip("const y = (a ** b) ** c;\n");
        assert!(out.contains("(a ** b) ** c"));
        let out = roundtrip("const y = a ** b ** c;\n");
        assert!(out.contains("a ** b ** c"));
    }

    #[test]
    fn print_template() {
        let out = roundtrip("const s = `x${a}y`;\n");
        assert!(out.contains("`x${a}y`"));
    }

    #[test]
    fn print_class() {
        let out = roundtrip("class A extends B { constructor(a) { super(a); } m(x) { return x; } }\n");
        assert!(out.contains("class A extends B {"));
        assert!(out.contains("constructor(a) {"));
    }

    #[test]
    fn print_obj_and_array() {
        let out = roundtrip("const o = { a: 1, b, m() { return 1; } };\nconst arr = [1, 2, 3];\n");
        assert!(out.contains("a: 1"));
        assert!(out.contains("m() {"));
    }

    #[test]
    fn print_try_switch() {
        let out = roundtrip(
            "try { f(); } catch (e) { g(e); } finally { h(); }\nswitch (x) { case 1: a(); break; default: b(); }\n",
        );
        assert!(out.contains("try {"));
        assert!(out.contains("} catch (e) {"));
        assert!(out.contains("} finally {"));
        assert!(out.contains("case 1:"));
    }
}
