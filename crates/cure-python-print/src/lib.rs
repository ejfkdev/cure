//! Python 打印器：4 空格缩进、表达式优先级最小括号、原文区域逐行缩进。
//! 目标：输出可被 CPython 3.15 `ast.parse` 接受，且 AST 与输入等价。

use cure_python_ast::{CompKind, NodeData, PyAst, PyBin, PyBoolOp, PyId, PyUn};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
    Lambda,
    IfExp,
    Or,
    And,
    Not,
    Cmp,
    BitOr,
    BitXor,
    BitAnd,
    Shift,
    Add,
    Mul,
    Unary,
    Power,
    Await,
    Primary,
}

impl Prec {
    fn succ(self) -> Prec {
        match self {
            Prec::Lambda => Prec::IfExp,
            Prec::IfExp => Prec::Or,
            Prec::Or => Prec::And,
            Prec::And => Prec::Not,
            Prec::Not => Prec::Cmp,
            Prec::Cmp => Prec::BitOr,
            Prec::BitOr => Prec::BitXor,
            Prec::BitXor => Prec::BitAnd,
            Prec::BitAnd => Prec::Shift,
            Prec::Shift => Prec::Add,
            Prec::Add => Prec::Mul,
            Prec::Mul => Prec::Unary,
            Prec::Unary => Prec::Power,
            Prec::Power => Prec::Await,
            Prec::Await | Prec::Primary => Prec::Primary,
        }
    }
}

pub fn print_module(ast: &PyAst, module: PyId) -> String {
    let mut p = Printer {
        ast,
        out: String::new(),
        indent: 0,
    };
    for &stmt in ast.children(module) {
        p.print_stmt(stmt);
    }
    let mut s = p.out;
    while s.ends_with('\n') {
        s.pop();
    }
    s.push('\n');
    s
}

struct Printer<'a> {
    ast: &'a PyAst,
    out: String,
    indent: usize,
}

impl<'a> Printer<'a> {
    fn ind(&mut self) {
        for _ in 0..self.indent {
            self.out.push_str("    ");
        }
    }

    fn line(&mut self, s: &str) {
        self.ind();
        self.out.push_str(s);
        self.out.push('\n');
    }

    fn blank_block(&mut self) {
        self.ind();
        self.out.push_str("pass\n");
    }

    fn print_block(&mut self, block: PyId) {
        let ch = self.ast.children(block);
        self.indent += 1;
        if ch.is_empty() {
            self.blank_block();
        } else {
            for &s in ch {
                self.print_stmt(s);
            }
        }
        self.indent -= 1;
    }

