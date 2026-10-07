//! # cure-java-print
//!
//! cure-java-ast → 规范格式化的 Java 源码（canonical printer / formatter）。
//!
//! 两层能力，可分别使用：
//! - [`print`]：方法体 / 语句子树（arena 节点）；
//! - [`print_unit`]：整个编译单元（package/imports/类/字段/方法签名保真，
//!   方法体走规范格式化）。
//!
//! 签名里的泛型、注解、throws 按原文输出；不可解析区域（Raw）原样保留。

use cure_java_ast::{Sym, 
    BinOp, CompilationUnit, Declarator, JType, JavaAst, JavaId, Lit, Member, NodeData, NodeKind,
    Param, TypeDecl, TypeKind, UnOp,
};

// ---- 优先级表（数值越大绑定越紧）----
#[allow(dead_code)]
mod prec {
    pub const ASSIGN: u8 = 2; // 右结合
    pub const TERNARY: u8 = 3; // 右结合
    pub const OR: u8 = 4;
    pub const AND: u8 = 5;
    pub const BIT_OR: u8 = 6;
    pub const BIT_XOR: u8 = 7;
    pub const BIT_AND: u8 = 8;
    pub const EQUALITY: u8 = 9;
    pub const RELATIONAL: u8 = 10;
    pub const SHIFT: u8 = 11;
    pub const ADDITIVE: u8 = 12;
    pub const MULTIPLICATIVE: u8 = 13;
    pub const UNARY: u8 = 14;
    pub const POSTFIX: u8 = 15;
    pub const PRIMARY: u8 = 16;
}

fn bin_prec(op: BinOp) -> u8 {
    use BinOp::*;
    match op {
        Or => prec::OR,
        And => prec::AND,
        BitOr => prec::BIT_OR,
        BitXor => prec::BIT_XOR,
        BitAnd => prec::BIT_AND,
        Eq | Ne => prec::EQUALITY,
        Lt | Le | Gt | Ge => prec::RELATIONAL,
        Shl | Shr | UShr => prec::SHIFT,
        Add | Sub => prec::ADDITIVE,
        Mul | Div | Rem => prec::MULTIPLICATIVE,
    }
}

fn bin_symbol(op: BinOp) -> &'static str {
    use BinOp::*;
    match op {
        Add => "+",
        Sub => "-",
        Mul => "*",
        Div => "/",
        Rem => "%",
        Shl => "<<",
        Shr => ">>",
        UShr => ">>>",
        Lt => "<",
        Le => "<=",
        Gt => ">",
        Ge => ">=",
        Eq => "==",
        Ne => "!=",
        BitAnd => "&",
        BitXor => "^",
        BitOr => "|",
        And => "&&",
        Or => "||",
    }
}

fn assign_symbol(op: Option<BinOp>) -> String {
    match op {
        None => "=".into(),
        Some(o) => format!("{}=", bin_symbol(o)),
    }
}

fn un_symbol(op: UnOp) -> &'static str {
    match op {
        UnOp::Not => "!",
        UnOp::Neg => "-",
        UnOp::BitNot => "~",
        UnOp::PreInc | UnOp::PostInc => "++",
        UnOp::PreDec | UnOp::PostDec => "--",
    }
}

/// 缩进单位（canonical 4 空格）。
pub const INDENT: &str = "    ";

// ---------------------------------------------------------------------------
// 编译单元 / 成员
// ---------------------------------------------------------------------------

/// 整个编译单元 → 格式化源码。
pub fn print_unit(ast: &JavaAst, unit: &CompilationUnit) -> String {
    let mut out = String::new();
    if let Some(pkg) = &unit.package {
        out.push_str(&format!("package {pkg};\n\n"));
    }
    for imp in &unit.imports {
        out.push_str(&format!("import {imp};\n"));
    }
    if !unit.imports.is_empty() {
        out.push('\n');
    }
    for raw in &unit.raws {
        out.push_str(raw);
        out.push('\n');
    }
    let mut p = Printer {
        ast,
        out: String::new(),
        level: 0,
    };
    for (i, t) in unit.types.iter().enumerate() {
        if i > 0 {
            p.out.push('\n');
            p.out.push('\n');
        }
        p.type_decl(t);
    }
    out.push_str(&p.out);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn mods_prefix(mods: &str) -> String {
    if mods.is_empty() {
        String::new()
    } else {
        format!("{mods} ")
    }
}

fn kind_kw(k: TypeKind) -> &'static str {
    match k {
        TypeKind::Class => "class",
        TypeKind::Interface => "interface",
        TypeKind::Enum => "enum",
        TypeKind::Record => "record",
        TypeKind::Annotation => "@interface",
    }
}

pub fn ty_str(ty: &JType) -> String {
    match ty {
        JType::Bool => "boolean".into(),
        JType::Byte => "byte".into(),
        JType::Short => "short".into(),
        JType::Int => "int".into(),
        JType::Long => "long".into(),
        JType::Char => "char".into(),
        JType::Float => "float".into(),
        JType::Double => "double".into(),
        JType::Void => "void".into(),
        JType::Ref(name) => name.clone(),
        JType::Array(inner) => format!("{}[]", ty_str(inner)),
        JType::Var => "var".into(),
    }
}

