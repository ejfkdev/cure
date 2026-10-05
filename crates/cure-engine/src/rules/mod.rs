//! 引擎内置通用规则：只依赖 `Lang` 抽象，任何语言 crate 均可复用。
//!
//! 每条规则文档化其语义前提；所有提案都必须严格降低成本（runner 校验）。

use std::collections::HashSet;

use crate::analysis::{prefix_effects_readable, reads_vars, subtree_contains};
use crate::effect::Effect;
use crate::kind::{BinOp, LitRef, NodeKind, UnOp};
use crate::pattern::{matches, Pat};
use crate::rule::{Edit, RewriteCtx, Rule};
use crate::lang::Lang;

// ---------------------------------------------------------------------------
// 括号消除：Paren(x) → x（纯分组节点，删除永远安全）
// ---------------------------------------------------------------------------

pub struct ParenRemoval;

impl<L: Lang> Rule<L> for ParenRemoval {
    fn name(&self) -> &'static str {
        "paren_removal"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Paren {
            return None;
        }
        let inner = *lang.children(id).first()?;
        Some(Edit::Replace {
            target: id,
            with: inner,
        })
    }
}

// ---------------------------------------------------------------------------
// 常量条件：if (true) {A} [else B] → A；if (false) {A} else B → 删除
// ---------------------------------------------------------------------------

pub struct ConstCondition;

impl<L: Lang> Rule<L> for ConstCondition {
    fn name(&self) -> &'static str {
        "const_condition"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk } = ctx;
        if lang.kind(id) != NodeKind::If {
            return None;
        }
        let ch = lang.children(id);
        let cond = *ch.first()?;
        match lang.literal(cond) {
            Some(LitRef::Bool(true)) => {
                // then 分支必然执行：拼进父块（避免嵌套块）；非块父级则整体替换
                let then = ch.get(1).copied()?;
                if let (Some(parent), Some(index)) = (walk.parent(id), walk.index(id)) {
                    if lang.kind(parent) == NodeKind::Block {
                        let insert: Vec<L::Id> = if lang.kind(then) == NodeKind::Block {
                            lang.children(then).to_vec()
                        } else {
                            vec![then]
                        };
                        return Some(Edit::Splice {
                            node: parent,
                            index,
                            remove: 1,
                            insert,
                        });
                    }
                }
                Some(Edit::Replace {
                    target: id,
                    with: then,
                })
            }
            Some(LitRef::Bool(false)) => {
                // else 分支必然执行：保留之；无 else（或空 else）才可整删
                let els = ch.get(2).copied();
                if let (Some(parent), Some(index)) = (walk.parent(id), walk.index(id)) {
                    if lang.kind(parent) == NodeKind::Block {
                        let insert: Vec<L::Id> = match els {
                            Some(e) => {
                                if lang.kind(e) == NodeKind::Block {
                                    lang.children(e).to_vec()
                                } else {
                                    vec![e]
                                }
                            }
                            None => Vec::new(),
                        };
                        return Some(if insert.is_empty() {
                            Edit::Delete { node: id }
                        } else {
                            Edit::Splice {
                                node: parent,
                                index,
                                remove: 1,
                                insert,
                            }
                        });
                    }
                }
                match els {
                    Some(e) => Some(Edit::Replace {
                        target: id,
                        with: e,
                    }),
                    None => Some(Edit::Delete { node: id }),
                }
            }
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// 布尔返回：if (c) return true else return false → return c（反极性 → return !c）
// 前提：c 为原始 boolean。
// ---------------------------------------------------------------------------

// 裸分支：if (c) return true; else return false;
static P_IF_RET_TF: Pat = Pat::Kind(
    NodeKind::If,
    &[Pat::Bind("cond"), Pat::Kind(NodeKind::Return, &[Pat::LitBool(true)]), Pat::Kind(NodeKind::Return, &[Pat::LitBool(false)])],
);
static P_IF_RET_FT: Pat = Pat::Kind(
    NodeKind::If,
    &[Pat::Bind("cond"), Pat::Kind(NodeKind::Return, &[Pat::LitBool(false)]), Pat::Kind(NodeKind::Return, &[Pat::LitBool(true)])],
);
// 块包裹分支（反编译器/规范形态）：if (c) { return true; } else { return false; }
static P_IF_BRET_TF: Pat = Pat::Kind(
    NodeKind::If,
    &[
        Pat::Bind("cond"),
        Pat::Kind(NodeKind::Block, &[Pat::Kind(NodeKind::Return, &[Pat::LitBool(true)])]),
        Pat::Kind(NodeKind::Block, &[Pat::Kind(NodeKind::Return, &[Pat::LitBool(false)])]),
    ],
);
static P_IF_BRET_FT: Pat = Pat::Kind(
    NodeKind::If,
    &[
        Pat::Bind("cond"),
        Pat::Kind(NodeKind::Block, &[Pat::Kind(NodeKind::Return, &[Pat::LitBool(false)])]),
        Pat::Kind(NodeKind::Block, &[Pat::Kind(NodeKind::Return, &[Pat::LitBool(true)])]),
    ],
);

pub struct BooleanReturn;

impl<L: Lang> Rule<L> for BooleanReturn {
    fn name(&self) -> &'static str {
        "boolean_return"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::If {
            return None;
        }
        // 四种形态：裸/块分支 × 正/反极性。块分支整体被裸 return 替换。
        let negated = |cond: L::Id, lang: &mut L| {
            let nc = lang.build_unary(UnOp::Not, cond);
            lang.build_return(Some(nc))
        };
        for (pat, invert) in [
            (&P_IF_RET_TF, false),
            (&P_IF_BRET_TF, false),
            (&P_IF_RET_FT, true),
            (&P_IF_BRET_FT, true),
        ] {
            if let Some(b) = matches(&*lang, id, pat) {
                let cond = lookup(&b, "cond")?;
                if !lang.is_bool(cond) {
                    return None;
                }
                let ret = if invert {
                    negated(cond, lang)
                } else {
                    lang.build_return(Some(cond))
                };
                return Some(Edit::Replace {
                    target: id,
                    with: ret,
                });
            }
        }
        None
    }
}