    fn print_stmt(&mut self, id: PyId) {
        match self.ast.data(id) {
            NodeData::ExprStmt => {
                let e = self.ast.children(id)[0];
                let text = self.expr_string(e, Prec::Lambda);
                self.line(&text);
            }
            NodeData::Assign { .. } => {
                let ch = self.ast.children(id);
                let n = match self.ast.data(id) {
                    NodeData::Assign { targets } => *targets,
                    _ => 0,
                };
                let mut s = String::new();
                for (i, &t) in ch[..n].iter().enumerate() {
                    if i > 0 {
                        s.push_str(" = ");
                    }
                    s.push_str(&self.expr_string(t, Prec::Lambda));
                }
                s.push_str(" = ");
                s.push_str(&self.expr_string(ch[n], Prec::Lambda));
                self.line(&s);
            }
            NodeData::AugAssign { op } => {
                let ch = self.ast.children(id);
                let s = format!(
                    "{} {}= {}",
                    self.expr_string(ch[0], Prec::Lambda),
                    op.symbol(),
                    self.expr_string(ch[1], Prec::Lambda)
                );
                self.line(&s);
            }
            NodeData::AnnAssign { annotation_raw } => {
                let ch = self.ast.children(id);
                let mut s = format!(
                    "{}: {}",
                    self.expr_string(ch[0], Prec::Lambda),
                    annotation_raw.trim()
                );
                if ch.len() > 1 {
                    s.push_str(" = ");
                    s.push_str(&self.expr_string(ch[1], Prec::Lambda));
                }
                self.line(&s);
            }
            NodeData::FunctionDef {
                name,
                params_raw,
                ret_raw,
                deco_raw,
                generics_raw,
                is_async,
            } => {
                for line in deco_raw.lines().filter(|l| !l.trim().is_empty()) {
                    self.line(line.trim_end());
                }
                let mut s = String::new();
                if *is_async {
                    s.push_str("async ");
                }
                s.push_str("def ");
                s.push_str(name);
                s.push_str(generics_raw);
                s.push('(');
                let pr = params_raw.trim();
                s.push_str(pr);
                // 参数表末行若带 `#` 注释，`)` 会被注释吞掉 → 换行再闭合
                if pr.rsplit('\n').next().unwrap_or(pr).contains('#') {
                    s.push('\n');
                    for _ in 0..=self.indent {
                        s.push_str("    ");
                    }
                }
                s.push(')');
                if !ret_raw.trim().is_empty() {
                    s.push_str(" -> ");
                    s.push_str(ret_raw.trim());
                }
                s.push(':');
                self.line(&s);
                let body = self.ast.children(id)[0];
                self.print_block(body);
            }
            NodeData::ClassDef {
                name,
                header_raw,
                deco_raw,
            } => {
                for line in deco_raw.lines().filter(|l| !l.trim().is_empty()) {
                    self.line(line.trim_end());
                }
                let mut s = format!("class {name}");
                s.push_str(header_raw);
                s.push(':');
                self.line(&s);
                let body = self.ast.children(id)[0];
                self.print_block(body);
            }
            NodeData::If => self.print_if(id, "if"),
            NodeData::While => {
                let ch = self.ast.children(id);
                let s = format!("while {}:", self.expr_string(ch[0], Prec::Lambda));
                self.line(&s);
                self.print_block(ch[1]);
                if ch.len() > 2 {
                    self.line("else:");
                    self.print_block(ch[2]);
                }
            }
            NodeData::For {
                target_raw,
                is_async,
            } => {
                let ch = self.ast.children(id);
                let mut s = String::new();
                if *is_async {
                    s.push_str("async ");
                }
                s.push_str("for ");
                s.push_str(target_raw.trim());
                s.push_str(" in ");
                s.push_str(&self.expr_string(ch[0], Prec::Lambda));
                s.push(':');
                self.line(&s);
                self.print_block(ch[1]);
                if ch.len() > 2 {
                    self.line("else:");
                    self.print_block(ch[2]);
                }
            }
            NodeData::Try {
                handler_count,
                has_else,
                has_finally,
                ..
            } => {
                let ch = self.ast.children(id);
                self.line("try:");
                self.print_block(ch[0]);
                let hc = *handler_count;
                for &h in &ch[1..1 + hc] {
                    if let NodeData::ExceptHandler { clause_raw } = self.ast.data(h) {
                        let body = self.ast.children(h)[0];
                        self.line(&format!("{}:", clause_raw.trim()));
                        self.print_block(body);
                    }
                }
                let mut idx = 1 + hc;
                if *has_else {
                    self.line("else:");
                    self.print_block(ch[idx]);
                    idx += 1;
                }
                if *has_finally {
                    self.line("finally:");
                    self.print_block(ch[idx]);
                }
            }
            NodeData::With {
                items_raw,
                is_async,
            } => {
                let body = self.ast.children(id)[0];
                let mut s = String::new();
                if *is_async {
                    s.push_str("async ");
                }
                s.push_str("with ");
                s.push_str(items_raw.trim());
                s.push(':');
                self.line(&s);
                self.print_block(body);
            }
            NodeData::Match => {
                let ch = self.ast.children(id);
                let subj = self.expr_string(ch[0], Prec::Lambda);
                self.line(&format!("match {subj}:"));
                self.indent += 1;
                for &case in &ch[1..] {
                    self.print_case(case);
                }
                self.indent -= 1;
            }
            NodeData::Return => {
                let ch = self.ast.children(id);
                if ch.is_empty() {
                    self.line("return");
                } else {
                    let s = format!("return {}", self.expr_string(ch[0], Prec::Lambda));
                    self.line(&s);
                }
            }
            NodeData::Raise { from } => {
                let ch = self.ast.children(id);
                let mut s = String::from("raise");
                if let Some(&e) = ch.first() {
                    s.push(' ');
                    s.push_str(&self.expr_string(e, Prec::Lambda));
                }
                if *from {
                    if let Some(&c) = ch.get(1) {
                        s.push_str(" from ");
                        s.push_str(&self.expr_string(c, Prec::Lambda));
                    }
                }
                self.line(&s);
            }
            NodeData::Assert => {
                let ch = self.ast.children(id);
                let mut s = format!("assert {}", self.expr_string(ch[0], Prec::Lambda));
                if ch.len() > 1 {
                    s.push_str(", ");
                    s.push_str(&self.expr_string(ch[1], Prec::Lambda));
                }
                self.line(&s);
            }
            NodeData::Delete => {
                let ch = self.ast.children(id);
                let list: Vec<String> = ch
                    .iter()
                    .map(|&e| self.expr_string(e, Prec::Lambda))
                    .collect();
                self.line(&format!("del {}", list.join(", ")));
            }
            NodeData::Import(raw) => self.line(raw.trim()),
            NodeData::FromImport(raw) => self.line(raw.trim()),
            NodeData::Global(raw) => self.line(&format!("global {}", raw.trim())),
            NodeData::Nonlocal(raw) => self.line(&format!("nonlocal {}", raw.trim())),
            NodeData::Pass => self.line("pass"),
            NodeData::Break => self.line("break"),
            NodeData::Continue => self.line("continue"),
            NodeData::RawStmt(raw) => {
                let base = self.indent;
                let text = raw.trim_end();
                for line in text.lines() {
                    if line.is_empty() {
                        self.out.push('\n');
                    } else {
                        self.indent = base;
                        self.ind();
                        self.out.push_str(line);
                        self.out.push('\n');
                    }
                }
            }
            _ => {
                // 表达式被误放在语句位（解析器不会产生；保守原文）
                self.line("pass");
            }
        }
    }