fn param_str(p: &Param) -> String {
    let dots = if p.varargs { "..." } else { "" };
    format!("{}{}{} {}", mods_prefix(&p.mods), ty_str(&p.ty), dots, p.name)
}

fn declarator_str(ast: &JavaAst, ty: &JType, d: &Declarator) -> String {
    let mut p = Printer {
        ast,
        out: String::new(),
        level: 0,
    };
    let mut s = d.name.clone();
    for _ in 0..d.extra_dims {
        s.push_str("[]");
    }
    if let Some(init) = d.init {
        s.push_str(" = ");
        let full = wrap_dims(ty, d.extra_dims);
        let init = decl_init_expr(ast, &full, init);
        p.expr(init, prec::ASSIGN);
        s.push_str(&p.out);
    }
    s
}

/// `T[] x = new T[]{…}` 在声明处回退惯用短形态 `T[] x = {…}`：
/// 解析器把裸 `{…}` 归一化成 `new T[]{…}`（表达式位置合法），
/// 打印在声明上下文镜像还原。仅当维度匹配、无尺寸、唯一 ArrayLit 子节点。
fn decl_init_expr(ast: &JavaAst, ty: &JType, init: JavaId) -> JavaId {
    if let NodeData::NewArray { dims, sized, .. } = ast.data(init) {
        if *sized == 0 && ast.children(init).len() == 1 {
            let only = ast.children(init)[0];
            if matches!(ast.data(only), NodeData::ArrayLit) {
                let mut n = 0u16;
                let mut t = ty;
                while let JType::Array(inner) = t {
                    t = inner;
                    n += 1;
                }
                if n == *dims && !matches!(t, JType::Var) {
                    return only;
                }
            }
        }
    }
    init
}

fn wrap_dims(ty: &JType, n: u16) -> JType {
    let mut t = ty.clone();
    for _ in 0..n {
        t = JType::Array(Box::new(t));
    }
    t
}

fn throws_str(throws: &[String]) -> String {
    if throws.is_empty() {
        String::new()
    } else {
        format!(" throws {}", throws.join(", "))
    }
}

// ---------------------------------------------------------------------------
// 节点打印器
// ---------------------------------------------------------------------------

pub struct Printer<'a> {
    ast: &'a JavaAst,
    out: String,
    level: usize,
}

/// 把 AST 子树打印为格式化源码。
///
/// 顶层如果是 [`NodeData::Block`]，视为"方法体"：输出其语句序列，
/// 不含外层花括号（方法头部的 `{}` 由调用方负责）；内部块照常带花括号。
pub fn print(ast: &JavaAst, root: JavaId) -> String {
    let mut p = Printer {
        ast,
        out: String::new(),
        level: 0,
    };
    if ast.data(root) == &NodeData::Block {
        let children = ast.children(root).to_vec();
        for (i, &s) in children.iter().enumerate() {
            if i > 0 {
                p.newline();
            }
            p.stmt(s);
        }
    } else {
        p.stmt(root);
    }
    p.out
}

impl<'a> Printer<'a> {
    fn indent(&mut self) {
        for _ in 0..self.level {
            self.out.push_str(INDENT);
        }
    }
    fn newline(&mut self) {
        self.out.push('\n');
    }

    // ---- 类型声明 / 成员 ----

    fn type_decl(&mut self, t: &TypeDecl) {
        self.indent();
        self.out.push_str(&mods_prefix(&t.mods));
        self.out.push_str(kind_kw(t.kind));
        self.out.push(' ');
        self.out.push_str(&t.name);
        self.out.push_str(&t.ty_params);
        self.out.push_str(&t.header);
        // JLS 声明序：extends 在 permits 之前（permits 提前 javac 报
        //「需要 '{'」——差分审查抓获，JDK 语料 SourceFileAttribute 复现）
        if !t.extends.is_empty() {
            self.out.push_str(&format!(" extends {}", t.extends.join(", ")));
        }
        if !t.permits.is_empty() {
            self.out.push_str(&format!(" permits {}", t.permits.join(", ")));
        }
        if !t.implements.is_empty() {
            self.out
                .push_str(&format!(" implements {}", t.implements.join(", ")));
        }
        self.out.push_str(" {");
        self.level += 1;
        if !t.enum_constants.is_empty() {
            self.newline();
            self.indent();
            self.out.push_str(&t.enum_constants.join(", "));
            if !t.members.is_empty() {
                self.out.push(';');
            }
        }
        for m in &t.members {
            self.newline();
            self.member(m);
        }
        self.level -= 1;
        self.newline();
        self.indent();
        self.out.push('}');
    }

