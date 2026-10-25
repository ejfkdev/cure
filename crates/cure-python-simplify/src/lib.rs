//! Python 简化：字面量折叠（整数/布尔/`and`/`or`/比较/条件表达式）、
//! 常量条件分支内联、循环恒假折叠、终止语句后的不可达删除、`pass` 清理。
//!
//! 只做 Python 语义明确安全的变换：整数域用 i128 计算并回验 i64；
//! 除法/除零/大整数/浮点表示一律不折；原文区域不触碰。

use std::collections::BTreeMap;

use cure_engine::Config;
use cure_python_ast::{CompKind, LitKind, Literal, NodeData, PyAst, PyBin, PyBoolOp, PyId, PyUn};

#[derive(Clone, Debug, Default)]
pub struct Report {
    pub edits: usize,
    pub iterations: usize,
    pub by_rule: BTreeMap<&'static str, usize>,
}

impl Report {
    fn hit(&mut self, rule: &'static str) {
        self.edits += 1;
        *self.by_rule.entry(rule).or_insert(0) += 1;
    }
}

pub fn default_rules() -> &'static [&'static str] {
    &[
        "py_const_fold",
        "py_const_if",
        "py_unreachable",
        "py_pass_cleanup",
    ]
}

pub fn simplify_module(ast: &mut PyAst, module: PyId, cfg: &Config) -> Report {
    let mut total = Report::default();
    for _ in 0..6 {
        let mut pass = Pass {
            ast,
            cfg,
            report: Report::default(),
            changed: false,
        };
        pass.simplify_block(module);
        total.edits += pass.report.edits;
        for (k, v) in pass.report.by_rule {
            *total.by_rule.entry(k).or_insert(0) += v;
        }
        total.iterations += 1;
        if !pass.changed {
            break;
        }
    }
    total
}

struct Pass<'a> {
    ast: &'a mut PyAst,
    cfg: &'a Config,
    report: Report,
    changed: bool,
}

impl<'a> Pass<'a> {
    fn enabled(&self, rule: &str) -> bool {
        !self.cfg.disabled_rules.contains(rule)
    }

    fn hit(&mut self, rule: &'static str) {
        self.report.hit(rule);
        self.changed = true;
    }

    fn as_int(&self, id: PyId) -> Option<i64> {
        match self.ast.data(id) {
            NodeData::Literal(l) if l.kind == LitKind::Int => l.int,
            _ => None,
        }
    }

    /// `-5` 的 Unary 子树与折叠后的负字面量打印结果相同；只有原字面量
    /// 采用规范十进制写法时才成立（`-0x10` → `-16` 是可见变化）。
    fn minus_is_print_neutral(&self, inner: PyId, a: i64) -> bool {
        if a <= 0 {
            return false;
        }
        match self.ast.data(inner) {
            NodeData::Literal(l) => l.kind == LitKind::Int && l.raw == a.to_string(),
            _ => false,
        }
    }

    fn truthy_lit(&self, id: PyId) -> Option<bool> {
        match self.ast.data(id) {
            NodeData::Literal(l) => match l.kind {
                LitKind::Bool => l.bool_val,
                LitKind::None => Some(false),
                LitKind::Int => l.int.map(|v| v != 0),
                _ => None,
            },
            _ => None,
        }
    }

    fn set_int(&mut self, id: PyId, v: i64, rule: &'static str) {
        *self.ast.data_mut(id) = NodeData::Literal(Literal::int(v));
        self.ast.set_children(id, vec![]);
        self.hit(rule);
    }

    /// 折叠不改变打印结果时（`-5` 的 Unary 与负字面量都打印 `-5`）只
    /// 归一 AST、不记编辑：否则 `--check` 会在已收敛文件上报「可改写」。
    /// 父节点同轮已能看到折叠后的整数，不会因此漏掉后续常量折叠。
    fn set_int_quiet(&mut self, id: PyId, v: i64) {
        *self.ast.data_mut(id) = NodeData::Literal(Literal::int(v));
        self.ast.set_children(id, vec![]);
    }

    fn set_literal_from(&mut self, id: PyId, src: PyId) {
        if let NodeData::Literal(l) = self.ast.data(src) {
            let lit = l.clone();
            *self.ast.data_mut(id) = NodeData::Literal(lit);
            self.ast.set_children(id, vec![]);
            self.hit("py_const_fold");
        }
    }