fn lookup<Id: Copy>(binds: &[(&'static str, Id)], name: &str) -> Option<Id> {
    binds.iter().find(|(n, _)| *n == name).map(|(_, id)| *id)
}

// ---------------------------------------------------------------------------
// 空分支：if (c) {} else {B} → if (!c) {B}；if (c) {A} else {} → if (c) {A}；
//         if (c) {} → 删除（仅当 c 无副作用）。
// ---------------------------------------------------------------------------

pub struct IfElseEmpty;

impl<L: Lang> Rule<L> for IfElseEmpty {
    fn name(&self) -> &'static str {
        "if_else_empty"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::If {
            return None;
        }
        let ch = lang.children(id);
        let cond = *ch.first()?;
        let is_empty_block = |n: L::Id| -> bool {
            lang.kind(n) == NodeKind::Block && lang.children(n).is_empty()
        };
        match ch.len() {
            3 => {
                let (then, els) = (ch[1], ch[2]);
                if is_empty_block(then) {
                    if is_empty_block(els) {
                        // 两支全空：塌缩成 if(c){}，交给 len==2 分支处理
                        return Some(Edit::Splice {
                            node: id,
                            index: 0,
                            remove: 3,
                            insert: vec![cond, then],
                        });
                    }
                    // then 空、else 有内容：取反条件交换分支（条件两侧都恰好求值一次，永远安全）
                    let nc = lang.build_unary(UnOp::Not, cond);
                    return Some(Edit::Splice {
                        node: id,
                        index: 0,
                        remove: 3,
                        insert: vec![nc, els],
                    });
                }
                if is_empty_block(els) {
                    return Some(Edit::Splice {
                        node: id,
                        index: 0,
                        remove: 3,
                        insert: vec![cond, then],
                    });
                }
                None
            }
            2 => {
                let then = ch[1];
                if is_empty_block(then) && lang.effect(cond) <= Effect::MayRead {
                    // 删除整个 if 会丢掉 cond 的求值，仅当 cond 无副作用时安全
                    return Some(Edit::Delete { node: id });
                }
                None
            }
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// 布尔比较：x == true → x；x == false → !x；x != true → !x；x != false → x（两个操作数序）
// 前提：x 为原始 boolean（装箱比较是引用比较，is_bool 必须排除）。
// ---------------------------------------------------------------------------

pub struct BoolCompare;

impl<L: Lang> Rule<L> for BoolCompare {
    fn name(&self) -> &'static str {
        "bool_compare"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Binary {
            return None;
        }
        let op = lang.bin_op(id)?;
        if !matches!(op, BinOp::Eq | BinOp::Ne) {
            return None;
        }
        let ch = lang.children(id);
        let (l, r) = (*ch.first()?, *ch.get(1)?);
        let lit_bool = |n: L::Id| -> Option<bool> {
            match lang.literal(n) {
                Some(LitRef::Bool(b)) => Some(b),
                _ => None,
            }
        };
        // 统一成 (x, lit)
        let (x, lit) = match (lit_bool(l), lit_bool(r)) {
            (None, Some(b)) => (l, b),
            (Some(b), None) => (r, b),
            _ => return None,
        };
        if !lang.is_bool(x) {
            return None;
        }
        // (op, lit) → 替换结果
        let neg = match (op, lit) {
            (BinOp::Eq, true) | (BinOp::Ne, false) => false,
            (BinOp::Eq, false) | (BinOp::Ne, true) => true,
            _ => return None,
        };
        let with = if neg {
            lang.build_unary(UnOp::Not, x)
        } else {
            x
        };
        Some(Edit::Replace {
            target: id,
            with,
        })
    }
}

// ---------------------------------------------------------------------------
// 双重否定：!!x → x
// 安全性：`!x` 要能通过类型检查，x 必为 boolean，无需类型查询。
// ---------------------------------------------------------------------------

pub struct DoubleNot;

impl<L: Lang> Rule<L> for DoubleNot {
    fn name(&self) -> &'static str {
        "double_not"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Unary || lang.un_op(id) != Some(UnOp::Not) {
            return None;
        }
        let inner = *lang.children(id).first()?;
        if lang.kind(inner) != NodeKind::Unary || lang.un_op(inner) != Some(UnOp::Not) {
            return None;
        }
        let x = *lang.children(inner).first()?;
        Some(Edit::Replace {
            target: id,
            with: x,
        })
    }
}

// ---------------------------------------------------------------------------
// 常量折叠：!true → false / !false → true
// （局部传播把常量内联进 !x 后，由此规则收敛到自然形态）
// ---------------------------------------------------------------------------

pub struct BoolNotFold;

impl<L: Lang> Rule<L> for BoolNotFold {
    fn name(&self) -> &'static str {
        "bool_not_fold"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Unary || lang.un_op(id) != Some(UnOp::Not) {
            return None;
        }
        let operand = *lang.children(id).first()?;
        match lang.literal(operand) {
            Some(LitRef::Bool(b)) => {
                let folded = lang.build_bool(!b);
                Some(Edit::Replace {
                    target: id,
                    with: folded,
                })
            }
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// 自赋值：x = x → 删除（仅限语言层局部变量）
// ---------------------------------------------------------------------------

pub struct SelfAssign;

impl<L: Lang> Rule<L> for SelfAssign {
    fn name(&self) -> &'static str {
        "self_assign"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Assign {
            return None;
        }
        if lang.assign_op(id).is_some() {
            return None; // x += x 有读语义差异，不动
        }
        let ch = lang.children(id);
        let (t, v) = (*ch.first()?, *ch.get(1)?);
        if lang.kind(t) != NodeKind::VarRef || lang.kind(v) != NodeKind::VarRef {
            return None;
        }
        if lang.var_name(t) != lang.var_name(v) || !lang.is_local_var(t) {
            return None;
        }
        Some(Edit::Delete { node: id })
    }
}

// ---------------------------------------------------------------------------
// 算术恒等式（仅精确整数，浮点有 -0.0/NaN 陷阱）：
// x+0 → x, 0+x → x, x-0 → x, x*1 → x, 1*x → x, x/1 → x
// ---------------------------------------------------------------------------

pub struct ArithIdentity;

impl<L: Lang> Rule<L> for ArithIdentity {
    fn name(&self) -> &'static str {
        "arith_identity"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Binary {
            return None;
        }
        let op = lang.bin_op(id)?;
        let ch = lang.children(id);
        let (l, r) = (*ch.first()?, *ch.get(1)?);
        let int_lit = |n: L::Id, v: i64| -> bool {
            matches!(lang.literal(n), Some(l) if l.as_int() == Some(v))
        };
        let x = match op {
            BinOp::Add if int_lit(r, 0) => l,
            BinOp::Add if int_lit(l, 0) => r,
            BinOp::Sub if int_lit(r, 0) => l,
            BinOp::Mul if int_lit(r, 1) => l,
            BinOp::Mul if int_lit(l, 1) => r,
            BinOp::Div if int_lit(r, 1) => l,
            _ => return None,
        };
        if !lang.is_exact_int(x) {
            return None;
        }
        Some(Edit::Replace {
            target: id,
            with: x,
        })
    }
}

// ---------------------------------------------------------------------------
// 局部变量传播：
//   T x = V; …; [唯一一次读 x，且区间内无对相关变量的写、无遮蔽]
//   → 用 V 替换该次读，删除声明。
//
// 安全条件：
//   - effect(V) ≤ MayRead（纯值/局部读）：可自由移动，只要求区间内的
//     显式局部写不与 V 读到的变量相交；
//   - effect(V) > MayRead（可能抛/写/未知）：仅当声明与唯一使用**相邻**、
//     使用处于直线语句（return/expr-stmt/assign/throw）、路径无条件求值、
//     且 use 之前求值的兄弟子树效果 ≤ MayRead 时才内联
//     （保持 V 先于语句内其他效果求值）。
// ---------------------------------------------------------------------------

pub struct LocalPropagation;

impl<L: Lang> Rule<L> for LocalPropagation {
    fn name(&self) -> &'static str {
        "local_propagation"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk } = ctx;
        if lang.kind(id) != NodeKind::VarDecl {
            return None;
        }
        let name = lang.var_name(id)?.to_string();
        let ch = lang.children(id);
        if ch.len() != 1 {
            return None; // 必须有且仅有 init
        }
        let value = ch[0];
        // 自引用 init（int x = x + 1 之类）直接放弃
        if subtree_contains(&*lang, value, |n| {
            lang.kind(n) == NodeKind::VarRef && lang.var_name(n) == Some(name.as_str())
        }) {
            return None;
        }
        let parent = walk.parent(id)?;
        if lang.kind(parent) != NodeKind::Block {
            return None;
        }
        let index = walk.index(id)?;
        let stmts: Vec<L::Id> = lang.children(parent).to_vec();
        if index + 1 > stmts.len() {
            return None;
        }

        // 扫描 decl 之后的区域
        let mut uses: Vec<L::Id> = Vec::new();
        let mut writes: HashSet<String> = HashSet::new();
        let mut shadowed = false;
        for &s in &stmts[index + 1..] {
            scan_region(&*lang, s, &name, &mut uses, &mut writes, &mut shadowed);
        }
        if shadowed || uses.len() != 1 {
            return None;
        }
        let use_id = uses[0];
        let ve = lang.effect(value);

        if ve <= Effect::MayRead {
            // 自由移动：区间内的显式局部写不得触碰 V 读到的变量
            // （对声明变量自身的写也包含在 writes 里，一票否决）
            let mut reads = HashSet::new();
            reads_vars(&*lang, value, &mut reads);
            reads.insert(name);
            for w in &writes {
                if reads.contains(w) {
                    return None;
                }
            }
            return Some(Edit::Multi(vec![
                Edit::Replace {
                    target: use_id,
                    with: value,
                },
                Edit::Delete { node: id },
            ]));
        }

        // 有副作用的 V：只允许相邻 + 直线 + 前缀可读
        if index + 1 >= stmts.len() {
            return None;
        }
        let stmt = stmts[index + 1];
        if !subtree_contains(&*lang, stmt, |n| n == use_id) {
            return None;
        }
        match lang.kind(stmt) {
            NodeKind::Return | NodeKind::ExprStmt | NodeKind::Assign | NodeKind::Throw => {}
            _ => return None,
        }
        if !straight_path(&*lang, stmt, use_id) {
            return None;
        }
        if !prefix_effects_readable(&*lang, stmt, use_id) {
            return None;
        }
        Some(Edit::Multi(vec![
            Edit::Replace {
                target: use_id,
                with: value,
            },
            Edit::Delete { node: id },
        ]))
    }
}