    fn member(&mut self, m: &Member) {
        let ast = self.ast;
        match m {
            Member::Field {
                mods,
                ty,
                declarators,
            } => {
                self.indent();
                self.out.push_str(&mods_prefix(mods));
                self.out.push_str(&ty_str(ty));
                self.out.push(' ');
                let ds: Vec<String> = declarators
                    .iter()
                    .map(|d| declarator_str(ast, ty, d))
                    .collect();
                self.out.push_str(&ds.join(", "));
                self.out.push(';');
            }
            Member::Method {
                mods,
                ty_params,
                ret,
                name,
                params,
                throws,
                body,
            } => {
                self.indent();
                self.out.push_str(&mods_prefix(mods));
                if !ty_params.is_empty() {
                    self.out.push_str(ty_params);
                    self.out.push(' ');
                }
                self.out.push_str(&ty_str(ret));
                self.out.push(' ');
                self.out.push_str(name);
                self.param_list(params);
                self.out.push_str(&throws_str(throws));
                match body {
                    Some(b) => {
                        self.out.push(' ');
                        self.body_stmt(*b);
                    }
                    None => self.out.push(';'),
                }
            }
            Member::Constructor {
                mods,
                ty_params,
                name,
                params,
                throws,
                body,
                compact,
            } => {
                self.indent();
                self.out.push_str(&mods_prefix(mods));
                if !ty_params.is_empty() {
                    self.out.push_str(ty_params);
                    self.out.push(' ');
                }
                self.out.push_str(name);
                if !*compact {
                    self.param_list(params);
                }
                self.out.push_str(&throws_str(throws));
                match body {
                    Some(b) => {
                        self.out.push(' ');
                        self.body_stmt(*b);
                    }
                    None => self.out.push(';'),
                }
            }
            Member::Initializer { is_static, body } => {
                self.indent();
                if *is_static {
                    self.out.push_str("static ");
                }
                self.body_stmt(*body);
            }
            Member::Type(t) => self.type_decl(t),
            Member::Raw(text) => {
                // 原文区域：首行对齐当前缩进，续行保持
                let ind = INDENT.repeat(self.level);
                let mut first = true;
                for line in text.lines() {
                    if !first {
                        self.out.push('\n');
                    }
                    first = false;
                    if !line.trim().is_empty() {
                        self.out.push_str(&ind);
                    }
                    self.out.push_str(line.trim_end());
                }
            }
        }
    }

    fn param_list(&mut self, params: &[Param]) {
        self.out.push('(');
        for (i, p) in params.iter().enumerate() {
            if i > 0 {
                self.out.push_str(", ");
            }
            self.out.push_str(&param_str(p));
        }
        self.out.push(')');
    }

    // ---- 语句 ----