    fn set_bool(&mut self, id: PyId, v: bool, rule: &'static str) {
        *self.ast.data_mut(id) = NodeData::Literal(Literal::boolean(v));
        self.ast.set_children(id, vec![]);
        self.hit(rule);
    }

    fn simplify_block(&mut self, block: PyId) {
        let old = self.ast.children(block).to_vec();
        let mut new = Vec::with_capacity(old.len());
        for stmt in old {
            let mut v = self.simplify_stmt(stmt);
            new.append(&mut v);
        }
        // 终止语句后的不可达删除
        if self.enabled("py_unreachable") {
            if let Some(pos) = new.iter().position(|&s| is_terminal(self.ast, s)) {
                if pos + 1 < new.len()
                    && !new[pos + 1..]
                        .iter()
                        .any(|&s| has_scope_or_yield(self.ast, s))
                {
                    new.truncate(pos + 1);
                    self.hit("py_unreachable");
                }
            }
        }
        // 纯 pass 清理：块内还有其它语句时 pass 可删
        if self.enabled("py_pass_cleanup") && new.len() > 1 {
            let before = new.len();
            new.retain(|&s| !matches!(self.ast.data(s), NodeData::Pass));
            if new.len() != before {
                self.hit("py_pass_cleanup");
            }
        }
        if new.is_empty() {
            let p = self.ast.simple(NodeData::Pass);
            new.push(p);
        }
        self.ast.set_children(block, new);
    }