/// 扫描一个语句子树：收集对 `name` 的**读**、所有显式局部写、同名声明遮蔽。
fn scan_region<L: Lang>(
    lang: &L,
    node: L::Id,
    name: &str,
    uses: &mut Vec<L::Id>,
    writes: &mut HashSet<String>,
    shadowed: &mut bool,
) {
    match lang.kind(node) {
        NodeKind::Assign => {
            let ch = lang.children(node);
            if let Some(&t) = ch.first() {
                if lang.kind(t) == NodeKind::VarRef {
                    if let Some(n) = lang.var_name(t) {
                        writes.insert(n.to_string());
                    }
                }
            }
            if let Some(&v) = ch.get(1) {
                scan_region(lang, v, name, uses, writes, shadowed);
            }
        }
        NodeKind::Unary if lang.un_op(node).is_some_and(|o| o.is_incdec()) => {
            let ch = lang.children(node);
            if let Some(&t) = ch.first() {
                if lang.kind(t) == NodeKind::VarRef {
                    if let Some(n) = lang.var_name(t) {
                        writes.insert(n.to_string());
                    }
                }
            }
        }
        NodeKind::VarDecl => {
            if let Some(n) = lang.var_name(node) {
                if n == name {
                    *shadowed = true;
                }
                writes.insert(n.to_string());
            }
            for &c in lang.children(node) {
                scan_region(lang, c, name, uses, writes, shadowed);
            }
        }
        // ForEach 循环变量 / Catch 绑定：在子作用域声明了同名变量 → 视为遮蔽；
        // ForEach 每轮给循环变量赋值 → 也算显式写。
        NodeKind::ForEach | NodeKind::Catch => {
            if let Some(n) = lang.var_name(node) {
                if n == name {
                    *shadowed = true;
                }
                if lang.kind(node) == NodeKind::ForEach {
                    writes.insert(n.to_string());
                }
            }
            for &c in lang.children(node) {
                scan_region(lang, c, name, uses, writes, shadowed);
            }
        }
        NodeKind::VarRef => {
            if lang.var_name(node) == Some(name) {
                uses.push(node);
            }
        }
        _ => {
            for &c in lang.children(node) {
                scan_region(lang, c, name, uses, writes, shadowed);
            }
        }
    }
}