    fn stmt(&mut self, id: JavaId) {
        let ast = self.ast;
        if ast.data(id) == &NodeData::Block {
            self.indent();
            self.block_body(id);
            return;
        }
        match ast.data(id) {
            NodeData::Block => unreachable!(),
            NodeData::Empty => {}
            NodeData::Raw { text } => {
                let ind = INDENT.repeat(self.level);
                let mut first = true;
                for line in text.lines() {
                    if !first {
                        self.out.push('\n');
                    }
                    first = false;
                    if !line.trim().is_empty() {
                        self.out.push_str(&ind);
                    }
                    self.out.push_str(line.trim_end());
                }
            }
            NodeData::ExprStmt => {
                let ch = ast.children(id);
                if ch.is_empty() {
                    // 空表达式语句（解析失败兜底产物）：不打印
                    return;
                }
                self.indent();
                self.expr(ch[0], prec::ASSIGN);
                self.out.push(';');
            }
            NodeData::VarDecl { name, ty } => {
                self.indent();
                self.out.push_str(&ty_str(ty));
                self.out.push(' ');
                self.out.push_str(name);
                if let Some(&init) = ast.children(id).first() {
                    self.out.push_str(" = ");
                    let init = decl_init_expr(ast, ty, init);
                    self.expr(init, prec::ASSIGN);
                }
                self.out.push(';');
            }
            NodeData::Assign { op } => {
                self.indent();
                let ch = ast.children(id);
                self.expr(ch[0], prec::ASSIGN + 1);
                self.out.push(' ');
                self.out.push_str(&assign_symbol(*op));
                self.out.push(' ');
                self.expr(ch[1], prec::ASSIGN);
                self.out.push(';');
            }
            NodeData::If => {
                self.indent();
                self.if_tail(id);
            }
            NodeData::While => {
                self.indent();
                let ch = ast.children(id);
                self.out.push_str("while (");
                self.expr(ch[0], prec::ASSIGN);
                self.out.push_str(") ");
                self.body_stmt(ch[1]);
            }
            NodeData::DoWhile => {
                self.indent();
                let ch = ast.children(id);
                self.out.push_str("do ");
                self.body_stmt(ch[0]);
                self.out.push_str(" while (");
                self.expr(ch[1], prec::ASSIGN);
                self.out.push_str(");");
            }
            NodeData::For {
                inits,
                has_cond,
                steps,
            } => {
                self.indent();
                let ch = ast.children(id);
                let n_steps = *steps as usize;
                let body = *ch.last().unwrap();
                let mut idx = 0;
                self.out.push_str("for (");
                for k in 0..*inits as usize {
                    if k > 0 {
                        self.out.push_str(", ");
                    }
                    self.for_header_part(ch[idx], k == 0);
                    idx += 1;
                }
                self.out.push_str("; ");
                if *has_cond {
                    self.expr(ch[idx], prec::ASSIGN);
                    idx += 1;
                }
                self.out.push_str("; ");
                for k in 0..n_steps {
                    if k > 0 {
                        self.out.push_str(", ");
                    }
                    self.expr(ch[idx], prec::ASSIGN);
                    idx += 1;
                }
                self.out.push_str(") ");
                self.body_stmt(body);
            }
            NodeData::ForEach { name, ty } => {
                self.indent();
                let ch = ast.children(id);
                self.out
                    .push_str(&format!("for ({} {} : ", ty_str(ty), name));
                self.expr(ch[0], prec::ASSIGN);
                self.out.push_str(") ");
                self.body_stmt(ch[1]);
            }
            NodeData::Return => {
                self.indent();
                self.out.push_str("return");
                if let Some(&v) = ast.children(id).first() {
                    self.out.push(' ');
                    self.expr(v, prec::ASSIGN);
                }
                self.out.push(';');
            }
            NodeData::Break { label } => {
                self.indent();
                self.out.push_str("break");
                if let Some(l) = label {
                    self.out.push(' ');
                    self.out.push_str(l);
                }
                self.out.push(';');
            }
            NodeData::Continue { label } => {
                self.indent();
                self.out.push_str("continue");
                if let Some(l) = label {
                    self.out.push(' ');
                    self.out.push_str(l);
                }
                self.out.push(';');
            }
            NodeData::Throw => {
                self.indent();
                self.out.push_str("throw ");
                self.expr(ast.children(id)[0], prec::ASSIGN);
                self.out.push(';');
            }
            NodeData::Try => {
                self.indent();
                let ch = ast.children(id);
                // children: [resource…, try_block, catch…, (finally)?]
                let try_idx = ch
                    .iter()
                    .position(|&c| ast.data(c) == &NodeData::Block)
                    .unwrap_or(0);
                self.out.push_str("try ");
                if try_idx > 0 {
                    self.out.push('(');
                    for (i, &r) in ch[..try_idx].iter().enumerate() {
                        if i > 0 {
                            self.out.push_str("; ");
                        }
                        self.resource(r);
                    }
                    self.out.push_str(") ");
                }
                self.body_stmt(ch[try_idx]);
                for &c in &ch[try_idx + 1..] {
                    if let NodeData::Catch { ty_raw, name } = ast.data(c) {
                        self.out.push_str(&format!(" catch ({ty_raw} {name}) "));
                        let blk = ast.children(c)[0];
                        self.body_stmt(blk);
                    } else {
                        self.out.push_str(" finally ");
                        self.body_stmt(c);
                    }
                }
            }
            NodeData::Synchronized => {
                self.indent();
                let ch = ast.children(id);
                self.out.push_str("synchronized (");
                self.expr(ch[0], prec::ASSIGN);
                self.out.push_str(") ");
                self.body_stmt(ch[1]);
            }
            NodeData::Switch => {
                self.indent();
                self.switch_print(id);
            }
            NodeData::Case { .. } => {
                self.indent();
                self.case(id);
            }
            NodeData::Label { name } => {
                self.indent();
                self.out.push_str(name);
                self.out.push(':');
                let body = ast.children(id)[0];
                if self.ast.data(body) == &NodeData::Empty {
                    // label:;（指向空分号）——直接同行输出分号
                    self.out.push_str(";");
                    return;
                }
                self.level += 1;
                self.newline();
                self.stmt(body);
                self.level -= 1;
            }
            NodeData::Assert => {
                self.indent();
                let ch = ast.children(id);
                self.out.push_str("assert ");
                self.expr(ch[0], prec::TERNARY);
                if let Some(&m) = ch.get(1) {
                    self.out.push_str(" : ");
                    self.expr(m, prec::ASSIGN);
                }
                self.out.push(';');
            }
            _ => {
                // 表达式直接作为语句出现
                self.indent();
                self.expr(id, prec::ASSIGN);
                self.out.push(';');
            }
        }
    }

    /// try-with-resources 资源：语句但不带分号。
    fn resource(&mut self, id: JavaId) {
        let ast = self.ast;
        match ast.data(id) {
            NodeData::VarDecl { name, ty } => {
                self.out.push_str(&ty_str(ty));
                self.out.push(' ');
                self.out.push_str(name);
                if let Some(&init) = ast.children(id).first() {
                    self.out.push_str(" = ");
                    let init = decl_init_expr(ast, ty, init);
                    self.expr(init, prec::ASSIGN);
                }
            }
            _ => {
                // 表达式资源（try (stream)——JEP 提案形态/GJF testdata i155）：
                // 解析侧包了 ExprStmt，解包后打印（曾打 /* ExprStmt */ 占位
                // 符 → 重解析形态漂移 → 幂等失败）
                let target = if ast.data(id) == &NodeData::ExprStmt {
                    ast.children(id).first().copied().unwrap_or(id)
                } else {
                    id
                };
                self.expr(target, prec::ASSIGN);
            }
        }
    }

