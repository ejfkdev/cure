//! Python 简化：字面量折叠（整数/布尔/`and`/`or`/比较/条件表达式）、
//! 常量条件分支内联、循环恒假折叠、终止语句后的不可达删除、`pass` 清理。
//!
//! 只做 Python 语义明确安全的变换：整数域用 i128 计算并回验 i64；
//! 除法/除零/大整数/浮点表示一律不折；原文区域不触碰。

mod literal;

use std::collections::{BTreeMap, BTreeSet};

use cure_engine::Config;
use cure_python_ast::{
    ArgKind, CompKind, LitKind, Literal, NodeData, PyAst, PyBin, PyBoolOp, PyId, PyUn,
};

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
        "py_call_fold",
        "py_subscript_fold",
        "py_const_if",
        "py_if_ternary",
        "py_nested_if",
        "py_else_after_terminal",
        "py_dead_store",
        "py_dead_expr",
        "py_return_none",
        "py_unreachable",
        "py_pass_cleanup",
    ]
}

pub fn simplify_module(ast: &mut PyAst, module: PyId, cfg: &Config) -> Report {
    let mut total = Report::default();
    let mut shadowed = BTreeSet::new();
    collect_shadowed(ast, module, &mut shadowed);
    let unicode_literals = uses_unicode_literals(ast, module);
    for _ in 0..32 {
        let mut pass = Pass {
            ast,
            cfg,
            report: Report::default(),
            changed: false,
            docstring_guard: false,
            shadowed: shadowed.clone(),
            unicode_literals,
        };
        pass.simplify_block(module, true);
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
    /// 当前正在简化作用域首条可能形成 docstring 的语句。
    docstring_guard: bool,
    /// 模块内被绑定过的内建名（保守：任一作用域绑定即不折叠该内建的调用）。
    shadowed: BTreeSet<String>,
    /// `from __future__ import unicode_literals`：Python 2 下 str()/chr()
    /// 仍返回 bytes，不能把结果折成（会变成 unicode 的）普通字面量。
    unicode_literals: bool,
}

impl<'a> Pass<'a> {
    fn enabled(&self, rule: &str) -> bool {
        !self.cfg.disabled_rules.contains(rule)
    }

    fn hit(&mut self, rule: &'static str) {
        self.report.hit(rule);
        self.changed = true;
    }

    /// Python 2 的 `1L` 是 long，与 int 的 `type()`/repr 不同；涉及 long 的
    /// 运算不能折成普通整数（值相同但类型变了）。
    fn is_py2_long(&self, id: PyId) -> bool {
        match self.ast.data(id) {
            NodeData::Literal(l) if l.kind == LitKind::Int => {
                let t = l.raw.trim_end();
                t.len() > 1 && matches!(t.as_bytes().last(), Some(b'L' | b'l'))
            }
            _ => false,
        }
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

    fn as_plain_str(&self, id: PyId) -> Option<&str> {
        match self.ast.data(id) {
            NodeData::Literal(l) => literal::plain_str(l),
            _ => None,
        }
    }

    /// 写入字面量结果；作用域首条语句里不允许把表达式变成裸字符串
    /// （否则会创建原本不存在的 docstring）。
    fn set_literal_checked(
        &mut self,
        id: PyId,
        lit: Literal,
        rule: &'static str,
    ) -> bool {
        if self.docstring_guard && matches!(lit.kind, LitKind::Str | LitKind::Bytes) {
            return false;
        }
        *self.ast.data_mut(id) = NodeData::Literal(lit);
        self.ast.set_children(id, vec![]);
        self.hit(rule);
        true
    }


    fn simplify_block(&mut self, block: PyId, scope: bool) {
        let old = self.ast.children(block).to_vec();
        let orig_first_doc = old
            .first()
            .is_some_and(|&s| is_doc_expr(self.ast, s));
        let mut new = Vec::with_capacity(old.len());
        for (idx, stmt) in old.into_iter().enumerate() {
            let saved = self.docstring_guard;
            self.docstring_guard = scope
                && idx == 0
                && matches!(
                    self.ast.data(stmt),
                    NodeData::ExprStmt | NodeData::If | NodeData::While
                );
            let mut v = self.simplify_stmt(stmt);
            self.docstring_guard = saved;
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
        // 相邻字面量死存：`x = 1; x = [2]` → 第一条可删（两侧都无副作用）。
        if self.enabled("py_dead_store") {
            let mut i = 0;
            while i + 1 < new.len() {
                if let Some(name) = dead_store_target(self.ast, new[i]) {
                    if overwrites_with_pure(self.ast, new[i + 1], &name) {
                        new.remove(i);
                        self.hit("py_dead_store");
                        continue;
                    }
                }
                i += 1;
            }
        }
        // 纯 pass 清理：块内还有其它语句时 pass 可删。作用域首条的 pass
        // 不能删掉后让下一条裸字符串变成 docstring。
        if self.enabled("py_pass_cleanup") && new.len() > 1 {
            let keep_first = scope
                && matches!(self.ast.data(new[0]), NodeData::Pass)
                && new.get(1).is_some_and(|&s| is_doc_expr(self.ast, s));
            let before = new.len();
            let mut kept = Vec::with_capacity(new.len());
            for (i, &s) in new.iter().enumerate() {
                if i == 0 && keep_first {
                    kept.push(s);
                } else if !matches!(self.ast.data(s), NodeData::Pass) {
                    kept.push(s);
                }
            }
            new = kept;
            if new.len() != before {
                self.hit("py_pass_cleanup");
            }
        }
        if new.is_empty() {
            let p = self.ast.simple(NodeData::Pass);
            new.push(p);
        }
        // 作用域首条原本不是裸字符串：折叠/内联不能把它变成 docstring。
        if scope && !orig_first_doc && new.first().is_some_and(|&s| is_doc_expr(self.ast, s)) {
            let p = self.ast.simple(NodeData::Pass);
            new.insert(0, p);
            self.changed = true;
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
                self.simplify_block(body, false);
                let orelse = ch.get(2).copied();
                if let Some(o) = orelse {
                    self.simplify_block(o, false);
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
                if self.enabled("py_if_ternary") {
                    if let Some(merged) = self.merge_if_ternary(test, body, orelse) {
                        self.hit("py_if_ternary");
                        return vec![merged];
                    }
                }
                if self.enabled("py_else_after_terminal") {
                    if let Some(o) = orelse {
                        let och = self.ast.children(o).to_vec();
                        let is_elif = och.len() == 1
                            && matches!(self.ast.data(och[0]), NodeData::If);
                        let body_terminal = self
                            .ast
                            .children(body)
                            .last()
                            .is_some_and(|&s| is_terminal(self.ast, s));
                        if !is_elif && body_terminal {
                            self.ast.set_children(id, vec![test, body]);
                            self.hit("py_else_after_terminal");
                            let mut out = vec![id];
                            out.extend(och);
                            return out;
                        }
                    }
                }
                if self.enabled("py_nested_if") && orelse.is_none() {
                    let bch = self.ast.children(body).to_vec();
                    if bch.len() == 1 && matches!(self.ast.data(bch[0]), NodeData::If) {
                        let inner = bch[0];
                        let ich = self.ast.children(inner).to_vec();
                        if ich.len() == 2 {
                            let and =
                                self.ast.bool_op(PyBoolOp::And, vec![test, ich[0]]);
                            let new_if = self.ast.if_(and, ich[1], None);
                            self.hit("py_nested_if");
                            return vec![new_if];
                        }
                    }
                }
                vec![id]
            }
            NodeData::While => {
                let ch = self.ast.children(id).to_vec();
                let test = self.simplify_expr(ch[0]);
                self.ast.set_child(id, 0, test);
                self.simplify_block(ch[1], false);
                let orelse = ch.get(2).copied();
                if let Some(o) = orelse {
                    self.simplify_block(o, false);
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
                self.simplify_block(ch[1], false);
                if let Some(&o) = ch.get(2) {
                    self.simplify_block(o, false);
                }
                let _ = (target_raw, is_async);
                vec![id]
            }
            NodeData::FunctionDef { .. } | NodeData::ClassDef { .. } => {
                let body = self.ast.children(id)[0];
                self.simplify_block(body, true);
                vec![id]
            }
            NodeData::With { .. } => {
                let body = self.ast.children(id)[0];
                self.simplify_block(body, false);
                vec![id]
            }
            NodeData::Try {
                handler_count,
                has_else,
                has_finally,
                ..
            } => {
                let ch = self.ast.children(id).to_vec();
                self.simplify_block(ch[0], false);
                for &h in &ch[1..1 + handler_count] {
                    let hb = self.ast.children(h)[0];
                    self.simplify_block(hb, false);
                }
                let mut idx = 1 + handler_count;
                if has_else {
                    self.simplify_block(ch[idx], false);
                    idx += 1;
                }
                if has_finally {
                    self.simplify_block(ch[idx], false);
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
                    self.simplify_block(body, false);
                }
                vec![id]
            }
            NodeData::ExprStmt => {
                let e = self.ast.children(id)[0];
                let e2 = self.simplify_expr(e);
                self.ast.set_child(id, 0, e2);
                if self.enabled("py_dead_expr")
                    && !self.docstring_guard
                    && matches!(
                        self.ast.data(e2),
                        NodeData::Literal(Literal {
                            kind: LitKind::Int
                                | LitKind::Float
                                | LitKind::Complex
                                | LitKind::Bool
                                | LitKind::None
                                | LitKind::Ellipsis,
                            ..
                        })
                    )
                {
                    self.hit("py_dead_expr");
                    return vec![];
                }
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
                    if self.enabled("py_return_none")
                        && matches!(
                            self.ast.data(v2),
                            NodeData::Literal(Literal {
                                kind: LitKind::None,
                                ..
                            })
                        )
                    {
                        self.ast.set_children(id, vec![]);
                        self.hit("py_return_none");
                    }
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
            NodeData::Print { .. } => {
                let ch = self.ast.children(id).to_vec();
                for (i, &e) in ch.iter().enumerate() {
                    let e2 = self.simplify_expr(e);
                    self.ast.set_child(id, i, e2);
                }
                vec![id]
            }
            _ => vec![id],
        }
    }

    /// `if c: x = a else: x = b` → `x = a if c else b`；
    /// `if c: return a else: return b` → `return a if c else b`。
    fn merge_if_ternary(
        &mut self,
        test: PyId,
        body: PyId,
        orelse: Option<PyId>,
    ) -> Option<PyId> {
        let o = orelse?;
        let bch = self.ast.children(body).to_vec();
        let och = self.ast.children(o).to_vec();
        if bch.len() != 1 || och.len() != 1 {
            return None;
        }
        let a = bch[0];
        let b = och[0];
        match (self.ast.data(a), self.ast.data(b)) {
            (NodeData::Assign { targets: t1 }, NodeData::Assign { targets: t2 })
                if *t1 == 1 && *t2 == 1 =>
            {
                let ac = self.ast.children(a).to_vec();
                let bc = self.ast.children(b).to_vec();
                let n1 = match self.ast.data(ac[0]) {
                    NodeData::Name(n) => n.clone(),
                    _ => return None,
                };
                let n2 = match self.ast.data(bc[0]) {
                    NodeData::Name(n) => n.clone(),
                    _ => return None,
                };
                if n1 != n2 {
                    return None;
                }
                let ie = self.ast.if_exp(ac[1], test, bc[1]);
                Some(self.ast.assign(vec![ac[0]], ie))
            }
            (NodeData::Return, NodeData::Return) => {
                let ac = self.ast.children(a).to_vec();
                let bc = self.ast.children(b).to_vec();
                if ac.len() != 1 || bc.len() != 1 {
                    return None;
                }
                let ie = self.ast.if_exp(ac[0], test, bc[0]);
                Some(self.ast.return_(Some(ie)))
            }
            _ => None,
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
                if self.enabled("py_const_fold")
                    && !self.is_py2_long(l)
                    && !self.is_py2_long(r)
                {
                    if let (Some(a), Some(b)) = (self.as_int(l), self.as_int(r)) {
                        if let Some(v) = fold_int(op, a, b) {
                            self.set_int(id, v, "py_const_fold");
                        }
                    }
                }
                if self.enabled("py_const_fold") && !self.docstring_guard {
                    match op {
                        PyBin::Add => {
                            let pair = (
                                self.as_plain_str(l).map(str::to_string),
                                self.as_plain_str(r).map(str::to_string),
                            );
                            if let (Some(a), Some(b)) = pair {
                                if let Some(lit) = literal::make_str(&format!("{a}{b}")) {
                                    self.set_literal_checked(id, lit, "py_const_fold");
                                }
                            }
                        }
                        PyBin::Mul => {
                            let sn = if let Some(s) = self.as_plain_str(l) {
                                Some((s.to_string(), self.as_int(r)))
                            } else if let Some(s) = self.as_plain_str(r) {
                                Some((s.to_string(), self.as_int(l)))
                            } else {
                                None
                            };
                            if let Some((s, Some(n))) = sn {
                                if let Some(out) = literal::repeat(&s, n) {
                                    if let Some(lit) = literal::make_str(&out) {
                                        self.set_literal_checked(id, lit, "py_const_fold");
                                    }
                                }
                            }
                        }
                        _ => {}
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
                            if !self.is_py2_long(v2) {
                                if let Some(a) = self.as_int(v2) {
                                    let v = match op {
                                        PyUn::Minus => a.checked_neg(),
                                        PyUn::Plus => Some(a),
                                        PyUn::Invert => Some(!a),
                                        _ => None,
                                    };
                                    if let Some(v) = v {
                                        if op == PyUn::Minus
                                            && self.minus_is_print_neutral(v2, a)
                                        {
                                            self.set_int_quiet(id, v);
                                        } else {
                                            self.set_int(id, v, "py_const_fold");
                                        }
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
                        let Some(sc) = op.scalar() else {
                            all = false;
                            break;
                        };
                        if let (Some(a), Some(b)) = (self.as_int(vals[i]), self.as_int(vals[i + 1]))
                        {
                            if !cmp_int(sc, a, b) {
                                result = false;
                                break;
                            }
                        } else if let (Some(a), Some(b)) =
                            (self.as_plain_str(vals[i]), self.as_plain_str(vals[i + 1]))
                        {
                            if !cmp_str(sc, a, b) {
                                result = false;
                                break;
                            }
                        } else {
                            all = false;
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
                        let picked = if t { body } else { orelse };
                        if self.docstring_guard && is_str_lit(self.ast, picked) {
                            return id;
                        }
                        if !has_scope_or_yield(self.ast, dropped) {
                            self.hit("py_const_if");
                            return picked;
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
                if self.enabled("py_call_fold") {
                    let lit = fold_call(
                        self.ast,
                        id,
                        &self.shadowed,
                        self.unicode_literals,
                    );
                    if let Some(lit) = lit {
                        self.set_literal_checked(id, lit, "py_call_fold");
                    }
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
                if self.enabled("py_subscript_fold") {
                    match fold_subscript(self.ast, id) {
                        Some(SubFold::Lit(lit)) => {
                            self.set_literal_checked(id, lit, "py_subscript_fold");
                        }
                        Some(SubFold::Node(n)) => {
                            if !(self.docstring_guard && is_str_lit(self.ast, n)) {
                                self.hit("py_subscript_fold");
                                return n;
                            }
                        }
                        Some(SubFold::Seq(items, tuple)) => {
                            let data = if tuple {
                                NodeData::Tuple
                            } else {
                                NodeData::List
                            };
                            let n = self.ast.push(data, items);
                            self.hit("py_subscript_fold");
                            return n;
                        }
                        None => {}
                    }
                }
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
            NodeData::Paren | NodeData::Repr => {
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

fn is_doc_expr(ast: &PyAst, id: PyId) -> bool {
    match ast.data(id) {
        NodeData::ExprStmt => {
            let e = ast.children(id)[0];
            matches!(
                ast.data(e),
                NodeData::Literal(Literal {
                    kind: LitKind::Str | LitKind::Bytes,
                    ..
                })
            )
        }
        _ => false,
    }
}

fn is_terminal(ast: &PyAst, id: PyId) -> bool {
    matches!(
        ast.data(id),
        NodeData::Return | NodeData::Raise { .. } | NodeData::Break | NodeData::Continue
    )
}

fn dead_store_target(ast: &PyAst, id: PyId) -> Option<String> {
    match ast.data(id) {
        NodeData::Assign { targets: 1 } => {
            let ch = ast.children(id);
            let name = match ast.data(ch[0]) {
                NodeData::Name(n) => n.clone(),
                _ => return None,
            };
            is_pure_value(ast, ch[1]).then_some(name)
        }
        _ => None,
    }
}

fn overwrites_with_pure(ast: &PyAst, id: PyId, name: &str) -> bool {
    match ast.data(id) {
        NodeData::Assign { targets: 1 } => {
            let ch = ast.children(id);
            matches!(ast.data(ch[0]), NodeData::Name(n) if n == name)
                && is_pure_value(ast, ch[1])
        }
        _ => false,
    }
}

fn uses_unicode_literals(ast: &PyAst, id: PyId) -> bool {
    if let NodeData::FromImport(raw) = ast.data(id) {
        if raw.contains("unicode_literals") {
            return true;
        }
    }
    ast.children(id).iter().any(|&c| uses_unicode_literals(ast, c))
}

const FOLD_BUILTINS: &[&str] = &[
    "len", "ord", "chr", "str", "int", "abs", "bool", "min", "max",
];

fn is_fold_builtin(name: &str) -> bool {
    FOLD_BUILTINS.contains(&name)
}

fn collect_shadowed(ast: &PyAst, id: PyId, out: &mut BTreeSet<String>) {
    match ast.data(id) {
        NodeData::Assign { targets } => {
            for &t in &ast.children(id)[..*targets] {
                collect_target_names(ast, t, out);
            }
        }
        NodeData::AnnAssign { .. } | NodeData::AugAssign { .. } => {
            if let Some(&t) = ast.children(id).first() {
                collect_target_names(ast, t, out);
            }
        }
        NodeData::For { target_raw, .. } | NodeData::CompFor { target_raw, .. } => {
            collect_words(target_raw, out);
        }
        NodeData::FunctionDef {
            name, params_raw, ..
        } => {
            if is_fold_builtin(name) {
                out.insert(name.clone());
            }
            collect_words(params_raw, out);
        }
        NodeData::ClassDef { name, .. } => {
            if is_fold_builtin(name) {
                out.insert(name.clone());
            }
        }
        NodeData::Lambda { params_raw } => collect_words(params_raw, out),
        NodeData::With { items_raw, .. } => collect_words(items_raw, out),
        NodeData::Import(raw) | NodeData::FromImport(raw) => {
            collect_words(raw, out);
            // `from x import *` 可能导入任意名字（包括 len/str）；
            // 导入 builtins/__builtins__ 则可能改写内建本身——保守关闭。
            if raw.contains('*') || raw.contains("builtins") {
                for n in FOLD_BUILTINS {
                    out.insert((*n).to_string());
                }
            }
        }
        NodeData::Name(n) if n == "builtins" || n == "__builtins__" => {
            for b in FOLD_BUILTINS {
                out.insert((*b).to_string());
            }
        }
        NodeData::Global(raw) | NodeData::Nonlocal(raw) => collect_words(raw, out),
        NodeData::ExceptHandler { clause_raw } => collect_words(clause_raw, out),
        NodeData::Case { pattern_raw, .. } => collect_words(pattern_raw, out),
        NodeData::RawStmt(raw) => {
            if raw.trim_start().starts_with("exec") {
                for n in FOLD_BUILTINS {
                    out.insert((*n).to_string());
                }
            }
        }
        _ => {}
    }
    for &c in ast.children(id) {
        collect_shadowed(ast, c, out);
    }
}

fn collect_target_names(ast: &PyAst, id: PyId, out: &mut BTreeSet<String>) {
    match ast.data(id) {
        NodeData::Name(n) => {
            if is_fold_builtin(n) {
                out.insert(n.clone());
            }
        }
        NodeData::Tuple | NodeData::List | NodeData::Starred => {
            for &c in ast.children(id) {
                collect_target_names(ast, c, out);
            }
        }
        _ => {}
    }
}

fn collect_words(raw: &str, out: &mut BTreeSet<String>) {
    let mut cur = String::new();
    for c in raw.chars() {
        if c.is_alphanumeric() || c == '_' {
            cur.push(c);
        } else {
            if is_fold_builtin(&cur) {
                out.insert(cur.clone());
            }
            cur.clear();
        }
    }
    if is_fold_builtin(&cur) {
        out.insert(cur);
    }
}

fn plain_of(ast: &PyAst, id: PyId) -> Option<String> {
    match ast.data(id) {
        NodeData::Literal(l) => literal::plain_str(l).map(str::to_string),
        _ => None,
    }
}

fn int_of(ast: &PyAst, id: PyId) -> Option<i64> {
    match ast.data(id) {
        NodeData::Literal(l) if l.kind == LitKind::Int => l.int,
        _ => None,
    }
}

fn is_long_lit(l: &Literal) -> bool {
    l.kind == LitKind::Int && {
        let t = l.raw.trim_end();
        t.len() > 1 && matches!(t.as_bytes().last(), Some(b'L' | b'l'))
    }
}

fn is_str_lit(ast: &PyAst, id: PyId) -> bool {
    matches!(
        ast.data(id),
        NodeData::Literal(Literal {
            kind: LitKind::Str | LitKind::Bytes,
            ..
        })
    )
}

fn is_pure_value(ast: &PyAst, id: PyId) -> bool {
    match ast.data(id) {
        NodeData::Literal(_) => true,
        NodeData::Paren => ast
            .children(id)
            .first()
            .is_some_and(|&c| is_pure_value(ast, c)),
        NodeData::Tuple | NodeData::List => ast
            .children(id)
            .iter()
            .all(|&c| is_pure_value(ast, c)),
        _ => false,
    }
}

fn literal_truthy(l: &Literal) -> Option<bool> {
    match l.kind {
        LitKind::Bool => l.bool_val,
        LitKind::None => Some(false),
        LitKind::Int => l.int.map(|v| v != 0),
        LitKind::Str => literal::plain_str(l).map(|s| !s.is_empty()),
        _ => None,
    }
}

#[derive(Clone)]
enum SubFold {
    Lit(Literal),
    Node(PyId),
    Seq(Vec<PyId>, bool),
}

fn slice_parts(ast: &PyAst, id: PyId) -> Option<(Option<i64>, Option<i64>, Option<i64>)> {
    let (lower, upper, step) = match ast.data(id) {
        NodeData::Slice {
            lower,
            upper,
            step,
            ..
        } => (*lower, *upper, *step),
        _ => return None,
    };
    let ch = ast.children(id);
    let mut i = 0;
    let lo = if lower {
        let v = int_of(ast, *ch.get(i)?)?;
        i += 1;
        Some(v)
    } else {
        None
    };
    let up = if upper {
        let v = int_of(ast, *ch.get(i)?)?;
        i += 1;
        Some(v)
    } else {
        None
    };
    let st = if step {
        Some(int_of(ast, *ch.get(i)?)?)
    } else {
        None
    };
    Some((lo, up, st))
}

fn fold_subscript(ast: &PyAst, id: PyId) -> Option<SubFold> {
    let ch = ast.children(id);
    if ch.len() != 2 {
        return None;
    }
    let base = ch[0];
    let idx = ch[1];
    if let Some(s) = plain_of(ast, base) {
        if let Some(i) = int_of(ast, idx) {
            let out = literal::char_at(&s, i)?;
            return literal::make_str(&out).map(SubFold::Lit);
        }
        if matches!(ast.data(idx), NodeData::Slice { .. }) {
            let (l, u, st) = slice_parts(ast, idx)?;
            let out = literal::slice(&s, l, u, st)?;
            return literal::make_str(&out).map(SubFold::Lit);
        }
        return None;
    }
    let is_tuple = match ast.data(base) {
        NodeData::Tuple => true,
        NodeData::List => false,
        _ => return None,
    };
    let items = ast.children(base);
    if !items.iter().all(|&c| is_pure_value(ast, c)) {
        return None;
    }
    if let Some(i) = int_of(ast, idx) {
        let pos = literal::norm_index(items.len(), i)?;
        return Some(SubFold::Node(items[pos]));
    }
    if matches!(ast.data(idx), NodeData::Slice { .. }) {
        let (l, u, st) = slice_parts(ast, idx)?;
        let poses = literal::slice_indices(items.len(), l, u, st)?;
        let sel = poses.into_iter().map(|p| items[p]).collect();
        return Some(SubFold::Seq(sel, is_tuple));
    }
    None
}

fn fold_call(
    ast: &PyAst,
    id: PyId,
    shadowed: &BTreeSet<String>,
    unicode_literals: bool,
) -> Option<Literal> {
    let args = match ast.data(id) {
        NodeData::Call { args } => args,
        _ => return None,
    };
    let ch = ast.children(id);
    if ch.len() != args.len() + 1 || args.iter().any(|a| a.kind != ArgKind::Positional) {
        return None;
    }
    let f = ch[0];
    let av = &ch[1..];
    match ast.data(f) {
        NodeData::Name(n) => {
            if shadowed.contains(n) {
                None
            } else {
                fold_builtin(ast, n, av, unicode_literals)
            }
        }
        NodeData::Attribute { name } => {
            let base = *ast.children(f).first()?;
            let s = plain_of(ast, base)?;
            fold_str_method(ast, name, &s, av)
        }
        _ => None,
    }
}

fn fold_builtin(
    ast: &PyAst,
    name: &str,
    av: &[PyId],
    unicode_literals: bool,
) -> Option<Literal> {
    match name {
        "len" if av.len() == 1 => {
            if let Some(s) = plain_of(ast, av[0]) {
                return Some(Literal::int(s.len() as i64));
            }
            match ast.data(av[0]) {
                NodeData::List | NodeData::Tuple
                    if ast.children(av[0]).iter().all(|&c| is_pure_value(ast, c)) =>
                {
                    Some(Literal::int(ast.children(av[0]).len() as i64))
                }
                _ => None,
            }
        }
        "ord" if av.len() == 1 => {
            let s = plain_of(ast, av[0])?;
            let mut it = s.chars();
            let c = it.next()?;
            if it.next().is_some() || !c.is_ascii() {
                return None;
            }
            Some(Literal::int(c as i64))
        }
        "chr" if av.len() == 1 && !unicode_literals => {
            let n = int_of(ast, av[0])?;
            if !(32..=126).contains(&n) {
                return None;
            }
            literal::make_str(&(n as u8 as char).to_string())
        }
        "str" if av.len() == 1 && !unicode_literals => {
            let s = match ast.data(av[0]) {
                NodeData::Literal(l) => match l.kind {
                    LitKind::Bool => l
                        .bool_val
                        .map(|b| if b { "True" } else { "False" }.to_string()),
                    LitKind::None => Some("None".to_string()),
                    LitKind::Int => l.int.map(|v| v.to_string()),
                    LitKind::Str => literal::plain_str(l).map(str::to_string),
                    _ => None,
                },
                _ => None,
            }?;
            literal::make_str(&s)
        }
        "int" if av.len() == 1 => {
            let s = plain_of(ast, av[0])?;
            let t = s.trim();
            if t.is_empty() {
                return None;
            }
            let digits = t.strip_prefix(['+', '-']).unwrap_or(t);
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            t.parse::<i64>().ok().map(Literal::int)
        }
        "abs" if av.len() == 1 => match ast.data(av[0]) {
            NodeData::Literal(l) if l.kind == LitKind::Int && !is_long_lit(l) => {
                l.int?.checked_abs().map(Literal::int)
            }
            _ => None,
        },
        "bool" if av.is_empty() => Some(Literal::boolean(false)),
        "bool" if av.len() == 1 => match ast.data(av[0]) {
            NodeData::Literal(l) => literal_truthy(l).map(Literal::boolean),
            _ => None,
        },
        "min" | "max" if av.len() >= 2 => {
            let mut best: Option<i64> = None;
            for &a in av {
                match ast.data(a) {
                    NodeData::Literal(l) if l.kind == LitKind::Int && !is_long_lit(l) => {
                        let v = l.int?;
                        best = Some(match (best, name) {
                            (None, _) => v,
                            (Some(b), "min") => b.min(v),
                            (Some(b), _) => b.max(v),
                        });
                    }
                    _ => return None,
                }
            }
            best.map(Literal::int)
        }
        _ => None,
    }
}

fn fold_str_method(ast: &PyAst, name: &str, s: &str, av: &[PyId]) -> Option<Literal> {
    match name {
        "upper" if av.is_empty() => literal::make_str(&s.to_ascii_uppercase()),
        "lower" if av.is_empty() => literal::make_str(&s.to_ascii_lowercase()),
        "swapcase" if av.is_empty() => {
            let out: String = s
                .chars()
                .map(|c| {
                    if c.is_ascii_lowercase() {
                        c.to_ascii_uppercase()
                    } else if c.is_ascii_uppercase() {
                        c.to_ascii_lowercase()
                    } else {
                        c
                    }
                })
                .collect();
            literal::make_str(&out)
        }
        "capitalize" if av.is_empty() => {
            let mut out = String::with_capacity(s.len());
            for (i, c) in s.chars().enumerate() {
                if i == 0 {
                    out.push(c.to_ascii_uppercase());
                } else {
                    out.push(c.to_ascii_lowercase());
                }
            }
            literal::make_str(&out)
        }
        "strip" | "lstrip" | "rstrip" => {
            let chars = match av.len() {
                0 => None,
                1 => Some(plain_of(ast, av[0])?),
                _ => return None,
            };
            let out = literal::strip(
                s,
                chars.as_deref(),
                name != "rstrip",
                name != "lstrip",
            );
            literal::make_str(&out)
        }
        "replace" if av.len() == 2 || av.len() == 3 => {
            let old = plain_of(ast, av[0])?;
            let new = plain_of(ast, av[1])?;
            let count = if av.len() == 3 {
                Some(int_of(ast, av[2])?)
            } else {
                None
            };
            literal::replace(s, &old, &new, count).and_then(|o| literal::make_str(&o))
        }
        "startswith" | "endswith" if av.len() == 1 => {
            let p = plain_of(ast, av[0])?;
            let v = if name == "startswith" {
                s.starts_with(&p)
            } else {
                s.ends_with(&p)
            };
            Some(Literal::boolean(v))
        }
        "join" if av.len() == 1 => {
            let is_seq = matches!(ast.data(av[0]), NodeData::List | NodeData::Tuple);
            if !is_seq {
                return None;
            }
            let items = ast.children(av[0]);
            let mut parts = Vec::with_capacity(items.len());
            for &it in items {
                parts.push(plain_of(ast, it)?);
            }
            literal::make_str(&parts.join(s))
        }
        _ => None,
    }
}

fn cmp_str(op: cure_python_ast::fold::ScalarCmp, a: &str, b: &str) -> bool {
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