/// root → target 的路径上不得有条件求值结构（Ternary / If / 短路右支）。
fn straight_path<L: Lang>(lang: &L, root: L::Id, target: L::Id) -> bool {
    fn go<L: Lang>(lang: &L, node: L::Id, target: L::Id) -> bool {
        if node == target {
            return true;
        }
        match lang.kind(node) {
            NodeKind::Ternary | NodeKind::If => return false,
            NodeKind::Binary if lang.bin_op(node).is_some_and(|o| o.is_short_circuit()) => {
                return false
            }
            _ => {}
        }
        for &c in lang.children(node) {
            if subtree_contains(lang, c, |n| n == target) {
                return go(lang, c, target);
            }
        }
        false
    }
    go(lang, root, target)
}


// ---------------------------------------------------------------------------
// 短路折叠：
//   false && x → false / true || x → true（x 原本就不求值，恒安全）
//   true && x → x / false || x → x / x && true → x / x || false → x（x 保留，恒安全）
//   x && false → false / x || true → true（x 的求值被丢弃，需 effect ≤ MayRead）
// ---------------------------------------------------------------------------

pub struct BoolShortCircuit;

impl<L: Lang> Rule<L> for BoolShortCircuit {
    fn name(&self) -> &'static str {
        "bool_short_circuit"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Binary {
            return None;
        }
        let op = lang.bin_op(id)?;
        if !op.is_short_circuit() {
            return None;
        }
        let ch = lang.children(id);
        let (l, r) = (*ch.first()?, *ch.get(1)?);
        let is_lit = |n: L::Id, v: bool| matches!(lang.literal(n), Some(LitRef::Bool(b)) if b == v);
        // 左侧字面量：短路后 x 可能根本不求值 → 直接取字面量结果
        let with = match (op, is_lit(l, true), is_lit(l, false)) {
            (BinOp::And, _, true) => l,  // false && x → false（x 不求值）
            (BinOp::Or, true, _) => l,   // true || x → true（x 不求值）
            (BinOp::And, true, _) => r,  // true && x → x
            (BinOp::Or, _, true) => r,   // false || x → x
            _ => {
                // 右侧字面量：x && true → x / x || false → x（x 保留求值，恒安全）；
                // x && false → false / x || true → true（x 的求值被丢弃，需可丢弃）
                match (op, is_lit(r, true), is_lit(r, false)) {
                    (BinOp::And, true, _) => l,
                    (BinOp::Or, _, true) => l,
                    (BinOp::And, _, true) | (BinOp::Or, true, _) => {
                        if lang.effect(l) <= Effect::MayRead {
                            r
                        } else {
                            return None;
                        }
                    }
                    _ => return None,
                }
            }
        };
        Some(Edit::Replace {
            target: id,
            with,
        })
    }
}