    fn case(&mut self, id: JavaId) {
        let ast = self.ast;
        let NodeData::Case {
            labels,
            is_default,
            arrow,
        } = ast.data(id)
        else {
            self.out.push_str("/* bad case */");
            return;
        };
        let ch = ast.children(id);
        let (label_ids, stmt_ids) = ch.split_at(*labels as usize);
        let label_text = |l: JavaId| -> String {
            let mut lp = Printer {
                ast: self.ast,
                out: String::new(),
                level: 0,
            };
            lp.expr(l, prec::TERNARY);
            lp.out
        };
        if *arrow {
            let mut head = String::new();
            let texts: Vec<String> = label_ids.iter().map(|&l| label_text(l)).collect();
            if !texts.is_empty() {
                head.push_str(&format!("case {}", texts.join(", ")));
                if *is_default {
                    head.push_str(", default");
                }
            } else {
                head.push_str("default");
            }
            self.out.push_str(&head);
            self.out.push_str(" -> ");
            if stmt_ids.len() == 1 && ast.data(stmt_ids[0]) == &NodeData::Block {
                self.block_body(stmt_ids[0]);
            } else if stmt_ids.len() == 1 {
                // 内联渲染单语句（无前导缩进）
                let mut sp = Printer {
                    ast: self.ast,
                    out: String::new(),
                    level: 0,
                };
                sp.stmt(stmt_ids[0]);
                self.out.push_str(&sp.out);
            } else {
                self.out.push('{');
                self.level += 1;
                for &s in stmt_ids {
                    self.newline();
                    self.stmt(s);
                }
                self.level -= 1;
                self.newline();
                self.indent();
                self.out.push('}');
            }
        } else {
            // 经典 fallthrough：每个标签一行
            let mut first = true;
            for &l in label_ids {
                if !first {
                    self.newline();
                    self.indent();
                }
                first = false;
                self.out.push_str("case ");
                self.out.push_str(&label_text(l));
                self.out.push(':');
            }
            if *is_default {
                self.out.push_str("default:");
            }
            self.level += 1;
            for &s in stmt_ids {
                self.newline();
                self.stmt(s);
            }
            self.level -= 1;
        }
    }

    /// switch 主体（语句与表达式位置通用）。
    fn switch_print(&mut self, id: JavaId) {
        let ch = self.ast.children(id).to_vec();
        self.out.push_str("switch (");
        self.expr(ch[0], prec::ASSIGN);
        self.out.push_str(") {");
        self.level += 1;
        for &c in &ch[1..] {
            self.newline();
            self.indent();
            self.case(c);
        }
        self.level -= 1;
        self.newline();
        self.indent();
        self.out.push('}');
    }

    /// if/else 链（else if 不额外嵌套缩进）。
    fn if_tail(&mut self, id: JavaId) {
        let ast = self.ast;
        let ch = ast.children(id);
        self.out.push_str("if (");
        self.expr(ch[0], prec::ASSIGN);
        self.out.push_str(") ");
        self.body_stmt(ch[1]);
        if let Some(&els) = ch.get(2) {
            self.out.push_str(" else ");
            if ast.data(els) == &NodeData::If {
                self.if_tail(els);
            } else {
                self.body_stmt(els);
            }
        }
    }

    /// for 头部的初始化段（VarDecl 无分号 / 表达式）。
    /// `with_type=false` 时 VarDecl 只输出名字（`int i = 0, j = 1` 的后续项）。
    fn for_header_part(&mut self, id: JavaId, with_type: bool) {
        let ast = self.ast;
        if let NodeData::VarDecl { name, ty } = ast.data(id) {
            if with_type {
                self.out.push_str(&ty_str(ty));
                self.out.push(' ');
            }
            self.out.push_str(name);
            if let Some(&init) = ast.children(id).first() {
                self.out.push_str(" = ");
                let init = decl_init_expr(ast, ty, init);
                self.expr(init, prec::ASSIGN);
            }
            return;
        }
        // 表达式 init：for (i = 0; …)——解析侧包着 ExprStmt（或裸表达式）
        let expr = match ast.data(id) {
            NodeData::ExprStmt => *ast.children(id).first().unwrap_or(&id),
            _ => id,
        };
        self.expr(expr, prec::ASSIGN);
        // 非 VarDecl 非 ExprStmt 的罕见形态：交给 expr 输出（不再打占位注释——
        // `/* bad init */` 会污染合法 for 头并使输出无法重解析）
    }