    /// 返回 0..n 条语句（常量 if 内联）。
    fn simplify_stmt(&mut self, id: PyId) -> Vec<PyId> {
        match self.ast.data(id).clone() {
            NodeData::If => {
                let ch = self.ast.children(id).to_vec();
                let test = self.simplify_expr(ch[0]);
                self.ast.set_child(id, 0, test);
                let body = ch[1];
                self.simplify_block(body);
                let orelse = ch.get(2).copied();
                if let Some(o) = orelse {
                    self.simplify_block(o);
                }
                if self.enabled("py_const_if") {
                    if let Some(t) = self.truthy_lit(test) {
                        let dropped = if t { orelse } else { Some(body) };
                        if dropped.is_some_and(|d| has_scope_or_yield(self.ast, d)) {
                            return vec![id];
                        }
                        self.hit("py_const_if");
                        if t {
                            return self.ast.children(body).to_vec();
                        }
                        return orelse
                            .map(|o| self.ast.children(o).to_vec())
                            .unwrap_or_default();
                    }
                }
                vec![id]
            }
            NodeData::While => {
                let ch = self.ast.children(id).to_vec();
                let test = self.simplify_expr(ch[0]);
                self.ast.set_child(id, 0, test);
                self.simplify_block(ch[1]);
                let orelse = ch.get(2).copied();
                if let Some(o) = orelse {
                    self.simplify_block(o);
                }
                if self.enabled("py_const_if")
                    && self.truthy_lit(test) == Some(false)
                    && !has_scope_or_yield(self.ast, ch[1])
                {
                    self.hit("py_const_if");
                    return orelse
                        .map(|o| self.ast.children(o).to_vec())
                        .unwrap_or_default();
                }
                vec![id]
            }
            NodeData::For {
                target_raw,
                is_async,
            } => {
                let ch = self.ast.children(id).to_vec();
                let iter = self.simplify_expr(ch[0]);
                self.ast.set_child(id, 0, iter);
                self.simplify_block(ch[1]);
                if let Some(&o) = ch.get(2) {
                    self.simplify_block(o);
                }
                let _ = (target_raw, is_async);
                vec![id]
            }
            NodeData::FunctionDef { .. } | NodeData::ClassDef { .. } => {
                let body = self.ast.children(id)[0];
                self.simplify_block(body);
                vec![id]
            }
            NodeData::With { .. } => {
                let body = self.ast.children(id)[0];
                self.simplify_block(body);
                vec![id]
            }
            NodeData::Try {
                handler_count,
                has_else,
                has_finally,
                ..
            } => {
                let ch = self.ast.children(id).to_vec();
                self.simplify_block(ch[0]);
                for &h in &ch[1..1 + handler_count] {
                    let hb = self.ast.children(h)[0];
                    self.simplify_block(hb);
                }
                let mut idx = 1 + handler_count;
                if has_else {
                    self.simplify_block(ch[idx]);
                    idx += 1;
                }
                if has_finally {
                    self.simplify_block(ch[idx]);
                }
                vec![id]
            }
            NodeData::Match => {
                let ch = self.ast.children(id).to_vec();
                let subject = self.simplify_expr(ch[0]);
                self.ast.set_child(id, 0, subject);
                for &case in &ch[1..] {
                    let cch = self.ast.children(case).to_vec();
                    let body = if matches!(
                        self.ast.data(case),
                        NodeData::Case {
                            has_guard: true,
                            ..
                        }
                    ) {
                        let g = self.simplify_expr(cch[0]);
                        self.ast.set_child(case, 0, g);
                        cch[1]
                    } else {
                        cch[0]
                    };
                    self.simplify_block(body);
                }
                vec![id]
            }
            NodeData::ExprStmt => {
                let e = self.ast.children(id)[0];
                let e2 = self.simplify_expr(e);
                self.ast.set_child(id, 0, e2);
                vec![id]
            }
            NodeData::Assign { targets } => {
                let ch = self.ast.children(id).to_vec();
                for (i, &t) in ch[..targets].iter().enumerate() {
                    let t2 = self.simplify_target_expr(t);
                    self.ast.set_child(id, i, t2);
                }
                let v = self.simplify_expr(ch[targets]);
                self.ast.set_child(id, targets, v);
                vec![id]
            }
            NodeData::AugAssign { .. } => {
                let ch = self.ast.children(id).to_vec();
                let t = self.simplify_target_expr(ch[0]);
                self.ast.set_child(id, 0, t);
                let v = self.simplify_expr(ch[1]);
                self.ast.set_child(id, 1, v);
                vec![id]
            }
            NodeData::AnnAssign { .. } => {
                let ch = self.ast.children(id).to_vec();
                let t = self.simplify_target_expr(ch[0]);
                self.ast.set_child(id, 0, t);
                if ch.len() > 1 {
                    let v = self.simplify_expr(ch[1]);
                    self.ast.set_child(id, 1, v);
                }
                vec![id]
            }
            NodeData::Return => {
                if let Some(&v) = self.ast.children(id).first() {
                    let v2 = self.simplify_expr(v);
                    self.ast.set_child(id, 0, v2);
                }
                vec![id]
            }
            NodeData::Raise { from } => {
                let ch = self.ast.children(id).to_vec();
                if let Some(&e) = ch.first() {
                    let e2 = self.simplify_expr(e);
                    self.ast.set_child(id, 0, e2);
                }
                if from {
                    if let Some(&c) = ch.get(1) {
                        let c2 = self.simplify_expr(c);
                        self.ast.set_child(id, 1, c2);
                    }
                }
                vec![id]
            }
            NodeData::Assert => {
                let ch = self.ast.children(id).to_vec();
                let t = self.simplify_expr(ch[0]);
                self.ast.set_child(id, 0, t);
                if ch.len() > 1 {
                    let m = self.simplify_expr(ch[1]);
                    self.ast.set_child(id, 1, m);
                }
                vec![id]
            }
            NodeData::Delete => {
                let ch = self.ast.children(id).to_vec();
                for (i, &t) in ch.iter().enumerate() {
                    let t2 = self.simplify_target_expr(t);
                    self.ast.set_child(id, i, t2);
                }
                vec![id]
            }
            _ => vec![id],
        }
    }

    fn simplify_target_expr(&mut self, id: PyId) -> PyId {
        match self.ast.data(id).clone() {
            NodeData::Name(_) | NodeData::Raw(_) => id,
            NodeData::Attribute { .. } => {
                let b = self.ast.children(id)[0];
                let b2 = self.simplify_expr(b);
                self.ast.set_child(id, 0, b2);
                id
            }
            NodeData::Subscript => {
                let ch = self.ast.children(id).to_vec();
                let b = self.simplify_target_expr(ch[0]);
                self.ast.set_child(id, 0, b);
                let i = self.simplify_expr(ch[1]);
                self.ast.set_child(id, 1, i);
                id
            }
            NodeData::Tuple | NodeData::List => {
                let ch = self.ast.children(id).to_vec();
                for (i, &c) in ch.iter().enumerate() {
                    let c2 = self.simplify_target_expr(c);
                    self.ast.set_child(id, i, c2);
                }
                id
            }
            NodeData::Starred => {
                let c = self.ast.children(id)[0];
                let c2 = self.simplify_target_expr(c);
                self.ast.set_child(id, 0, c2);
                id
            }
            _ => id,
        }
    }