// ---------------------------------------------------------------------------
// 比较取反：!(a == b) → a != b（恒安全，含 NaN/引用）；
// !(a < b) → a >= b 等次序比较仅精确整数（浮点 NaN 下不成立）。
// ---------------------------------------------------------------------------

pub struct NotCompare;

impl<L: Lang> Rule<L> for NotCompare {
    fn name(&self) -> &'static str {
        "not_compare"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Unary || lang.un_op(id) != Some(UnOp::Not) {
            return None;
        }
        let inner = *lang.children(id).first()?;
        if lang.kind(inner) != NodeKind::Binary {
            return None;
        }
        let op = lang.bin_op(inner)?;
        let neg = match op {
            BinOp::Eq => BinOp::Ne,
            BinOp::Ne => BinOp::Eq,
            BinOp::Lt => BinOp::Ge,
            BinOp::Le => BinOp::Gt,
            BinOp::Gt => BinOp::Le,
            BinOp::Ge => BinOp::Lt,
            _ => return None,
        };
        if op.is_short_circuit() {
            return None;
        }
        // 次序比较的取反仅对精确整数成立（NaN: !(a<b)=true 但 a>=b=false）
        if matches!(op, BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge) {
            let ch = lang.children(inner);
            if !lang.is_exact_int(*ch.first()?) || !lang.is_exact_int(*ch.get(1)?) {
                return None;
            }
        }
        let ch = lang.children(inner).to_vec();
        let rebuilt = lang.build_bin(neg, ch[0], ch[1]);
        Some(Edit::Replace {
            target: id,
            with: rebuilt,
        })
    }
}