    fn print_if(&mut self, id: PyId, kw: &str) {
        let ch = self.ast.children(id);
        let s = format!("{kw} {}:", self.expr_string(ch[0], Prec::Lambda));
        self.line(&s);
        self.print_block(ch[1]);
        if ch.len() > 2 {
            let orelse = ch[2];
            let kids = self.ast.children(orelse);
            if kids.len() == 1 && matches!(self.ast.data(kids[0]), NodeData::If) {
                self.print_if(kids[0], "elif");
            } else {
                self.line("else:");
                self.print_block(orelse);
            }
        }
    }

    fn print_case(&mut self, id: PyId) {
        let NodeData::Case {
            pattern_raw,
            has_guard,
        } = self.ast.data(id)
        else {
            return;
        };
        let ch = self.ast.children(id);
        let (guard, body) = if *has_guard {
            (Some(ch[0]), ch[1])
        } else {
            (None, ch[0])
        };
        let mut s = format!("case {}", pattern_raw.trim());
        if let Some(g) = guard {
            s.push_str(" if ");
            s.push_str(&self.expr_string(g, Prec::Lambda));
        }
        s.push(':');
        self.line(&s);
        self.print_block(body);
    }

    fn expr_prec(&self, id: PyId) -> Prec {
        match self.ast.data(id) {
            NodeData::Lambda { .. } | NodeData::Yield { .. } => Prec::Lambda,
            NodeData::IfExp => Prec::IfExp,
            NodeData::BoolOp { op } => match op {
                PyBoolOp::Or => Prec::Or,
                PyBoolOp::And => Prec::And,
            },
            NodeData::Unary { op: PyUn::Not } => Prec::Not,
            NodeData::Compare { .. } => Prec::Cmp,
            NodeData::Binary { op } => match op {
                PyBin::BitOr => Prec::BitOr,
                PyBin::BitXor => Prec::BitXor,
                PyBin::BitAnd => Prec::BitAnd,
                PyBin::LShift | PyBin::RShift => Prec::Shift,
                PyBin::Add | PyBin::Sub => Prec::Add,
                PyBin::Mul | PyBin::Div | PyBin::FloorDiv | PyBin::Mod | PyBin::MatMul => Prec::Mul,
                PyBin::Pow => Prec::Power,
            },
            NodeData::Unary { .. } => Prec::Unary,
            NodeData::Await => Prec::Await,
            NodeData::NamedExpr { .. } => Prec::Primary,
            _ => Prec::Primary,
        }
    }

    fn expr_string(&mut self, id: PyId, parent: Prec) -> String {
        let prec = self.expr_prec(id);
        let need = prec < parent;
        if need {
            let saved = std::mem::take(&mut self.out);
            self.out.push('(');
            self.print_expr(id);
            self.out.push(')');
            let s = std::mem::replace(&mut self.out, saved);
            return s;
        }
        let saved = std::mem::take(&mut self.out);
        self.print_expr(id);
        std::mem::replace(&mut self.out, saved)
    }