    /// 控制流体：Block 直接展开，单语句换行缩进。
    fn body_stmt(&mut self, id: JavaId) {
        if self.ast.data(id) == &NodeData::Block {
            self.block_body(id);
        } else if self.ast.data(id) == &NodeData::Empty {
            // 控制流裸分号体（while(x); / label: while(x);）——必须输出 `;`，
            // 否则 while 无体也无关联语句（FooLabel 真实语料：输出不可重解析）
            self.out.push(';');
        } else {
            self.level += 1;
            self.newline();
            self.stmt(id);
            self.level -= 1;
        }
    }

    fn block_body(&mut self, id: JavaId) {
        let children = self.ast.children(id).to_vec();
        if children.is_empty() {
            self.out.push_str("{}");
            return;
        }
        self.out.push('{');
        self.level += 1;
        for &s in &children {
            if self.stmt_is_blank(s) {
                continue;
            }
            self.newline();
            self.stmt(s);
        }
        self.level -= 1;
        self.newline();
        self.indent();
        self.out.push('}');
    }

    /// 空/无输出语句（Empty、空 ExprStmt、空文本 Raw）——跳过不占行。
    fn stmt_is_blank(&self, id: JavaId) -> bool {
        match self.ast.data(id) {
            NodeData::Empty => true,
            NodeData::ExprStmt => self.ast.children(id).is_empty(),
            NodeData::Raw { text } => text.trim().is_empty(),
            NodeData::Block => self.ast.children(id).iter().all(|&c| self.stmt_is_blank(c)),
            _ => false,
        }
    }

    // ---- 表达式 ----