// ---------------------------------------------------------------------------
// 三元折叠：true ? a : b → a；false ? a : b → b（另一支原本不求值，恒安全）；
//           c ? a : a → a（c 的求值被丢弃，需 effect ≤ MayRead）
// ---------------------------------------------------------------------------

pub struct TernaryFold;

impl<L: Lang> Rule<L> for TernaryFold {
    fn name(&self) -> &'static str {
        "ternary_fold"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Ternary {
            return None;
        }
        let ch = lang.children(id);
        let (c, a, b) = (*ch.first()?, *ch.get(1)?, *ch.get(2)?);
        match lang.literal(c) {
            Some(LitRef::Bool(true)) => return Some(Edit::Replace { target: id, with: a }),
            Some(LitRef::Bool(false)) => return Some(Edit::Replace { target: id, with: b }),
            _ => {}
        }
        if crate::analysis::structurally_equal(&*lang, a, b) && lang.effect(c) <= Effect::MayRead {
            return Some(Edit::Replace { target: id, with: a });
        }
        None
    }
}

// ---------------------------------------------------------------------------
// 三元布尔：c ? true : false → c；c ? false : true → !c（c 为原始 boolean）
// ---------------------------------------------------------------------------

pub struct TernaryBool;

impl<L: Lang> Rule<L> for TernaryBool {
    fn name(&self) -> &'static str {
        "ternary_bool"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Ternary {
            return None;
        }
        let ch = lang.children(id);
        let (c, a, b) = (*ch.first()?, *ch.get(1)?, *ch.get(2)?);
        let lb = |n: L::Id| matches!(lang.literal(n), Some(LitRef::Bool(v)) if v);
        let is_false = |n: L::Id| matches!(lang.literal(n), Some(LitRef::Bool(v)) if !v);
        if !lang.is_bool(c) {
            return None;
        }
        let with = if lb(a) && is_false(b) {
            c
        } else if is_false(a) && lb(b) {
            lang.build_unary(UnOp::Not, c)
        } else {
            return None;
        };
        Some(Edit::Replace {
            target: id,
            with,
        })
    }
}

// ---------------------------------------------------------------------------
// 整数常量折叠（Java 包装算术语义）：1+2→3、2*3→6、1<<3→8、& | ^ 等。
// 除/余 除数为 0 不折叠（异常语义）。Int 按 i32 回绕，Long 按 i64。
// 字符串字面量拼接："a"+"b"→"ab"。
// ---------------------------------------------------------------------------

pub struct ConstFoldBin;

impl<L: Lang> Rule<L> for ConstFoldBin {
    fn name(&self) -> &'static str {
        "const_fold_bin"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Binary {
            return None;
        }
        let op = lang.bin_op(id)?;
        if op.is_short_circuit() || op.is_comparison() {
            return None;
        }
        let ch = lang.children(id);
        let (l, r) = (*ch.first()?, *ch.get(1)?);
        // 字符串拼接
        if op == BinOp::Add {
            if let (Some(LitRef::Str(a)), Some(LitRef::Str(b))) = (lang.literal(l), lang.literal(r)) {
                let joined = format!("{a}{b}");
                let with = lang.build_str(&joined);
                return Some(Edit::Replace { target: id, with });
            }
        }
        let (a, b) = (
            lang.literal(l).and_then(|x| x.as_int())?,
            lang.literal(r).and_then(|x| x.as_int())?,
        );
        let is_long = matches!(lang.literal(l), Some(LitRef::Long(_)))
            || matches!(lang.literal(r), Some(LitRef::Long(_)));
        let folded: Option<i64> = if is_long {
            fold_i64(op, a, b)
        } else {
            fold_i32(op, a, b).map(|v| v as i64)
        };
        let v = folded?;
        let with = lang.build_int(v, is_long);
        Some(Edit::Replace {
            target: id,
            with,
        })
    }
}

fn fold_i64(op: BinOp, a: i64, b: i64) -> Option<i64> {
    use BinOp::*;
    Some(match op {
        Add => a.wrapping_add(b),
        Sub => a.wrapping_sub(b),
        Mul => a.wrapping_mul(b),
        Div => {
            if b == 0 {
                return None;
            }
            a.wrapping_div(b)
        }
        Rem => {
            if b == 0 {
                return None;
            }
            a.wrapping_rem(b)
        }
        Shl => a.wrapping_shl((b as u64 & 63) as u32),
        Shr => a.wrapping_shr((b as u64 & 63) as u32),
        UShr => ((a as u64).wrapping_shr((b as u64 & 63) as u32)) as i64,
        BitAnd => a & b,
        BitXor => a ^ b,
        BitOr => a | b,
        _ => return None,
    })
}