    fn simplify_expr(&mut self, id: PyId) -> PyId {
        match self.ast.data(id).clone() {
            NodeData::Name(_) | NodeData::Literal(_) | NodeData::Raw(_) => id,
            NodeData::Binary { op } => {
                let ch = self.ast.children(id).to_vec();
                let l = self.simplify_expr(ch[0]);
                let r = self.simplify_expr(ch[1]);
                self.ast.set_child(id, 0, l);
                self.ast.set_child(id, 1, r);
                if self.enabled("py_const_fold") {
                    if let (Some(a), Some(b)) = (self.as_int(l), self.as_int(r)) {
                        if let Some(v) = fold_int(op, a, b) {
                            self.set_int(id, v, "py_const_fold");
                        }
                    }
                }
                id
            }
            NodeData::Unary { op } => {
                let v = self.ast.children(id)[0];
                let v2 = self.simplify_expr(v);
                self.ast.set_child(id, 0, v2);
                if self.enabled("py_const_fold") {
                    match op {
                        PyUn::Not => {
                            if let Some(t) = self.truthy_lit(v2) {
                                self.set_bool(id, !t, "py_const_fold");
                            }
                        }
                        PyUn::Minus | PyUn::Plus | PyUn::Invert => {
                            if let Some(a) = self.as_int(v2) {
                                let v = match op {
                                    PyUn::Minus => a.checked_neg(),
                                    PyUn::Plus => Some(a),
                                    PyUn::Invert => Some(!a),
                                    _ => None,
                                };
                                if let Some(v) = v {
                                    if op == PyUn::Minus && self.minus_is_print_neutral(v2, a) {
                                        self.set_int_quiet(id, v);
                                    } else {
                                        self.set_int(id, v, "py_const_fold");
                                    }
                                }
                            }
                        }
                    }
                }
                id
            }
            NodeData::BoolOp { op } => {
                let ch = self.ast.children(id).to_vec();
                let mut vals = Vec::with_capacity(ch.len());
                for &c in &ch {
                    vals.push(self.simplify_expr(c));
                }
                self.ast.set_children(id, vals.clone());
                if self.enabled("py_const_fold") {
                    // 全字面量：and/or 返回某个操作数（Python 语义）
                    let all_lit: Option<Vec<bool>> =
                        vals.iter().map(|&v| self.truthy_lit(v)).collect();
                    if let Some(ts) = all_lit {
                        let pick = match op {
                            PyBoolOp::And => ts.iter().position(|&t| !t).unwrap_or(ts.len() - 1),
                            PyBoolOp::Or => ts.iter().position(|&t| t).unwrap_or(ts.len() - 1),
                        };
                        let chosen = vals[pick];
                        let lit = match self.ast.data(chosen) {
                            NodeData::Literal(l) => l.clone(),
                            _ => unreachable!(),
                        };
                        *self.ast.data_mut(id) = NodeData::Literal(lit);
                        self.ast.set_children(id, vec![]);
                        self.hit("py_const_fold");
                        return id;
                    }
                    match op {
                        PyBoolOp::And => {
                            // 首个恒假 → 整体就是它（后续不执行）；
                            // 后续含 yield 时不能删（生成器语法标记）
                            if self.truthy_lit(vals[0]) == Some(false)
                                && vals[1..].iter().any(|&v| has_scope_or_yield(self.ast, v))
                            {
                                return id;
                            }
                            if self.truthy_lit(vals[0]) == Some(false) {
                                self.set_literal_from(id, vals[0]);
                                return id;
                            }
                            // 非末位的恒真项可丢（结果由后续操作数给出）
                            let out: Vec<PyId> = vals
                                .iter()
                                .enumerate()
                                .filter(|(i, &v)| {
                                    !(self.truthy_lit(v) == Some(true) && *i + 1 < vals.len())
                                })
                                .map(|(_, &v)| v)
                                .collect();
                            if out.len() != vals.len() {
                                self.hit("py_const_fold");
                                if out.len() == 1 {
                                    return out[0];
                                }
                                self.ast.set_children(id, out);
                            }
                        }
                        PyBoolOp::Or => {
                            if self.truthy_lit(vals[0]) == Some(true)
                                && vals[1..].iter().any(|&v| has_scope_or_yield(self.ast, v))
                            {
                                return id;
                            }
                            if self.truthy_lit(vals[0]) == Some(true) {
                                self.set_literal_from(id, vals[0]);
                                return id;
                            }
                            let out: Vec<PyId> = vals
                                .iter()
                                .enumerate()
                                .filter(|(i, &v)| {
                                    !(self.truthy_lit(v) == Some(false) && *i + 1 < vals.len())
                                })
                                .map(|(_, &v)| v)
                                .collect();
                            if out.len() != vals.len() {
                                self.hit("py_const_fold");
                                if out.len() == 1 {
                                    return out[0];
                                }
                                self.ast.set_children(id, out);
                            }
                        }
                    }
                }
                id
            }
            NodeData::Compare { ops } => {
                let ch = self.ast.children(id).to_vec();
                for (i, &c) in ch.iter().enumerate() {
                    let c2 = self.simplify_expr(c);
                    self.ast.set_child(id, i, c2);
                }
                if self.enabled("py_const_fold") {
                    let vals: Vec<PyId> = self.ast.children(id).to_vec();
                    let mut all = true;
                    let mut result = true;
                    for (i, op) in ops.iter().enumerate() {
                        let (Some(a), Some(b)) = (self.as_int(vals[i]), self.as_int(vals[i + 1]))
                        else {
                            all = false;
                            break;
                        };
                        let Some(sc) = op.scalar() else {
                            all = false;
                            break;
                        };
                        if !cmp_int(sc, a, b) {
                            result = false;
                            break;
                        }
                    }
                    if all {
                        self.set_bool(id, result, "py_const_fold");
                    }
                }
                id
            }
            NodeData::IfExp => {
                let ch = self.ast.children(id).to_vec();
                let body = self.simplify_expr(ch[0]);
                let test = self.simplify_expr(ch[1]);
                let orelse = self.simplify_expr(ch[2]);
                self.ast.set_child(id, 0, body);
                self.ast.set_child(id, 1, test);
                self.ast.set_child(id, 2, orelse);
                if self.enabled("py_const_if") {
                    if let Some(t) = self.truthy_lit(test) {
                        let dropped = if t { orelse } else { body };
                        if !has_scope_or_yield(self.ast, dropped) {
                            self.hit("py_const_if");
                            return if t { body } else { orelse };
                        }
                    }
                }
                id
            }
            NodeData::Tuple | NodeData::List | NodeData::Set => {
                let ch = self.ast.children(id).to_vec();
                for (i, &c) in ch.iter().enumerate() {
                    let c2 = self.simplify_expr(c);
                    self.ast.set_child(id, i, c2);
                }
                id
            }
            NodeData::Dict => {
                let ch = self.ast.children(id).to_vec();
                for (i, &c) in ch.iter().enumerate() {
                    let c2 = self.simplify_expr(c);
                    self.ast.set_child(id, i, c2);
                }
                id
            }
            NodeData::Call { .. } => {
                let ch = self.ast.children(id).to_vec();
                for (i, &c) in ch.iter().enumerate() {
                    let c2 = self.simplify_expr(c);
                    self.ast.set_child(id, i, c2);
                }
                id
            }
            NodeData::Attribute { .. } => {
                let b = self.ast.children(id)[0];
                let b2 = self.simplify_expr(b);
                self.ast.set_child(id, 0, b2);
                id
            }
            NodeData::Subscript => {
                let ch = self.ast.children(id).to_vec();
                let b = self.simplify_expr(ch[0]);
                let i = self.simplify_expr(ch[1]);
                self.ast.set_child(id, 0, b);
                self.ast.set_child(id, 1, i);
                id
            }
            NodeData::Slice { .. } => {
                let ch = self.ast.children(id).to_vec();
                for (i, &c) in ch.iter().enumerate() {
                    let c2 = self.simplify_expr(c);
                    self.ast.set_child(id, i, c2);
                }
                id
            }
            NodeData::Await | NodeData::Yield { .. } | NodeData::Starred | NodeData::DoubleStar => {
                let ch = self.ast.children(id).to_vec();
                for (i, &c) in ch.iter().enumerate() {
                    let c2 = self.simplify_expr(c);
                    self.ast.set_child(id, i, c2);
                }
                id
            }
            NodeData::Lambda { .. } => {
                let b = self.ast.children(id)[0];
                let b2 = self.simplify_expr(b);
                self.ast.set_child(id, 0, b2);
                id
            }
            NodeData::Paren => {
                let v = self.ast.children(id)[0];
                let v2 = self.simplify_expr(v);
                self.ast.set_child(id, 0, v2);
                id
            }
            NodeData::NamedExpr { .. } => {
                let v = self.ast.children(id)[0];
                let v2 = self.simplify_expr(v);
                self.ast.set_child(id, 0, v2);
                id
            }
            NodeData::Comp { kind } => {
                let ch = self.ast.children(id).to_vec();
                let n_elt = if kind == CompKind::Dict { 2 } else { 1 };
                for (i, &c) in ch.iter().enumerate() {
                    if i < n_elt {
                        let c2 = self.simplify_expr(c);
                        self.ast.set_child(id, i, c2);
                    } else {
                        self.simplify_comp_for(c);
                    }
                }
                id
            }
            _ => id,
        }
    }