    fn expr(&mut self, id: JavaId, min_prec: u8) {
        let ast = self.ast;
        match ast.data(id) {
            NodeData::Binary { op } => {
                let p = bin_prec(*op);
                let ch = ast.children(id);
                let need = p < min_prec;
                if need {
                    self.out.push('(');
                }
                self.expr(ch[0], p);
                self.out.push(' ');
                self.out.push_str(bin_symbol(*op));
                self.out.push(' ');
                self.expr(ch[1], p + 1);
                if need {
                    self.out.push(')');
                }
            }
            NodeData::InstanceOf { ty, bind } => {
                let e = ast.children(id)[0];
                let need = prec::RELATIONAL < min_prec;
                if need {
                    self.out.push('(');
                }
                self.expr(e, prec::RELATIONAL + 1);
                self.out.push_str(" instanceof ");
                self.out.push_str(&ty_str(ty));
                if let Some(b) = bind {
                    self.out.push(' ');
                    self.out.push_str(b);
                }
                if need {
                    self.out.push(')');
                }
            }
            NodeData::Unary { op } => {
                let postfix = matches!(op, UnOp::PostInc | UnOp::PostDec);
                let ch = ast.children(id);
                let need = prec::UNARY < min_prec;
                if need {
                    self.out.push('(');
                }
                if postfix {
                    self.expr(ch[0], prec::POSTFIX);
                    self.out.push_str(un_symbol(*op));
                } else {
                    self.out.push_str(un_symbol(*op));
                    // `-(-x)` 必须括号：裸拼会输出 `--x`，词法层变为前置自减
                    // （负字面量同理：`--5`）。`~(~x)`/`-(-x)` 中只有 Neg 有此
                    // 二义（`~~x` 合法、`-++x`/`-​--x` 词法可分）。
                    // 负字面量同样二义（解析器把 -5 折成字面量节点：Int/NumRaw）
                    let neg_lit = match ast.data(ch[0]) {
                        NodeData::Literal(Lit::Int(v)) => *v < 0,
                        NodeData::Literal(Lit::Long(v)) => *v < 0,
                        NodeData::Literal(Lit::Float(v)) => *v < 0.0,
                        NodeData::Literal(Lit::Double(v)) => *v < 0.0,
                        NodeData::Literal(Lit::NumRaw { text, .. }) => text.starts_with('-'),
                        _ => false,
                    };
                    let amb_neg = *op == UnOp::Neg
                        && (matches!(ast.data(ch[0]), NodeData::Unary { op: UnOp::Neg }) || neg_lit);
                    if amb_neg {
                        self.out.push('(');
                    }
                    self.expr(ch[0], prec::UNARY);
                    if amb_neg {
                        self.out.push(')');
                    }
                }
                if need {
                    self.out.push(')');
                }
            }
            NodeData::Ternary => {
                let ch = ast.children(id);
                let need = prec::TERNARY < min_prec;
                if need {
                    self.out.push('(');
                }
                self.expr(ch[0], prec::TERNARY + 1);
                self.out.push_str(" ? ");
                self.expr(ch[1], prec::TERNARY);
                self.out.push_str(" : ");
                self.expr(ch[2], prec::TERNARY);
                if need {
                    self.out.push(')');
                }
            }
            NodeData::Assign { op } => {
                let ch = ast.children(id);
                let need = prec::ASSIGN < min_prec;
                if need {
                    self.out.push('(');
                }
                self.expr(ch[0], prec::ASSIGN + 1);
                self.out.push(' ');
                self.out.push_str(&assign_symbol(*op));
                self.out.push(' ');
                self.expr(ch[1], prec::ASSIGN);
                if need {
                    self.out.push(')');
                }
            }
            NodeData::Call => {
                let ch = ast.children(id);
                let need = prec::POSTFIX < min_prec;
                if need {
                    self.out.push('(');
                }
                // 限定 new + 匿名类体：callee 名形如 "new X<…> {body…}"，合法
                // 顺序是 recv.new X(args) {body}（JLS 15.9.1——体在实参之后；
                // 曾打成 recv.new X {body}(args)，往返失败，spoon
                // ProblemReferenceBinding 抓获）
                let mut qual_new_anon = None;
                if let NodeData::Member { name } = ast.data(ch[0]) {
                    let name = ast.sn(*name);
                    if name.starts_with("new ") {
                        if let Some(i) = name.find(" {") {
                            qual_new_anon = Some(i);
                        }
                    }
                }
                if let Some(i) = qual_new_anon {
                    let name = ast
                        .sn(match ast.data(ch[0]) {
                            NodeData::Member { name } => *name,
                            _ => Sym::default(),
                        })
                        .to_string();
                    let name = name.as_str();
                    let (head, body) = (&name[..i], &name[i + 1..]);
                    self.expr(ast.children(ch[0])[0], prec::POSTFIX);
                    self.out.push('.');
                    self.out.push_str(head);
                    self.out.push('(');
                    for (j, &a) in ch[1..].iter().enumerate() {
                        if j > 0 {
                            self.out.push_str(", ");
                        }
                        self.expr(a, prec::ASSIGN);
                    }
                    self.out.push(')');
                    self.out.push(' ');
                    self.out.push_str(body);
                } else {
                    let sw_recv = !ch.is_empty() && ast.kind(ch[0]) == NodeKind::Switch;
                    if sw_recv {
                        self.out.push('(');
                    }
                    self.expr(ch[0], prec::POSTFIX);
                    if sw_recv {
                        self.out.push(')');
                    }
                    self.out.push('(');
                    for (i, &a) in ch[1..].iter().enumerate() {
                        if i > 0 {
                            self.out.push_str(", ");
                        }
                        self.expr(a, prec::ASSIGN);
                    }
                    self.out.push(')');
                }
                if need {
                    self.out.push(')');
                }
            }
            NodeData::Member { name } => {
                let name = ast.sn(*name);
                let name = name;
                let obj = ast.children(id)[0];
                let need = prec::POSTFIX < min_prec;
                // Switch 表达式作接收方必须带括号：switch (s) {…}.x 非法
                //（openjdk ConditionalWithVoid 往返抓获——打印丢括号 → 重解析失败）
                let switch_recv = ast.kind(obj) == NodeKind::Switch;
                if need || switch_recv {
                    self.out.push('(');
                }
                self.expr(obj, prec::POSTFIX);
                if need || switch_recv {
                    self.out.push(')');
                }
                // 数组类型前缀（"[]class"）：维度在点号前——int[].class
                let (dims, rest) = if name.starts_with("[]") {
                    let d = name[..name.find("class").unwrap_or(name.len())].to_string();
                    (d, &name[name.find("class").unwrap_or(0)..])
                } else {
                    (String::new(), name)
                };
                self.out.push_str(&dims);
                self.out.push('.');
                self.out.push_str(rest);
            }
            NodeData::MethodRef { name } => {
                let name = ast.sn(*name);
                let recv = ast.children(id)[0];
                let need = prec::POSTFIX < min_prec;
                if need {
                    self.out.push('(');
                }
                self.expr(recv, prec::POSTFIX);
                // 数组类型方法引用：name 形如 "[]::new"/"[][]::m"——
                // 维度属接收方类型（T[]::new），须在 :: 之前输出
                let (dims, rest) = match name.find("::") {
                    Some(i) if name.starts_with("[]") => (&name[..i], &name[i + 2..]),
                    _ => ("", name),
                };
                self.out.push_str(dims);
                self.out.push_str("::");
                self.out.push_str(rest);
                if need {
                    self.out.push(')');
                }
            }
            NodeData::Index => {
                let ch = ast.children(id);
                let need = prec::POSTFIX < min_prec;
                if need {
                    self.out.push('(');
                }
                self.expr(ch[0], prec::POSTFIX);
                self.out.push('[');
                self.expr(ch[1], prec::ASSIGN);
                self.out.push(']');
                if need {
                    self.out.push(')');
                }
            }
            NodeData::Cast { ty } => {
                let e = ast.children(id)[0];
                let need = prec::UNARY < min_prec;
                if need {
                    self.out.push('(');
                }
                self.out.push('(');
                self.out.push_str(&ty_str(ty));
                self.out.push_str(") ");
                self.expr(e, prec::UNARY);
                if need {
                    self.out.push(')');
                }
            }
            NodeData::New { ty, anon_raw } => {
                let args = ast.children(id);
                let need = prec::POSTFIX < min_prec;
                if need {
                    self.out.push('(');
                }
                self.out.push_str("new ");
                self.out.push_str(&ty_str(ty));
                self.out.push('(');
                for (i, &a) in args.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    self.expr(a, prec::ASSIGN);
                }
                self.out.push(')');
                if let Some(raw) = anon_raw {
                    self.out.push(' ');
                    self.out.push_str(raw);
                }
                if need {
                    self.out.push(')');
                }
            }
            NodeData::NewArray { ty, dims, sized } => {
                let ch = ast.children(id);
                let need = prec::POSTFIX < min_prec;
                if need {
                    self.out.push('(');
                }
                self.out.push_str("new ");
                self.out.push_str(&ty_str(ty));
                for i in 0..*dims {
                    self.out.push('[');
                    if (i as u16) < *sized {
                        self.expr(ch[i as usize], prec::ASSIGN);
                    }
                    self.out.push(']');
                }
                if let Some(&init) = ch.get(*sized as usize) {
                    if ast.data(init) == &NodeData::ArrayLit {
                        self.out.push(' ');
                        self.expr(init, prec::PRIMARY);
                    }
                }
                if need {
                    self.out.push(')');
                }
            }
            NodeData::Group => {
                // 合成分组：无作用域、无括号——子语句同层展开
                let children = self.ast.children(id).to_vec();
                for &s in &children {
                    self.stmt(s);
                }
            }
            NodeData::ArrayLit => {
                let elems = ast.children(id);
                self.out.push('{');
                for (i, &e) in elems.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    self.expr(e, prec::ASSIGN);
                }
                self.out.push('}');
            }
            NodeData::Switch => self.switch_print(id),
            NodeData::Raw { text } => self.out.push_str(text),
            NodeData::VarRef { name } => self.out.push_str(ast.sn(*name)),
            NodeData::This => self.out.push_str("this"),
            NodeData::Super => self.out.push_str("super"),
            NodeData::Literal(l) => self.out.push_str(&lit_str(l)),
            NodeData::Lambda { params_raw } => {
                let body = ast.children(id)[0];
                let need = prec::ASSIGN < min_prec;
                if need {
                    self.out.push('(');
                }
                self.out.push('(');
                self.out.push_str(params_raw);
                self.out.push_str(") -> ");
                if ast.data(body) == &NodeData::Block {
                    self.body_stmt(body);
                } else {
                    self.expr(body, prec::ASSIGN);
                }
                if need {
                    self.out.push(')');
                }
            }
            NodeData::Paren => {
                let e = ast.children(id)[0];
                self.out.push('(');
                self.expr(e, prec::ASSIGN);
                self.out.push(')');
            }
            _ => {
                // 未预期的表达式位置节点：保底打印
                self.out.push_str(&format!("/* {:?} */", ast.data(id)));
            }
        }
    }
}