    fn print_expr(&mut self, id: PyId) {
        match self.ast.data(id) {
            NodeData::Name(n) => self.out.push_str(n),
            NodeData::Literal(l) => self.out.push_str(&l.raw),
            NodeData::Raw(text) => self.out.push_str(text.trim()),
            NodeData::Tuple => {
                let ch = self.ast.children(id).to_vec();
                self.out.push('(');
                for (i, &c) in ch.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    {
                        let s = self.expr_string(c, Prec::Lambda);
                        self.out.push_str(&s);
                    }
                }
                if ch.len() == 1 {
                    self.out.push(',');
                }
                self.out.push(')');
            }
            NodeData::List => {
                let ch = self.ast.children(id).to_vec();
                self.out.push('[');
                for (i, &c) in ch.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    {
                        let s = self.expr_string(c, Prec::Lambda);
                        self.out.push_str(&s);
                    }
                }
                self.out.push(']');
            }
            NodeData::Set => {
                let ch = self.ast.children(id).to_vec();
                if ch.is_empty() {
                    self.out.push_str("set()");
                } else {
                    self.out.push('{');
                    for (i, &c) in ch.iter().enumerate() {
                        if i > 0 {
                            self.out.push_str(", ");
                        }
                        {
                            let s = self.expr_string(c, Prec::Lambda);
                            self.out.push_str(&s);
                        }
                    }
                    self.out.push('}');
                }
            }
            NodeData::Dict => {
                let ch = self.ast.children(id).to_vec();
                self.out.push('{');
                let mut i = 0;
                let mut first = true;
                while i < ch.len() {
                    if !first {
                        self.out.push_str(", ");
                    }
                    first = false;
                    if matches!(self.ast.data(ch[i]), NodeData::DoubleStar) {
                        self.out.push_str("**");
                        let v = self.ast.children(ch[i])[0];
                        {
                            let s = self.expr_string(v, Prec::Unary);
                            self.out.push_str(&s);
                        }
                        i += 1;
                    } else {
                        let k = ch[i];
                        let v = ch.get(i + 1).copied().unwrap_or(k);
                        {
                            let s = self.expr_string(k, Prec::Lambda);
                            self.out.push_str(&s);
                        }
                        self.out.push_str(": ");
                        {
                            let s = self.expr_string(v, Prec::Lambda);
                            self.out.push_str(&s);
                        }
                        i += 2;
                    }
                }
                self.out.push('}');
            }
            NodeData::Binary { op } => {
                let ch = self.ast.children(id).to_vec();
                let p = self.expr_prec(id);
                let (lp, rp) = if *op == PyBin::Pow {
                    (Prec::Unary, Prec::Power)
                } else {
                    (p, p.succ())
                };
                {
                    let s = self.expr_string(ch[0], lp);
                    self.out.push_str(&s);
                }
                self.out.push(' ');
                self.out.push_str(op.symbol());
                self.out.push(' ');
                {
                    let s = self.expr_string(ch[1], rp);
                    self.out.push_str(&s);
                }
            }
            NodeData::Unary { op } => {
                let v = self.ast.children(id)[0];
                if *op == PyUn::Not {
                    self.out.push_str("not ");
                    {
                        let s = self.expr_string(v, Prec::Not);
                        self.out.push_str(&s);
                    }
                } else {
                    self.out.push_str(op.symbol());
                    {
                        let s = self.expr_string(v, Prec::Unary);
                        self.out.push_str(&s);
                    }
                }
            }
            NodeData::BoolOp { op } => {
                let ch = self.ast.children(id).to_vec();
                let p = self.expr_prec(id);
                for (i, &c) in ch.iter().enumerate() {
                    if i > 0 {
                        self.out.push(' ');
                        self.out.push_str(op.symbol());
                        self.out.push(' ');
                    }
                    {
                        let s = self.expr_string(c, p);
                        self.out.push_str(&s);
                    }
                }
            }
            NodeData::Compare { ops } => {
                let ch = self.ast.children(id).to_vec();
                for (i, &c) in ch.iter().enumerate() {
                    if i > 0 {
                        self.out.push(' ');
                        self.out.push_str(ops[i - 1].symbol());
                        self.out.push(' ');
                    }
                    {
                        let s = self.expr_string(c, Prec::BitOr);
                        self.out.push_str(&s);
                    }
                }
            }
            NodeData::IfExp => {
                let ch = self.ast.children(id).to_vec();
                {
                    let s = self.expr_string(ch[0], Prec::Or);
                    self.out.push_str(&s);
                }
                self.out.push_str(" if ");
                {
                    let s = self.expr_string(ch[1], Prec::Or);
                    self.out.push_str(&s);
                }
                self.out.push_str(" else ");
                {
                    let s = self.expr_string(ch[2], Prec::IfExp);
                    self.out.push_str(&s);
                }
            }
            NodeData::Lambda { params_raw } => {
                let body = self.ast.children(id)[0];
                self.out.push_str("lambda");
                let p = params_raw.trim();
                if !p.is_empty() {
                    self.out.push(' ');
                    self.out.push_str(p);
                }
                self.out.push_str(": ");
                {
                    let s = self.expr_string(body, Prec::Lambda);
                    self.out.push_str(&s);
                }
            }
            NodeData::Call { args } => {
                let ch = self.ast.children(id).to_vec();
                {
                    let s = self.expr_string(ch[0], Prec::Primary);
                    self.out.push_str(&s);
                }
                self.out.push('(');
                for (i, &a) in ch[1..].iter().enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    match args.get(i) {
                        Some(cure_python_ast::CallArg {
                            kind: cure_python_ast::ArgKind::Keyword,
                            name,
                        }) => {
                            if let Some(n) = name {
                                self.out.push_str(n);
                                self.out.push('=');
                            }
                            {
                                let s = self.expr_string(a, Prec::Lambda);
                                self.out.push_str(&s);
                            }
                        }
                        Some(cure_python_ast::CallArg {
                            kind: cure_python_ast::ArgKind::Star,
                            ..
                        }) => {
                            self.out.push('*');
                            let v = self.ast.children(a)[0];
                            {
                                let s = self.expr_string(v, Prec::Unary);
                                self.out.push_str(&s);
                            }
                        }
                        Some(cure_python_ast::CallArg {
                            kind: cure_python_ast::ArgKind::KwStar,
                            ..
                        }) => {
                            self.out.push_str("**");
                            let v = self.ast.children(a)[0];
                            {
                                let s = self.expr_string(v, Prec::Unary);
                                self.out.push_str(&s);
                            }
                        }
                        _ => {
                            let s = self.expr_string(a, Prec::Lambda);
                            self.out.push_str(&s);
                        }
                    }
                }
                self.out.push(')');
            }
            NodeData::Attribute { name } => {
                let base = self.ast.children(id)[0];
                let mut bs = String::new();
                if matches!(self.ast.data(base), NodeData::Literal(_)) {
                    bs.push('(');
                    let bs2 = self.expr_string(base, Prec::Lambda);
                    bs.push_str(&bs2);
                    bs.push(')');
                } else {
                    bs = self.expr_string(base, Prec::Primary);
                }
                self.out.push_str(&bs);
                self.out.push('.');
                self.out.push_str(name);
            }
            NodeData::Subscript => {
                let ch = self.ast.children(id).to_vec();
                let base = self.expr_string(ch[0], Prec::Primary);
                self.out.push_str(&base);
                self.out.push('[');
                // 下标位的 Tuple 有两种来源：
                // - slice list（含 Slice 元素）：必须裸逗号分隔，加括号会
                //   让 slice 嵌套进 tuple display（3.15 语法错）；
                // - 显式 tuple 表达式（`a[()]`/`a[(1,)]`）：按 tuple 打印，
                //   否则空/单元素 tuple 会退化成标量。
                if let NodeData::Tuple = self.ast.data(ch[1]) {
                    let items = self.ast.children(ch[1]).to_vec();
                    let has_slice = items
                        .iter()
                        .any(|&it| matches!(self.ast.data(it), NodeData::Slice { .. }));
                    if has_slice {
                        for (i, &it) in items.iter().enumerate() {
                            if i > 0 {
                                self.out.push_str(", ");
                            }
                            let s = self.expr_string(it, Prec::Lambda);
                            self.out.push_str(&s);
                        }
                    } else {
                        let s = self.expr_string(ch[1], Prec::Lambda);
                        self.out.push_str(&s);
                    }
                } else {
                    let s = self.expr_string(ch[1], Prec::Lambda);
                    self.out.push_str(&s);
                }
                self.out.push(']');
            }
            NodeData::Slice { lower, upper, step } => {
                let ch = self.ast.children(id).to_vec();
                let mut i = 0;
                if *lower {
                    {
                        let s = self.expr_string(ch[i], Prec::Lambda);
                        self.out.push_str(&s);
                    }
                    i += 1;
                }
                self.out.push(':');
                if *upper {
                    {
                        let s = self.expr_string(ch[i], Prec::Lambda);
                        self.out.push_str(&s);
                    }
                    i += 1;
                }
                if *step {
                    self.out.push(':');
                    {
                        let s = self.expr_string(ch[i], Prec::Lambda);
                        self.out.push_str(&s);
                    }
                }
            }
            NodeData::Await => {
                let e = self.ast.children(id)[0];
                self.out.push_str("await ");
                {
                    let s = self.expr_string(e, Prec::Await);
                    self.out.push_str(&s);
                }
            }
            NodeData::Yield { from } => {
                let ch = self.ast.children(id).to_vec();
                self.out.push_str("yield");
                if *from {
                    self.out.push_str(" from ");
                } else if !ch.is_empty() {
                    self.out.push(' ');
                }
                if let Some(&v) = ch.first() {
                    {
                        let s = self.expr_string(v, Prec::Lambda);
                        self.out.push_str(&s);
                    }
                }
            }
            NodeData::Starred => {
                let e = self.ast.children(id)[0];
                self.out.push('*');
                {
                    let s = self.expr_string(e, Prec::Unary);
                    self.out.push_str(&s);
                }
            }
            NodeData::DoubleStar => {
                let e = self.ast.children(id)[0];
                self.out.push_str("**");
                {
                    let s = self.expr_string(e, Prec::Unary);
                    self.out.push_str(&s);
                }
            }
            NodeData::Comp { kind } => {
                let ch = self.ast.children(id).to_vec();
                let (open, close) = match kind {
                    CompKind::List => ("[", "]"),
                    CompKind::Set => ("{", "}"),
                    CompKind::Dict => ("{", "}"),
                    CompKind::Gen => ("(", ")"),
                };
                self.out.push_str(open);
                if *kind == CompKind::Dict {
                    {
                        let s = self.expr_string(ch[0], Prec::Lambda);
                        self.out.push_str(&s);
                    }
                    self.out.push_str(": ");
                    {
                        let s = self.expr_string(ch[1], Prec::Lambda);
                        self.out.push_str(&s);
                    }
                    self.print_comps(&ch[2..]);
                } else {
                    {
                        let s = self.expr_string(ch[0], Prec::Lambda);
                        self.out.push_str(&s);
                    }
                    self.print_comps(&ch[1..]);
                }
                self.out.push_str(close);
            }
            NodeData::CompFor {
                target_raw,
                is_async,
                if_count,
            } => {
                let ch = self.ast.children(id).to_vec();
                if *is_async {
                    self.out.push_str(" async for ");
                } else {
                    self.out.push_str(" for ");
                }
                self.out.push_str(target_raw.trim());
                self.out.push_str(" in ");
                {
                    let s = self.expr_string(ch[0], Prec::Or);
                    self.out.push_str(&s);
                }
                for &c in &ch[1..1 + *if_count] {
                    self.out.push_str(" if ");
                    {
                        let s = self.expr_string(c, Prec::Or);
                        self.out.push_str(&s);
                    }
                }
            }
            NodeData::Paren => {
                let inner = self.ast.children(id)[0];
                // NamedExpr 自身打印 `(name := value)`；再包一层会在
                // 二轮解析后逐轮加括号（幂等漂移）。
                if matches!(self.ast.data(inner), NodeData::NamedExpr { .. }) {
                    let s = self.expr_string(inner, Prec::Lambda);
                    self.out.push_str(&s);
                } else {
                    self.out.push('(');
                    let s = self.expr_string(inner, Prec::Lambda);
                    self.out.push_str(&s);
                    self.out.push(')');
                }
            }
            NodeData::NamedExpr { name } => {
                let v = self.ast.children(id)[0];
                self.out.push('(');
                self.out.push_str(name);
                self.out.push_str(" := ");
                {
                    let s = self.expr_string(v, Prec::Lambda);
                    self.out.push_str(&s);
                }
                self.out.push(')');
            }
            NodeData::RawStmt(text) => self.out.push_str(text.trim()),
            _ => self.out.push_str("None"),
        }
    }

    fn print_comps(&mut self, comps: &[PyId]) {
        for &c in comps {
            self.print_expr(c);
        }
    }
}