    fn simplify_comp_for(&mut self, id: PyId) {
        let ch = self.ast.children(id).to_vec();
        for (i, &c) in ch.iter().enumerate() {
            let c2 = self.simplify_expr(c);
            self.ast.set_child(id, i, c2);
        }
    }
}

/// 语法级作用域/生成器标记：`yield` 出现使函数成为生成器；
/// `global`/`nonlocal` 影响整个函数作用域——即使不可达/常量分支也不能删。
fn has_scope_or_yield(ast: &PyAst, id: PyId) -> bool {
    match ast.data(id) {
        NodeData::Yield { .. } | NodeData::Global(_) | NodeData::Nonlocal(_) => true,
        _ => ast.children(id).iter().any(|&c| has_scope_or_yield(ast, c)),
    }
}

fn is_terminal(ast: &PyAst, id: PyId) -> bool {
    matches!(
        ast.data(id),
        NodeData::Return | NodeData::Raise { .. } | NodeData::Break | NodeData::Continue
    )
}

fn cmp_int(op: cure_python_ast::fold::ScalarCmp, a: i64, b: i64) -> bool {
    use cure_python_ast::fold::ScalarCmp::*;
    match op {
        Eq => a == b,
        Ne => a != b,
        Lt => a < b,
        Le => a <= b,
        Gt => a > b,
        Ge => a >= b,
    }
}