fn fold_i32(op: BinOp, a: i64, b: i64) -> Option<i64> {
    use BinOp::*;
    let (x, y) = (a as i32, b as i32);
    Some(match op {
        Add => x.wrapping_add(y) as i64,
        Sub => x.wrapping_sub(y) as i64,
        Mul => x.wrapping_mul(y) as i64,
        Div => {
            if y == 0 {
                return None;
            }
            x.wrapping_div(y) as i64
        }
        Rem => {
            if y == 0 {
                return None;
            }
            x.wrapping_rem(y) as i64
        }
        Shl => x.wrapping_shl((y as u32) & 31) as i64,
        Shr => x.wrapping_shr((y as u32) & 31) as i64,
        UShr => (x as u32).wrapping_shr(x as u32 & 31) as i64,
        BitAnd => (x & y) as i64,
        BitXor => (x ^ y) as i64,
        BitOr => (x | y) as i64,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// 零元素 / 平凡算术：
//   0 * x → 0、x * 0 → 0（x 求值被丢弃：需 effect ≤ MayRead 且精确整数——
//     浮点 0.0*NaN=NaN）
//   x - x → 0（同名局部整型变量）
//   x << 0 / x >> 0 / x >>> 0 → x（位移 0 恒等，x 保留，恒安全）
// ---------------------------------------------------------------------------

pub struct ArithZero;

impl<L: Lang> Rule<L> for ArithZero {
    fn name(&self) -> &'static str {
        "arith_zero"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Binary {
            return None;
        }
        let op = lang.bin_op(id)?;
        let ch = lang.children(id);
        let (l, r) = (*ch.first()?, *ch.get(1)?);
        let int_lit = |n: L::Id, v: i64| lang.literal(n).and_then(|x| x.as_int()) == Some(v);

        // 移位 0：x << 0 → x（恒等）
        if matches!(op, BinOp::Shl | BinOp::Shr | BinOp::UShr) && int_lit(r, 0) {
            return Some(Edit::Replace {
                target: id,
                with: l,
            });
        }
        // 0 * x → 0 / x * 0 → 0
        if op == BinOp::Mul {
            let (zero, other) = if int_lit(l, 0) {
                (l, r)
            } else if int_lit(r, 0) {
                (r, l)
            } else {
                return None;
            };
            if !lang.is_exact_int(other) || lang.effect(other) > Effect::MayRead {
                return None;
            }
            return Some(Edit::Replace {
                target: id,
                with: zero,
            });
        }
        // x - x → 0（同名局部变量，整型）
        if op == BinOp::Sub
            && lang.kind(l) == NodeKind::VarRef
            && lang.kind(r) == NodeKind::VarRef
            && lang.var_name(l) == lang.var_name(r)
            && lang.is_local_var(l)
            && lang.is_exact_int(l)
        {
            let with = lang.build_int(0, false);
            return Some(Edit::Replace {
                target: id,
                with,
            });
        }
        None
    }
}

// ---------------------------------------------------------------------------
// 死赋值：x = a; x = b; → x = b;（a 的求值被丢弃：局部变量 x + effect(a) ≤ MayRead）
// 含声明形：int x = a; x = b; → int x = b;
// ---------------------------------------------------------------------------

pub struct DeadStore;

impl<L: Lang> Rule<L> for DeadStore {
    fn name(&self) -> &'static str {
        "dead_store"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk } = ctx;
        // 归一化第一条语句：VarDecl / Assign / ExprStmt{Assign}
        let (decl_form, first_assign) = match lang.kind(id) {
            NodeKind::VarDecl => {
                if lang.children(id).is_empty() {
                    return None; // 无 init 的声明不适用
                }
                (true, None)
            }
            NodeKind::Assign => (false, Some(id)),
            NodeKind::ExprStmt => {
                let ch = lang.children(id);
                if ch.len() == 1 && lang.kind(ch[0]) == NodeKind::Assign {
                    (false, Some(ch[0]))
                } else {
                    return None;
                }
            }
            _ => return None,
        };
        let first_value = if decl_form {
            *lang.children(id).first()?
        } else {
            let a = first_assign?;
            if lang.assign_op(a).is_some() {
                return None; // 复合赋值有读语义
            }
            *lang.children(a).get(1)?
        };

        // 父 Block + 相邻下一条语句
        let parent = walk.parent(id)?;
        if lang.kind(parent) != NodeKind::Block {
            return None;
        }
        let idx = walk.index(id)?;
        let stmts = lang.children(parent).to_vec();
        let next_stmt = *stmts.get(idx + 1)?;

        // 归一化第二条语句：Assign / ExprStmt{Assign}（简单赋值）
        let second = match lang.kind(next_stmt) {
            NodeKind::Assign => next_stmt,
            NodeKind::ExprStmt => {
                let ch = lang.children(next_stmt);
                if ch.len() == 1 && lang.kind(ch[0]) == NodeKind::Assign {
                    ch[0]
                } else {
                    return None;
                }
            }
            _ => return None,
        };
        if lang.assign_op(second).is_some() {
            return None;
        }
        let nch = lang.children(second);
        let (target, value) = (*nch.first()?, *nch.get(1)?);
        if lang.kind(target) != NodeKind::VarRef {
            return None;
        }

        // 变量一致性 + 局部性
        let name = if decl_form {
            if lang.var_name(id) != lang.var_name(target) {
                return None;
            }
            lang.var_name(id).map(|s| s.to_string())
        } else {
            let a = first_assign?;
            let t = *lang.children(a).first()?;
            if lang.kind(t) != NodeKind::VarRef {
                return None;
            }
            if lang.var_name(t) != lang.var_name(target) || !lang.is_local_var(t) {
                return None;
            }
            lang.var_name(t).map(|s| s.to_string())
        }?;
        // b 中不得引用 x（x = b 依赖旧值时不可删第一条）
        if subtree_contains(&*lang, value, |n| {
            lang.kind(n) == NodeKind::VarRef && lang.var_name(n) == Some(name.as_str())
        }) {
            return None;
        }
        // a 的求值被丢弃：effect ≤ MayRead
        if lang.effect(first_value) > Effect::MayRead {
            return None;
        }

        if decl_form {
            // int x = a; x = b; → init 换成 b，删除第二条语句
            Some(Edit::Multi(vec![
                Edit::Replace {
                    target: first_value,
                    with: value,
                },
                Edit::Delete {
                    node: next_stmt,
                },
            ]))
        } else {
            // x = a; x = b; → 删除第一条语句（id 本身：Assign 或 ExprStmt）
            Some(Edit::Delete { node: id })
        }
    }
}