fn ensure_frac(v: f64) -> String {
    let s = format!("{v}");
    if s.contains('.') || s.contains('e') || s.contains("inf") || s.contains("NaN") {
        s
    } else {
        format!("{s}.0")
    }
}

fn lit_str(l: &Lit) -> String {
    match l {
        Lit::Bool(true) => "true".into(),
        Lit::Bool(false) => "false".into(),
        Lit::Int(v) => format!("{v}"),
        Lit::Long(v) => format!("{v}L"),
        Lit::Float(v) => format!("{}f", ensure_frac(*v)),
        Lit::Double(v) => ensure_frac(*v),
        Lit::Char(c) => format!("'{}'", escape_char(*c)),
        Lit::Str(s) => {
            let mut out = String::with_capacity(s.len() + 2);
            out.push('"');
            for c in s.chars() {
                push_escaped(&mut out, c);
            }
            out.push('"');
            out
        }
        Lit::TextBlock(s) => {
            format!("\"\"\"\n{s}\"\"\"")
        }
        Lit::NumRaw { text, .. } => text.clone(),
        Lit::Null => "null".into(),
    }
}

fn push_escaped(out: &mut String, c: char) {
    match c {
        '"' => out.push_str("\\\""),
        '\\' => out.push_str("\\\\"),
        '\n' => out.push_str("\\n"),
        '\t' => out.push_str("\\t"),
        '\r' => out.push_str("\\r"),
        // 其余控制字符：\uXXXX（\u000a/\u000d 会被 JLS 预处理成行终止符，
        // 必须用上面的专用转义；其余控制码无此问题）
        c if (c as u32) < 0x20 || c as u32 == 0x7f => {
            out.push_str(&format!("\\u{:04X}", c as u32));
        }
        c => out.push(c),
    }
}

fn escape_char(c: char) -> String {
    match c {
        '\'' => "\\'".to_string(),
        '\\' => "\\\\".to_string(),
        '\n' => "\\n".to_string(),
        '\t' => "\\t".to_string(),
        '\r' => "\\r".to_string(),
        c if (c as u32) < 0x20 || c as u32 == 0x7f => format!("\\u{:04X}", c as u32),
        c => c.to_string(),
    }
}