fn fold_int(op: PyBin, a: i64, b: i64) -> Option<i64> {
    let (x, y) = (a as i128, b as i128);
    let v: i128 = match op {
        PyBin::Add => x + y,
        PyBin::Sub => x - y,
        PyBin::Mul => x * y,
        PyBin::FloorDiv => {
            if y == 0 {
                return None;
            }
            floor_div(x, y)
        }
        PyBin::Mod => {
            if y == 0 {
                return None;
            }
            py_mod(x, y)
        }
        PyBin::LShift => {
            if y < 0 || y > 63 {
                return None;
            }
            x.checked_shl(y as u32)?
        }
        PyBin::RShift => {
            if y < 0 || y > 63 {
                return None;
            }
            x >> (y as u32)
        }
        PyBin::BitAnd => x & y,
        PyBin::BitOr => x | y,
        PyBin::BitXor => x ^ y,
        PyBin::Pow => {
            if !(0..=63).contains(&y) {
                return None;
            }
            let mut acc: i128 = 1;
            for _ in 0..y {
                acc = acc.checked_mul(x)?;
            }
            acc
        }
        // 浮点结果/矩阵乘：不折（float repr 与异常语义）
        PyBin::Div | PyBin::MatMul => return None,
    };
    i64::try_from(v).ok()
}

fn floor_div(a: i128, b: i128) -> i128 {
    let q = a / b;
    let r = a % b;
    if r != 0 && ((r < 0) != (b < 0)) {
        q - 1
    } else {
        q
    }
}

fn py_mod(a: i128, b: i128) -> i128 {
    let r = a % b;
    if r != 0 && ((r < 0) != (b < 0)) {
        r + b
    } else {
        r
    }
}