// ---------------------------------------------------------------------------
// 不可达语句删除（块内 return/throw/break/continue 之后的语句）。
// 语义安全（永不执行），但属于 DCE 范畴——**默认不启用**，经 all_rules() 选配。
// ---------------------------------------------------------------------------

pub struct UnreachableAfterTerminal;

impl<L: Lang> Rule<L> for UnreachableAfterTerminal {
    fn name(&self) -> &'static str {
        "unreachable_after_terminal"
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Block {
            return None;
        }
        let children = lang.children(id).to_vec();
        let terminal = |n: L::Id| -> bool {
            matches!(
                lang.kind(n),
                NodeKind::Return | NodeKind::Throw | NodeKind::Break | NodeKind::Continue
            )
        };
        let mut cut: Option<(usize, usize)> = None;
        for (i, &s) in children.iter().enumerate() {
            if terminal(s) && i + 1 < children.len() {
                cut = Some((i + 1, children.len() - i - 1));
                break;
            }
        }
        let (index, remove) = cut?;
        Some(Edit::Splice {
            node: id,
            index,
            remove,
            insert: Vec::new(),
        })
    }
}

// ---------------------------------------------------------------------------
// 注册表
// ---------------------------------------------------------------------------

/// 默认规则（保守集：不做 DCE 类清理）。
pub fn default_rules<L: Lang>() -> Vec<Box<dyn Rule<L>>> {
    vec![
        Box::new(ParenRemoval),
        Box::new(ConstCondition),
        Box::new(BooleanReturn),
        Box::new(IfElseEmpty),
        Box::new(BoolCompare),
        Box::new(DoubleNot),
        Box::new(BoolNotFold),
        Box::new(BoolShortCircuit),
        Box::new(NotCompare),
        Box::new(TernaryFold),
        Box::new(TernaryBool),
        Box::new(ConstFoldBin),
        Box::new(SelfAssign),
        Box::new(ArithIdentity),
        Box::new(ArithZero),
        Box::new(LocalPropagation),
        Box::new(DeadStore),
    ]
}

/// 全量规则（含 DCE 类选配项；按设计文档"可选配，默认不启用"）。
pub fn all_rules<L: Lang>() -> Vec<Box<dyn Rule<L>>> {
    let mut rules = default_rules();
    rules.push(Box::new(UnreachableAfterTerminal));
    rules
}
