//! 引擎内置通用规则：只依赖 `Lang` 抽象，任何语言 crate 均可复用。
//!
//! 每条规则文档化其语义前提；所有提案都必须严格降低成本（runner 校验）。



use crate::analysis::{prefix_effects_readable, subtree_contains};
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Paren]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::If]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::If]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::If]
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk } = ctx;
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
                // 【位置守卫】Delete 只在语句位置（父为 Block）合法：
                // if 处于分支位置（if(c){ if(c2){} }——else-if 链化后的
                // 常见形态）时，抽走分支会给父 If 留下"只有条件没有体"的
                // 残骸（1-child If，打印机越界 panic——真实大语料抓获）。
                // 分支位置的空体 if 由父 If 的规则吸收（塌缩后自身再走
                // len==2/len==3 变换）。
                let parent_is_block = walk
                    .parent(id)
                    .map(|p| lang.kind(p) == NodeKind::Block)
                    .unwrap_or(false);
                if is_empty_block(then)
                    && lang.effect(cond) <= Effect::MayRead
                    && parent_is_block
                {
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Binary]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Unary]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Unary]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Assign]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Binary]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::VarDecl]
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
        // 裸数组字面量只在声明/赋值 RHS 位置合法（Java/多数语言同此）；
        // 传播到任意表达式位置会产出非法源码，且多站点传播会破坏数组对象
        // 同一性 → 拒绝（显式 new T[]{…} 形态可安全传播）
        if lang.kind(value) == NodeKind::ArrayLit {
            return None;
        }
        // lambda / 方法引用值：内联进 receiver 位产出非法 Java
        //（(() -> {}).run() / String::length.apply()——对抗波 2 抓获）。
        // 参数位合法但罕见——统一保守拒绝。
        if matches!(lang.kind(value), NodeKind::Lambda | NodeKind::MethodRef) {
            return None;
        }
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

        // 扫描 decl 之后的区域（用途/遮蔽全区间；写冲突窗口见下）
        let name_key = lang.var_key(id)?;
        let mut wa = Watch::<L>::new(name_key, &[]);
        for &s in &stmts[index + 1..] {
            scan_region(&*lang, s, &mut wa);
        }
        if wa.opaque || wa.shadowed || wa.uses.len() != 1 {
            return None;
        }
        let use_id = wa.uses[0];
        // value 读到的名字键（写冲突兴趣集；reads 含 name 自身——一票否决）
        let mut watch_keys: Vec<L::NameKey> = Vec::new();
        collect_read_keys(&*lang, value, &mut watch_keys);
        watch_keys.push(name_key);
        // 写冲突窗口 = [decl 后, 使用语句]：纯值移动到使用点，
        // 使用点之后的写不影响（值已被消费）。首个含 use 的语句即使用语句
        //（uses[0] 是遍历序首个读，与 subtree_contains 等价）
        let mut wb = Watch::<L>::new(name_key, &watch_keys);
        let mut use_stmt_idx = None;
        for (off, &s) in stmts[index + 1..].iter().enumerate() {
            let uses_before = wb.uses.len();
            scan_region(&*lang, s, &mut wb);
            if wb.uses.len() > uses_before {
                use_stmt_idx = Some(index + 1 + off);
                break; // 使用语句之后的写不参与冲突判定
            }
        }
        // 声明删除前提：use 之后不得再出现对该名字的任何引用（读/写/遮蔽）。
        // 值的传播只关心窗口内写，但**删声明**要求名字彻底无残留引用——
        // 使用点之后的死写（如 v = "y"）同样引用声明，残留会让输出失去声明。
        if let Some(ui) = use_stmt_idx {
            let mut wc = Watch::<L>::new(name_key, &[name_key]);
            for &s in &stmts[ui + 1..] {
                scan_region(&*lang, s, &mut wc);
                if wc.opaque || !wc.uses.is_empty() || wc.wrote(name_key) || wc.shadowed {
                    return None;
                }
            }
        }
        let ve = lang.effect(value);

        if ve <= Effect::MayRead {
            // 自由移动：区间内的显式局部写不得触碰 V 读到的变量
            // （对声明变量自身的写也包含在 writes 里，一票否决）
            if let Some(target_key) =
                use_stmt_target_write_key(&*lang, &stmts[index + 1..], use_id)
            {
                wb.clear_wrote(target_key);
            }
            if wb.opaque || wb.any_write() {
                return None;
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
            // VarDecl 同样合法：init 就是语句的全部求值，V 内联进 init 的
            // 求值时机与原声明位置一致（相邻保证无插入效果）
            NodeKind::Return
            | NodeKind::ExprStmt
            | NodeKind::Assign
            | NodeKind::Throw
            | NodeKind::VarDecl => {}
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

/// 区域扫描观察：主名字的读 + 小兴趣集的写命中 + 同名遮蔽。
///
/// 规则预先声明感兴趣的**写名字**（通常 ≤4：主名字 + value 读到的变量），
/// 收集期直接判中——零哈希、零集合重建。对比旧版：每次检查重建
/// `HashSet<&str>`（分配/重哈希/释放）曾占 release 运行时间 ~50%。
pub(crate) struct Watch<L: Lang> {
    /// 区域含不透明（Raw）内容：读/写集不可证明——所有守卫保守拒绝
    opaque: bool,
    /// 主名字键：读收集 + 遮蔽判定（整数等值比较）
    name: L::NameKey,
    /// 写冲突兴趣名字键（典型 2~4 个）
    watch: Vec<L::NameKey>,
    /// 与 watch 一一对应：窗口内是否出现过该名字的写
    writes_hit: Vec<bool>,
    shadowed: bool,
    /// 主名字的读（遍历序）
    uses: Vec<L::Id>,
}

impl<L: Lang> Watch<L> {
    pub(crate) fn new(name: L::NameKey, watch: &[L::NameKey]) -> Self {
        Watch {
            opaque: false,
            name,
            watch: watch.to_vec(),
            writes_hit: vec![false; watch.len()],
            shadowed: false,
            uses: Vec::new(),
        }
    }
    /// 名字键 `k` 在窗口内是否被写（聚合判断）
    pub(crate) fn wrote(&self, k: L::NameKey) -> bool {
        self.watch
            .iter()
            .zip(&self.writes_hit)
            .any(|(w, h)| *h && *w == k)
    }
    /// 任一兴趣名字被写
    pub(crate) fn any_write(&self) -> bool {
        self.writes_hit.iter().any(|h| *h)
    }
    /// 清除对 `k` 的写命中（use 语句自身对目标的写不参与冲突判定）
    pub(crate) fn clear_wrote(&mut self, k: L::NameKey) {
        for (w, h) in self.watch.iter().zip(self.writes_hit.iter_mut()) {
            if *w == k {
                *h = false;
            }
        }
    }
}

/// 扫描一个语句子树：主名字的**读**、兴趣名字的**写命中**、同名声明遮蔽。
fn scan_region<L: Lang>(lang: &L, node: L::Id, w: &mut Watch<L>) {
    lang.debug_verify_events(node);
    if let Some(events) = lang.region_events(node) {
        // 主名字区间：Use 收集 + Shadow 命中 + Write 命中（name 可能在 watch 里）
        let lo = events.partition_point(|e| e.key < w.name);
        let hi = events.partition_point(|e| e.key <= w.name);
        for e in &events[lo..hi] {
            match e.kind {
                crate::kind::EventKind::Use => w.uses.push(e.node),
                crate::kind::EventKind::Shadow => w.shadowed = true,
                crate::kind::EventKind::Write => {
                    for (wn, hit) in w.watch.iter().zip(w.writes_hit.iter_mut()) {
                        if *wn == w.name {
                            *hit = true;
                        }
                    }
                }
                // 防御：索引层保证含 Raw 的语句不建索引（键排序分区看不见
                // Opaque 键），此处理论上不可达
                crate::kind::EventKind::Opaque => w.opaque = true,
            }
        }
        // 其余 watch 键区间：只找 Write
        for (i, wk) in w.watch.iter().enumerate() {
            if *wk == w.name {
                continue; // 已随主名字区间处理
            }
            let lo = events.partition_point(|e| e.key < *wk);
            let hi = events.partition_point(|e| e.key <= *wk);
            for e in &events[lo..hi] {
                if e.kind == crate::kind::EventKind::Write {
                    w.writes_hit[i] = true;
                    break; // 区间内任一 Write 即命中
                }
            }
        }
        return;
    }
    // 递归路径也查不透明（索引被失效后落到此处的防御）
    if lang.is_opaque(node) {
        w.opaque = true;
        return;
    }
    // 无索引语言：原递归遍历（事件序一致；键比较同上）
    match lang.kind(node) {
        NodeKind::Assign => {
            let ch = lang.children(node);
            if let Some(&t) = ch.first() {
                if lang.kind(t) == NodeKind::VarRef {
                    if let Some(k) = lang.var_key(t) {
                        for (wn, hit) in w.watch.iter().zip(w.writes_hit.iter_mut()) {
                            if *wn == k {
                                *hit = true;
                            }
                        }
                    }
                }
            }
            if let Some(&v) = ch.get(1) {
                scan_region(lang, v, w);
            }
        }
        NodeKind::Unary if lang.un_op(node).is_some_and(|o| o.is_incdec()) => {
            let ch = lang.children(node);
            if let Some(&t) = ch.first() {
                if lang.kind(t) == NodeKind::VarRef {
                    if let Some(k) = lang.var_key(t) {
                        for (wn, hit) in w.watch.iter().zip(w.writes_hit.iter_mut()) {
                            if *wn == k {
                                *hit = true;
                            }
                        }
                    }
                }
            }
        }
        NodeKind::VarDecl => {
            if let Some(k) = lang.var_key(node) {
                if k == w.name {
                    w.shadowed = true;
                }
                for (wn, hit) in w.watch.iter().zip(w.writes_hit.iter_mut()) {
                    if *wn == k {
                        *hit = true;
                    }
                }
            }
            for &c in lang.children(node) {
                scan_region(lang, c, w);
            }
        }
        // ForEach 循环变量 / Catch 绑定：在子作用域声明了同名变量 → 视为遮蔽；
        // ForEach 每轮给循环变量赋值 → 也算显式写。
        NodeKind::ForEach | NodeKind::Catch => {
            if let Some(k) = lang.var_key(node) {
                if k == w.name {
                    w.shadowed = true;
                }
                if lang.kind(node) == NodeKind::ForEach {
                    for (wn, hit) in w.watch.iter().zip(w.writes_hit.iter_mut()) {
                        if *wn == k {
                            *hit = true;
                        }
                    }
                }
            }
            for &c in lang.children(node) {
                scan_region(lang, c, w);
            }
        }
        NodeKind::VarRef => {
            if lang.var_key(node) == Some(w.name) {
                w.uses.push(node);
            }
        }
        NodeKind::Raw => {
            // 不可解析原文：读/写集未知
            w.opaque = true;
        }
        _ => {
            for &c in lang.children(node) {
                scan_region(lang, c, w);
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Binary]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Unary]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Ternary]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Ternary]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Binary]
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
        // 字符串拼接（Str 与 Str/Char/Int/Long/Bool 字面量——拼接的隐式
        // valueOf 对这些基元是确定性的；浮点除外：Double.toString 算法与
        // Rust Display 不保证逐位一致）
        if op == BinOp::Add {
            let lit_str_of = |n: L::Id| -> Option<String> {
                match lang.literal(n)? {
                    LitRef::Str(s) => Some(s.to_string()),
                    LitRef::Char(c) => Some(c.to_string()),
                    LitRef::Int(v) => Some(v.to_string()),
                    LitRef::Long(v) => Some(v.to_string()),
                    LitRef::Bool(b) => Some(b.to_string()),
                    _ => None,
                }
            };
            if let (Some(a), Some(b)) = (lit_str_of(l), lit_str_of(r)) {
                if matches!(lang.literal(l), Some(LitRef::Str(_)))
                    || matches!(lang.literal(r), Some(LitRef::Str(_)))
                {
                    let joined = format!("{a}{b}");
                    let with = lang.build_str(&joined);
                    return Some(Edit::Replace { target: id, with });
                }
            }
        }
        let int_of = |x: Option<LitRef<'_>>| -> Option<i64> {
            match x? {
                // Java 中 char 参与算术时提升为 int
                LitRef::Int(v) | LitRef::Long(v) => Some(v),
                LitRef::Char(c) => Some(c as u32 as i64),
                _ => None,
            }
        };
        let (a, b) = (int_of(lang.literal(l))?, int_of(lang.literal(r))?);
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Binary]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::VarDecl, NodeKind::Assign, NodeKind::ExprStmt]
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

        // 归一化第二条语句：Assign / ExprStmt{Assign}（简单赋值）；
        // 不是赋值 → 落入「零用途死存储」形态（见下）
        let second = match lang.kind(next_stmt) {
            NodeKind::Assign => Some(next_stmt),
            NodeKind::ExprStmt => {
                let ch = lang.children(next_stmt);
                if ch.len() == 1 && lang.kind(ch[0]) == NodeKind::Assign {
                    Some(ch[0])
                } else {
                    None
                }
            }
            _ => None,
        };
        let Some(second) = second else {
            // 零用途死存储：后继对该变量**无读且无写**（含遮蔽重声明），
            // 存入的值永不流出 → 整条删除。值效果 ≤ MayRead 才可丢弃。
            // （批量化后可达的形态：传播先消费了击杀写，init 残留为死值；
            //   旧逐编辑时序下由相邻对形态逐步吸收，未暴露此缺口。）
            // **作用域前提**：名字必须声明于本块（块级作用域，出块不可见）。
            // 外层声明的名字可能在本块之后被外层代码读取（if/while 分支里
            // 对外层变量的赋值就是典型）→ 扫描只覆盖本块尾部，必须拒绝。
            // （property.rs seed=29 的随机程序抓获此漏洞。）
            let (name_node, target_local) = if decl_form {
                (id, true)
            } else {
                let t = *lang.children(first_assign?).first()?;
                if lang.kind(t) != NodeKind::VarRef {
                    return None;
                }
                (t, lang.is_local_var(t))
            };
            // 赋值形态：目标必须是本局部（声明形态天然是局部）
            if !target_local {
                return None;
            }
            let name = lang.var_name(name_node)?.to_string();
            let declared_in_this_block = if decl_form {
                true // 检查对象就是本块的声明
            } else if parent == walk.root {
                // 方法体根块：尾部扫描覆盖到方法末尾——参数/局部变量的
                // 零后续事件即可判定为死（方法外不可见）
                true
            } else {
                // 嵌套块：名字必须声明于本块（外层声明的名字可能被
                // 本块之后的外层代码读取）
                stmts[..idx].iter().any(|&s| {
                    lang.kind(s) == NodeKind::VarDecl && lang.var_name(s) == Some(name.as_str())
                })
            };
            if !declared_in_this_block {
                return None;
            }
            let name_key0 = lang.var_key(name_node)?;
            let mut w0 = Watch::<L>::new(name_key0, &[name_key0]);
            for &s in &stmts[idx + 1..] {
                scan_region(&*lang, s, &mut w0);
            }
            // 只判**本名字**的读/写（watch 集即本名字）；Raw 区域不可证明
            if w0.opaque || !w0.uses.is_empty() || w0.wrote(name_key0) || w0.shadowed {
                return None;
            }
            if lang.effect(first_value) > Effect::MayRead {
                return None;
            }
            return Some(Edit::Delete { node: id });
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

/// 块展平：Block 的直接孩子是 Block 且**内层无任何声明**（VarDecl/
/// ForEach/Catch 绑定引入作用域）→ 内层语句上提。典型形态：try 剥壳、
/// CFF 还原后的裸嵌套块。无声明 ⇒ 无遮蔽 ⇒ 语义等价。
pub struct BlockFlatten;

impl<L: Lang> Rule<L> for BlockFlatten {
    fn name(&self) -> &'static str {
        "block_flatten"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Block]
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk } = ctx;
        let parent = walk.parent(id)?;
        if lang.kind(parent) != NodeKind::Block {
            return None;
        }
        // 内层不得有任何声明（作用域引入）
        let inner = lang.children(id).to_vec();
        let declares = |n: L::Id| -> bool {
            matches!(
                lang.kind(n),
                NodeKind::VarDecl | NodeKind::ForEach | NodeKind::Catch
            )
        };
        if inner.iter().any(|&s| declares(s)) {
            return None;
        }
        let idx = walk.index(id)?;
        Some(Edit::Splice {
            node: parent,
            index: idx,
            remove: 1,
            insert: inner,
        })
    }
}

/// 数值双重取负：-(-x) → x（补码回绕下恒等：-(-MIN) ≡ MIN ≡ x）；
/// -(-lit) → |lit|（解析器不折负字面量的外层负）。
pub struct DoubleNegFold;

impl<L: Lang> Rule<L> for DoubleNegFold {
    fn name(&self) -> &'static str {
        "double_neg_fold"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Unary]
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, .. } = ctx;
        if lang.un_op(id) != Some(UnOp::Neg) {
            return None;
        }
        let inner = *lang.children(id).first()?;
        if lang.un_op(inner) == Some(UnOp::Neg) {
            let x = *lang.children(inner).first()?;
            return Some(Edit::Replace {
                target: id,
                with: x,
            });
        }
        // -(-lit)：负数值字面量取外层负 → 正字面量
        if let Some(v) = lang.literal(inner).and_then(|l| match l {
            LitRef::Int(v) => Some((v, false)),
            LitRef::Long(v) => Some((v, true)),
            _ => None,
        }) {
            if v.0 < 0 {
                let with = if v.1 {
                    lang.build_int(-v.0, true)
                } else {
                    lang.build_int(-v.0, false)
                };
                return Some(Edit::Replace {
                    target: id,
                    with,
                });
            }
        }
        None
    }
}

pub struct UnreachableAfterTerminal;

impl<L: Lang> Rule<L> for UnreachableAfterTerminal {
    fn name(&self) -> &'static str {
        "unreachable_after_terminal"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Block]
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
// 比较常量折叠（反混淆核心）：1 < 2 → true、2 == 3 → false。
// 仅整数（含 long）与布尔字面量；浮点（NaN 语义）与字符串（引用比较）不折。
// 连锁：boolean b = 2 > 1; if (b) {...} → 不透明谓词被完全击穿。
// ---------------------------------------------------------------------------

pub struct CmpConstFold;

impl<L: Lang> Rule<L> for CmpConstFold {
    fn name(&self) -> &'static str {
        "cmp_const_fold"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Binary]
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Binary {
            return None;
        }
        let op = lang.bin_op(id)?;
        if !op.is_comparison() {
            return None;
        }
        let ch = lang.children(id);
        let (l, r) = (*ch.first()?, *ch.get(1)?);
        let int_of = |x: Option<LitRef<'_>>| -> Option<i64> {
            match x? {
                LitRef::Int(v) | LitRef::Long(v) => Some(v),
                LitRef::Char(c) => Some(c as u32 as i64),
                _ => None,
            }
        };
        use LitRef::*;
        let result: Option<bool> = match (int_of(lang.literal(l)), int_of(lang.literal(r))) {
            (Some(x), Some(y)) => cmp_i64(op, x, y),
            _ => {
                if let (Some(Bool(x)), Some(Bool(y))) = (lang.literal(l), lang.literal(r)) {
                    if matches!(op, BinOp::Eq | BinOp::Ne) {
                        Some(if op == BinOp::Eq { x == y } else { x != y })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };
        let v = result?;
        let with = lang.build_bool(v);
        Some(Edit::Replace {
            target: id,
            with,
        })
    }
}

fn cmp_i64(op: BinOp, x: i64, y: i64) -> Option<bool> {
    use BinOp::*;
    Some(match op {
        Lt => x < y,
        Le => x <= y,
        Gt => x > y,
        Ge => x >= y,
        Eq => x == y,
        Ne => x != y,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// 位运算恒等式（混淆器高频产物）：
//   x ^ 0 → x、0 ^ x → x、x | 0 → x、0 | x → x、x & -1 → x、-1 & x → x（x 保留，恒安全）
//   x | -1 → -1、x & 0 → 0、0 & x → 0（x 求值被丢弃 → effect ≤ MayRead）
//   x ^ x → 0（同名局部整型变量）
// ---------------------------------------------------------------------------

pub struct BitIdentity;

impl<L: Lang> Rule<L> for BitIdentity {
    fn name(&self) -> &'static str {
        "bit_identity"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Binary]
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Binary {
            return None;
        }
        let op = lang.bin_op(id)?;
        if !matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor) {
            return None;
        }
        let ch = lang.children(id);
        let (l, r) = (*ch.first()?, *ch.get(1)?);
        let int_lit = |n: L::Id| lang.literal(n).and_then(|x| x.as_int());

        // x ^ x → 0（同名局部变量）
        if op == BinOp::BitXor
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

        // 字面量在任一侧（互换尝试）
        for (x, k) in [(l, r), (r, l)] {
            let Some(kv) = int_lit(k) else {
                continue;
            };
            if !lang.is_exact_int(x) {
                continue;
            }
            let identity = match (op, kv) {
                (BinOp::BitXor, 0) => Some(x),          // x ^ 0 → x
                (BinOp::BitOr, 0) => Some(x),           // x | 0 → x
                (BinOp::BitAnd, -1) => Some(x),         // x & -1 → x
                (BinOp::BitAnd, 0) | (BinOp::BitOr, -1) => {
                    // x & 0 → 0 / x | -1 → -1：丢弃 x 求值，需可丢弃
                    if lang.effect(x) > Effect::MayRead {
                        None
                    } else {
                        Some(k)
                    }
                }
                _ => None,
            };
            if let Some(with) = identity {
                return Some(Edit::Replace {
                    target: id,
                    with,
                });
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// 算术/异或重结合（混淆器经典）：(x + K1) - K2 → x ± K、(x ^ K1) ^ K2 → x ^ K。
// 整数回绕算术下恒安全；K == 0 时直接得 x（(x + 5) - 5 → x、(x ^ 84) ^ 84 → x）。
// ---------------------------------------------------------------------------

pub struct ArithReassoc;

impl<L: Lang> Rule<L> for ArithReassoc {
    fn name(&self) -> &'static str {
        "arith_reassoc"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Binary]
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::Binary {
            return None;
        }
        let op2 = lang.bin_op(id)?;
        if !matches!(op2, BinOp::Add | BinOp::Sub | BinOp::BitXor) {
            return None;
        }
        let och = lang.children(id).to_vec();
        let inner = och[0];
        if lang.kind(inner) != NodeKind::Binary {
            return None;
        }
        let op1 = lang.bin_op(inner)?;
        let ich = lang.children(inner).to_vec();
        let x = ich[0];

        // 字符串拼接重结合：(x + "K1") + "K2" → x + "K1K2"
        // （拼接满足结合律且各操作数恰按序求值一次；x 任意类型）。
        // **整链折叠**：左倾链 (((x+"a")+"a")+"a")… 一刀折成 x+"aaa…"——
        // 逐层折每 pass 一层是 O(n²)（openjdk DeepStringConcat 32001 项
        // 78s/32k passes 抓获；整链后 1 编辑收敛）
        if op1 == BinOp::Add && op2 == BinOp::Add {
            if let (Some(LitRef::Str(k1)), Some(LitRef::Str(k2))) =
                (lang.literal(ich[1]), lang.literal(och[1]))
            {
                // 收集左倾链（外→内），要求每层 (child + "str") 同构
                let mut chain: Vec<L::Id> = vec![id];
                let mut cur = id;
                loop {
                    let cch = lang.children(cur).to_vec();
                    let next = cch[0];
                    let deeper = lang.kind(next) == NodeKind::Binary
                        && lang.bin_op(next) == Some(BinOp::Add)
                        && matches!(lang.literal(lang.children(next).to_vec()[1]),
                            Some(LitRef::Str(_)));
                    if deeper {
                        chain.push(next);
                        cur = next;
                    } else {
                        break;
                    }
                }
                // base = 最内层的左操作数；parts 内→外 = 源顺序
                let innermost = *chain.last().unwrap();
                let ich2 = lang.children(innermost).to_vec();
                let base = ich2[0];
                let mut joined = String::new();
                for &n in chain.iter().rev() {
                    if let Some(LitRef::Str(k)) =
                        lang.literal(lang.children(n).to_vec()[1])
                    {
                        joined.push_str(k);
                    }
                }
                let _ = (k1, k2); // 已并入 joined
                let lit = lang.build_str(&joined);
                let with = lang.build_bin(BinOp::Add, base, lit);
                return Some(Edit::Replace {
                    target: id,
                    with,
                });
            }
        }

        // 整数重结合：(x ± K1) ± K2 → x ± K、(x ^ K1) ^ K2 → x ^ K
        let k1 = lang.literal(ich[1]).and_then(|x| x.as_int())?;
        let k2 = lang.literal(och[1]).and_then(|x| x.as_int())?;
        if !lang.is_exact_int(x) {
            return None;
        }
        let is_long = matches!(lang.literal(ich[1]), Some(LitRef::Long(_)))
            || matches!(lang.literal(och[1]), Some(LitRef::Long(_)));
        let sign = |o: BinOp| if o == BinOp::Sub { -1i64 } else { 1i64 };
        let delta: Option<i64> = match (op1, op2) {
            (BinOp::BitXor, BinOp::BitXor) => Some(k1 ^ k2),
            (a, b) if matches!(a, BinOp::Add | BinOp::Sub)
                && matches!(b, BinOp::Add | BinOp::Sub) =>
            {
                Some(sign(a) * k1 + sign(b) * k2)
            }
            _ => None,
        };
        let d = delta?;
        let with = if d == 0 {
            x
        } else if op1 == BinOp::BitXor && op2 == BinOp::BitXor {
            let lit = lang.build_int(d, is_long);
            lang.build_bin(BinOp::BitXor, x, lit)
        } else if d > 0 {
            let lit = lang.build_int(d, is_long);
            lang.build_bin(BinOp::Add, x, lit)
        } else {
            let lit = lang.build_int(-d, is_long);
            lang.build_bin(BinOp::Sub, x, lit)
        };
        Some(Edit::Replace {
            target: id,
            with,
        })
    }
}

// ---------------------------------------------------------------------------
// if→三元归并（反编译器/混淆器经典形态）：
//   if (c) return a; else return b;    →  return c ? a : b;
//   if (c) return a; return b;（收尾） →  return c ? a : b;
// 恒安全：c 单次求值、a/b 条件求值在两种形态下一致（三元是惰性的）。
// 布尔特例由 ternary_bool 继续收敛成 return c / return !c。
// ---------------------------------------------------------------------------

pub struct IfToTernary;

impl<L: Lang> Rule<L> for IfToTernary {
    fn name(&self) -> &'static str {
        "if_to_ternary"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::If]
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk } = ctx;
        if lang.kind(id) != NodeKind::If {
            return None;
        }
        let ch = lang.children(id).to_vec();
        let c = ch[0];

        // 提取"单 return 语句"分支：Block{Return} 或裸 Return
        let single_ret = |n: L::Id| -> Option<Option<L::Id>> {
            match lang.kind(n) {
                NodeKind::Return => Some(lang.children(n).first().copied()),
                NodeKind::Block if lang.children(n).len() == 1 => {
                    let inner = lang.children(n)[0];
                    if lang.kind(inner) == NodeKind::Return {
                        Some(lang.children(inner).first().copied())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        };

        if ch.len() == 3 {
            // 完整 if-else：两支都是单 return
            let a = single_ret(ch[1])?;
            let b = single_ret(ch[2])?;
            let with = match (a, b) {
                (Some(va), Some(vb)) => {
                    let t = lang.build_ternary(c, va, vb);
                    lang.build_return(Some(t))
                }
                (None, None) => lang.build_return(None),
                _ => return None,
            };
            return Some(Edit::Replace {
                target: id,
                with,
            });
        }

        // if-then + 紧随的收尾 return（必须是父块最后一条语句）
        if ch.len() != 2 {
            return None;
        }
        let a = single_ret(ch[1])?;
        let parent = walk.parent(id)?;
        if lang.kind(parent) != NodeKind::Block {
            return None;
        }
        let idx = walk.index(id)?;
        let stmts = lang.children(parent).to_vec();
        if idx + 2 != stmts.len() {
            return None; // return 必须是最后一条
        }
        let tail = stmts[idx + 1];
        if lang.kind(tail) != NodeKind::Return {
            return None;
        }
        let b = lang.children(tail).first().copied();
        let with = match (a, b) {
            (Some(va), Some(vb)) => {
                let t = lang.build_ternary(c, va, vb);
                lang.build_return(Some(t))
            }
            (None, None) => lang.build_return(None),
            _ => return None,
        };
        Some(Edit::Splice {
            node: parent,
            index: idx,
            remove: 2,
            insert: vec![with],
        })
    }
}

// ---------------------------------------------------------------------------
// if→三元赋值归并：
//   if (c) { x = a; } else { x = b; }  →  x = c ? a : b;
// 恒安全（x 为变量名，两种形态各求值一次；a/b 条件求值一致）。
// ---------------------------------------------------------------------------

pub struct IfAssignTernary;

impl<L: Lang> Rule<L> for IfAssignTernary {
    fn name(&self) -> &'static str {
        "if_assign_ternary"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::If]
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        if lang.kind(id) != NodeKind::If {
            return None;
        }
        let ch = lang.children(id).to_vec();
        if ch.len() != 3 {
            return None;
        }
        let c = ch[0];
        // 单赋值分支：Block{Assign} / Block{ExprStmt{Assign}} / 裸（ExprStmt 包裹的）Assign
        let single_assign = |n: L::Id| -> Option<(L::Id, L::Id)> {
            let unwrap = |i: L::Id| -> L::Id {
                if lang.kind(i) == NodeKind::ExprStmt {
                    let ch = lang.children(i);
                    if ch.len() == 1 && lang.kind(ch[0]) == NodeKind::Assign {
                        return ch[0];
                    }
                }
                i
            };
            let inner = match lang.kind(n) {
                NodeKind::Assign => n,
                NodeKind::ExprStmt => unwrap(n),
                NodeKind::Block if lang.children(n).len() == 1 => unwrap(lang.children(n)[0]),
                _ => return None,
            };
            if lang.kind(inner) != NodeKind::Assign {
                return None;
            }
            if lang.assign_op(inner).is_some() {
                return None;
            }
            let a = lang.children(inner).to_vec();
            if lang.kind(a[0]) != NodeKind::VarRef {
                return None;
            }
            Some((a[0], a[1]))
        };
        let (t1, va) = single_assign(ch[1])?;
        let (t2, vb) = single_assign(ch[2])?;
        if lang.var_name(t1) != lang.var_name(t2) || lang.var_name(t1).is_none() {
            return None;
        }
        let t = lang.build_ternary(c, va, vb);
        let with = lang.build_assign(t1, t);
        Some(Edit::Replace {
            target: id,
            with,
        })
    }
}


// ---------------------------------------------------------------------------
// 声明-赋值合并（反编译器把声明与赋值拆开的形态）：
//   int x; x = 5;  →  int x = 5;
// 守卫：相邻同块、简单赋值、value 不引用 x（依赖旧值则非法）。
// 之后 local_propagation 可继续把 init 内联到唯一使用处。
// ---------------------------------------------------------------------------

pub struct DeclAssignMerge;

impl<L: Lang> Rule<L> for DeclAssignMerge {
    fn name(&self) -> &'static str {
        "decl_assign_merge"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::VarDecl]
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk } = ctx;
        if lang.kind(id) != NodeKind::VarDecl || !lang.children(id).is_empty() {
            return None; // 必须是无 init 声明
        }
        let name = lang.var_name(id)?.to_string();
        let parent = walk.parent(id)?;
        if lang.kind(parent) != NodeKind::Block {
            return None;
        }
        let idx = walk.index(id)?;
        let stmts = lang.children(parent).to_vec();
        let next = *stmts.get(idx + 1)?;
        // 下一条：Assign(x, v) / ExprStmt{Assign(x, v)}（简单赋值）
        let assign = match lang.kind(next) {
            NodeKind::Assign => next,
            NodeKind::ExprStmt => {
                let ch = lang.children(next);
                if ch.len() == 1 && lang.kind(ch[0]) == NodeKind::Assign {
                    ch[0]
                } else {
                    return None;
                }
            }
            _ => return None,
        };
        if lang.assign_op(assign).is_some() {
            return None;
        }
        let ach = lang.children(assign).to_vec();
        let (target, value) = (ach[0], ach[1]);
        if lang.kind(target) != NodeKind::VarRef || lang.var_name(target) != Some(name.as_str()) {
            return None;
        }
        // value 不得引用 x（读旧值）
        if subtree_contains(&*lang, value, |n| {
            lang.kind(n) == NodeKind::VarRef && lang.var_name(n) == Some(name.as_str())
        }) {
            return None;
        }
        Some(Edit::Multi(vec![
            Edit::Splice {
                node: id,
                index: 0,
                remove: 0,
                insert: vec![value],
            },
            Edit::Delete { node: next },
        ]))
    }
}

// ---------------------------------------------------------------------------
// 赋值传播（寄存器拷贝消除，jadx/jcdc 产物）：
//   x = v; …唯一一次读 x（区间内无对 x / v 读集的写、无遮蔽）…
//   → 用 v 替换该次读，删除赋值。
// 安全锚点：x 必须在本块内有前置声明（赋值无作用域，块外读取会破坏语义）。
// v 纯/只读 → 自由移动（写冲突检查）；v 有副作用 → 相邻 + 直线 + 可读前缀。
// ---------------------------------------------------------------------------

pub struct AssignPropagation;

impl<L: Lang> Rule<L> for AssignPropagation {
    fn name(&self) -> &'static str {
        "assign_propagation"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Assign, NodeKind::ExprStmt]
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk } = ctx;
        // 语句形态：Assign / ExprStmt{Assign}
        let (assign, stmt_node) = match lang.kind(id) {
            NodeKind::Assign => (id, id),
            NodeKind::ExprStmt => {
                let ch = lang.children(id);
                if ch.len() == 1 && lang.kind(ch[0]) == NodeKind::Assign {
                    (ch[0], id)
                } else {
                    return None;
                }
            }
            _ => return None,
        };
        if lang.assign_op(assign).is_some() {
            return None;
        }
        let ach = lang.children(assign).to_vec();
        let (target, value) = (ach[0], ach[1]);
        if lang.kind(target) != NodeKind::VarRef || !lang.is_local_var(target) {
            return None;
        }
        // 裸数组字面量只在声明/赋值 RHS 位置合法，传播会产出非法源码 → 拒绝
        if lang.kind(value) == NodeKind::ArrayLit {
            return None;
        }
        // lambda / 方法引用值同 ArrayLit：receiver 位非法 → 保守拒绝
        if matches!(lang.kind(value), NodeKind::Lambda | NodeKind::MethodRef) {
            return None;
        }
        let name = lang.var_name(target)?.to_string();

        let parent = walk.parent(stmt_node)?;
        if lang.kind(parent) != NodeKind::Block {
            return None;
        }
        let idx = walk.index(stmt_node)?;
        let stmts = lang.children(parent).to_vec();

        // 锚点：x 在本块 idx 之前有 VarDecl（保证 x 不会逃逸到块外）
        let anchored = stmts[..idx].iter().any(|&s| {
            lang.kind(s) == NodeKind::VarDecl && lang.var_name(s) == Some(name.as_str())
        });
        if !anchored {
            return None;
        }

        // value 不引用 x（x = x + 1 之类）
        if subtree_contains(&*lang, value, |n| {
            lang.kind(n) == NodeKind::VarRef && lang.var_name(n) == Some(name.as_str())
        }) {
            return None;
        }

        // 扫描赋值之后的区域（用途/遮蔽全区间；写冲突窗口见下）
        let name_key = lang.var_key(target)?;
        let mut wa = Watch::<L>::new(name_key, &[]);
        for &s in &stmts[idx + 1..] {
            scan_region(&*lang, s, &mut wa);
        }
        if wa.opaque || wa.shadowed || wa.uses.len() != 1 {
            return None;
        }
        let use_id = wa.uses[0];
        // 写冲突兴趣集：value 读到的名字键 + x 自身（自身被写 → 覆盖，拒绝）
        let mut watch_keys: Vec<L::NameKey> = Vec::new();
        collect_read_keys(&*lang, value, &mut watch_keys);
        watch_keys.push(name_key);
        let mut wb = Watch::<L>::new(name_key, &watch_keys);
        for &s in &stmts[idx + 1..] {
            let uses_before = wb.uses.len();
            scan_region(&*lang, s, &mut wb);
            if uses_before < wb.uses.len() {
                break; // 使用语句之后的写不参与冲突判定
            }
        }
        // x 自身在窗口内被写 → 赋值会被覆盖，拒绝（先于目标写排除）
        if wb.wrote(name_key) {
            return None;
        }
        let ve = lang.effect(value);

        if ve <= Effect::MayRead {
            if let Some(target_key) = use_stmt_target_write_key(&*lang, &stmts[idx + 1..], use_id) {
                wb.clear_wrote(target_key);
            }
            if wb.any_write() {
                return None;
            }
            return Some(Edit::Multi(vec![
                Edit::Replace {
                    target: use_id,
                    with: value,
                },
                Edit::Delete { node: stmt_node },
            ]));
        }

        // 有副作用：相邻 + 直线 + 可读前缀（与 local_propagation 同判据）
        if idx + 1 >= stmts.len() {
            return None;
        }
        let stmt = stmts[idx + 1];
        if !subtree_contains(&*lang, stmt, |n| n == use_id) {
            return None;
        }
        match lang.kind(stmt) {
            NodeKind::Return
            | NodeKind::ExprStmt
            | NodeKind::Assign
            | NodeKind::Throw
            | NodeKind::VarDecl => {}
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
            Edit::Delete { node: stmt_node },
        ]))
    }
}


/// 单一使用点若恰为 `target = use` 的 RHS，则该语句对 target 的写发生在
/// use 求值**之后**——从写冲突集中排除（否则 `int v = x + s; x = v;` 这类
/// 循环尾寄存器回拷永远无法内联）。
/// 排除目标必须≠被传播变量自身：`int i = 1; i = i;` 里排除 `i` 会删掉声明
/// 留下无绑定的赋值（该场景由 self_assign 处理）。
fn use_stmt_target_write_key<L: Lang>(
    lang: &L,
    stmts_after: &[L::Id],
    use_id: L::Id,
) -> Option<L::NameKey> {
    for &s in stmts_after {
        if subtree_contains(lang, s, |n| n == use_id) {
            let assign = match lang.kind(s) {
                NodeKind::Assign => s,
                NodeKind::ExprStmt => {
                    let ch = lang.children(s);
                    if ch.len() == 1 && lang.kind(ch[0]) == NodeKind::Assign {
                        ch[0]
                    } else {
                        return None;
                    }
                }
                _ => return None,
            };
            if lang.assign_op(assign).is_some() {
                return None;
            }
            let ch = lang.children(assign);
            if ch.len() == 2 && ch[1] == use_id && lang.kind(ch[0]) == NodeKind::VarRef {
                return lang.var_key(ch[0]);
            }
            return None;
        }
    }
    None
}

/// 收集 `id` 子树中 VarRef 读到的全部名字键（去重，线性）。
/// 替代旧 reads_vars 的 HashSet 版：value 的读名字典型 ≤10，线性查重更快。
fn collect_read_keys<L: Lang>(lang: &L, id: L::Id, out: &mut Vec<L::NameKey>) {
    let mut stack = vec![id];
    while let Some(n) = stack.pop() {
        if lang.kind(n) == NodeKind::VarRef {
            if let Some(k) = lang.var_key(n) {
                if !out.contains(&k) {
                    out.push(k);
                }
            }
        }
        for &c in lang.children(n) {
            stack.push(c);
        }
    }
}


// ---------------------------------------------------------------------------
// 尾部 continue 删除（反编译器产物）：
//   while (c) { …; continue; }  →  while (c) { … }
// 循环体最后一条无标签 continue 与顺序落入下一轮等价。恒安全。
// ---------------------------------------------------------------------------

pub struct TrailingContinue;

impl<L: Lang> Rule<L> for TrailingContinue {
    fn name(&self) -> &'static str {
        "trailing_continue"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::While, NodeKind::For, NodeKind::DoWhile, NodeKind::ForEach]
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk: _ } = ctx;
        // While/For/DoWhile/ForEach 的 body 为 Block 且末语句为无标签 continue
        let body = match lang.kind(id) {
            NodeKind::While | NodeKind::For | NodeKind::DoWhile | NodeKind::ForEach => {
                *lang.children(id).last()?
            }
            _ => return None,
        };
        if lang.kind(body) != NodeKind::Block {
            return None;
        }
        let ch = lang.children(body).to_vec();
        let last = *ch.last()?;
        if lang.kind(last) != NodeKind::Continue || !lang.children(last).is_empty() {
            // JavaAST 的 Continue 标签在负载里，children 为空即无标签（引擎层约定）
            // 对带标签的语言实现：children[last] 非空即有标签 → 不动
            if lang.kind(last) != NodeKind::Continue {
                return None;
            }
        }
        Some(Edit::Delete { node: last })
    }
}

// ---------------------------------------------------------------------------
// 多用途拷贝传播（寄存器副本消除）：
//   x = y;（y 为简单 VarRef，x 有本块声明锚点，之后无对 x/y 的写、无遮蔽）
//   → 把 x 的**所有**后续读替换为 y，删除赋值。
// 守卫 y 不被写：替换后各使用点读的是 y 的当前值，须与原 x 的值一致。
// ---------------------------------------------------------------------------

pub struct MultiUseCopyPropagation;

impl<L: Lang> Rule<L> for MultiUseCopyPropagation {
    fn name(&self) -> &'static str {
        "multi_use_copy"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Assign, NodeKind::ExprStmt]
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk } = ctx;
        // 语句形态：Assign(x, VarRef y) / ExprStmt{Assign(...)}
        let (assign, stmt_node) = match lang.kind(id) {
            NodeKind::Assign => (id, id),
            NodeKind::ExprStmt => {
                let ch = lang.children(id);
                if ch.len() == 1 && lang.kind(ch[0]) == NodeKind::Assign {
                    (ch[0], id)
                } else {
                    return None;
                }
            }
            _ => return None,
        };
        if lang.assign_op(assign).is_some() {
            return None;
        }
        let ach = lang.children(assign).to_vec();
        let (target, value) = (ach[0], ach[1]);
        if lang.kind(target) != NodeKind::VarRef || lang.kind(value) != NodeKind::VarRef {
            return None;
        }
        // lambda / 方法引用值：内联进 receiver 位非法（对抗波 2）→ 保守拒绝
        if matches!(lang.kind(value), NodeKind::Lambda | NodeKind::MethodRef) {
            return None;
        }
        if !lang.is_local_var(target) {
            return None;
        }
        let name = lang.var_name(target)?.to_string();
        let src = lang.var_name(value)?.to_string();
        if name == src {
            return None;
        }

        let parent = walk.parent(stmt_node)?;
        if lang.kind(parent) != NodeKind::Block {
            return None;
        }
        let idx = walk.index(stmt_node)?;
        let stmts = lang.children(parent).to_vec();

        // 锚点：x 在本块 idx 之前有 VarDecl
        let anchored = stmts[..idx].iter().any(|&s| {
            lang.kind(s) == NodeKind::VarDecl && lang.var_name(s) == Some(name.as_str())
        });
        if !anchored {
            return None;
        }

        // 扫描后续：收集 x 的全部读；对 x 的任何写 / 对 y 的任何写 / 遮蔽 → 拒绝
        let name_key = lang.var_key(target)?;
        let src_key = lang.var_key(value)?;
        let mut w1 = Watch::<L>::new(name_key, &[name_key, src_key]);
        for &s in &stmts[idx + 1..] {
            scan_region(&*lang, s, &mut w1);
        }
        if w1.opaque || w1.shadowed || w1.uses.is_empty() {
            return None;
        }
        if w1.wrote(name_key) || w1.wrote(src_key) {
            return None;
        }
        // y 也不得在【赋值前】与 x 指向不同值（x=y 之前 y 已是其所值，无需检查）；
        // 但 x=y 之间不能有对 y 的写（相邻语句，天然无中间）——赋值本身就是当前值 ✓

        let mut edits: Vec<Edit<L>> = w1.uses
            .iter()
            .map(|&u| Edit::Replace {
                target: u,
                with: value,
            })
            .collect();
        edits.push(Edit::Delete { node: stmt_node });
        Some(Edit::Multi(edits))
    }
}


// ---------------------------------------------------------------------------
// 尾部裸 return 删除（DAD/androguard 伪影）：
//   void 方法/构造器体最后一条无值 return 与自然结束等价，删除。
// 仅匹配根块（方法体）末语句，避免动到块中间的提前 return。
// ---------------------------------------------------------------------------

pub struct TrailingReturn;

impl<L: Lang> Rule<L> for TrailingReturn {
    fn name(&self) -> &'static str {
        "trailing_return"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Return]
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let root = ctx.root();
        let parent = ctx.parent(id)?;
        let lang = ctx.lang;
        if lang.kind(id) != NodeKind::Return || !lang.children(id).is_empty() {
            return None;
        }
        if parent != root || lang.kind(parent) != NodeKind::Block {
            return None;
        }
        if lang.children(parent).last() != Some(&id) {
            return None;
        }
        Some(Edit::Delete { node: id })
    }
}


// ---------------------------------------------------------------------------
// 三元-布尔运算归并（反混淆产物）：
//   c ? a : false → c && a、c ? true : a → c || a（a 为布尔）
//   c 单次求值、a 条件求值，两种形态完全一致。
// ---------------------------------------------------------------------------

pub struct TernaryBoolOp;

impl<L: Lang> Rule<L> for TernaryBoolOp {
    fn name(&self) -> &'static str {
        "ternary_bool_op"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Ternary]
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
        let with = if is_false(b) && lang.is_bool(a) {
            lang.build_bin(BinOp::And, c, a)
        } else if lb(b) && lang.is_bool(a) {
            // c ? a : true ≡ !c || a —— 少见，跳过保持保守
            return None;
        } else if lb(a) && lang.is_bool(b) {
            // c ? true : b ≡ c || b
            lang.build_bin(BinOp::Or, c, b)
        } else if is_false(a) && lang.is_bool(b) {
            // c ? false : b ≡ !c && b —— 少见，跳过保持保守
            return None;
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
// 远距存储消除（寄存器预声明清理，ddc/jcdc 产物）：
//   int x = 0; …（无 x 事件）…; x = 1;   →  int x = 1;（字面量提升）
//   int x = 0; …（无 x 事件）…; x = v;   →  int x;（init 死亡，剥除）
//   x = 0; …（无 x 事件）…; x = v;       →  删除第一条（值 ≤ MayRead）
// 守卫：首个 x 事件必须是写；语句内 x 出现若非简单赋值目标即视为读（阻断）。
// ---------------------------------------------------------------------------

pub struct StoreKill;

/// 语句级事件分类：任意路径的读都保活 init；只有**支配写**才击杀。
/// （条件分支里的读在部分路径消费旧值——`x = C ? x+1 : x+2` 的分支读
/// 就是活引用；条件分支里的写不必然执行 → 不击杀，继续扫。）
enum KillEvent<L: Lang> {
    /// 任意位置的读 → init 活着
    Read,
    /// 支配位置的简单赋值（写）
    DomWrite(L::Id),
    /// 同名重声明（遮蔽）
    Shadow,
}

/// 扫描一个节点在 `name` 上的事件。
/// `dominating`：当前子树是否在必经路径上（If/While 的 cond、Ternary 的
/// cond、For 的 init/cond、非短路二元两侧 = true；分支体/循环体 = false）。
fn scan_kill_event<L: Lang>(
    lang: &L,
    node: L::Id,
    name: &str,
    dominating: bool,
) -> Option<KillEvent<L>> {
    match lang.kind(node) {
        NodeKind::VarRef => {
            if lang.var_name(node) == Some(name) {
                Some(KillEvent::Read)
            } else {
                None
            }
        }
        NodeKind::VarDecl => {
            if lang.var_name(node) == Some(name) {
                Some(KillEvent::Shadow)
            } else {
                // 其他变量声明：init 必经求值
                for &c in lang.children(node) {
                    if let Some(e) = scan_kill_event(lang, c, name, dominating) {
                        return Some(e);
                    }
                }
                None
            }
        }
        NodeKind::Assign => {
            let ch = lang.children(node);
            // RHS 先求值
            if let Some(&val) = ch.get(1) {
                if let Some(e) = scan_kill_event(lang, val, name, dominating) {
                    if matches!(e, KillEvent::Read | KillEvent::Shadow) {
                        return Some(e);
                    }
                }
            }
            if lang.assign_op(node).is_none() {
                if let Some(&t) = ch.first() {
                    if lang.kind(t) == NodeKind::VarRef && lang.var_name(t) == Some(name) {
                        if dominating {
                            return Some(KillEvent::DomWrite(node));
                        }
                        return None; // 条件写：不击杀，也不算读
                    }
                }
            }
            None
        }
        NodeKind::If | NodeKind::While | NodeKind::Ternary => {
            // cond 必经；分支体/循环体不必然——但其中的【读】仍保活 init，
            // 必须扫描（以非支配标记：读传播，写不击杀）
            let ch = lang.children(node);
            if let Some(&c) = ch.first() {
                if let Some(e) = scan_kill_event(lang, c, name, true) {
                    return Some(e);
                }
            }
            for &c in &ch[1..] {
                if let Some(e) = scan_kill_event(lang, c, name, false) {
                    if matches!(e, KillEvent::Read | KillEvent::Shadow) {
                        return Some(e);
                    }
                }
            }
            None
        }
        NodeKind::Binary => {
            let op = lang.bin_op(node)?;
            let ch = lang.children(node);
            if op.is_short_circuit() {
                // 左支必经；右支条件求值（读→保活；写→不击杀）
                if let Some(&l) = ch.first() {
                    if let Some(e) = scan_kill_event(lang, l, name, true) {
                        return Some(e);
                    }
                }
                if let Some(&r) = ch.get(1) {
                    if let Some(e) = scan_kill_event(lang, r, name, false) {
                        if matches!(e, KillEvent::Read | KillEvent::Shadow) {
                            return Some(e);
                        }
                    }
                }
                None
            } else {
                for &c in ch {
                    if let Some(e) = scan_kill_event(lang, c, name, dominating) {
                        return Some(e);
                    }
                }
                None
            }
        }
        NodeKind::For | NodeKind::ForEach | NodeKind::Switch | NodeKind::Try => {
            // 必经部分（For 的 inits+cond / ForEach 的 iterable / Switch 的
            // subject）以支配标记扫；其余（循环体/case 体/try 块）以非支配
            // 标记扫——其中的读仍保活
            let ch = lang.children(node);
            let dom_len = match lang.kind(node) {
                NodeKind::For => ch.len().saturating_sub(2),
                NodeKind::ForEach | NodeKind::Switch => 1,
                _ => 0,
            };
            for &c in &ch[..dom_len] {
                if let Some(e) = scan_kill_event(lang, c, name, true) {
                    return Some(e);
                }
            }
            for &c in &ch[dom_len..] {
                if let Some(e) = scan_kill_event(lang, c, name, false) {
                    if matches!(e, KillEvent::Read | KillEvent::Shadow) {
                        return Some(e);
                    }
                }
            }
            None
        }
        _ => {
            for &c in lang.children(node) {
                if let Some(e) = scan_kill_event(lang, c, name, dominating) {
                    return Some(e);
                }
            }
            None
        }
    }
}

impl<L: Lang> Rule<L> for StoreKill {
    fn name(&self) -> &'static str {
        "store_kill"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::VarDecl, NodeKind::Assign, NodeKind::ExprStmt]
    }
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>> {
        let RewriteCtx { lang, walk } = ctx;
        // 形态：VarDecl(带 init) 或 Assign（裸/ExprStmt 包裹）
        let (is_decl, stmt_node, first_value) = match lang.kind(id) {
            NodeKind::VarDecl => {
                if lang.children(id).is_empty() {
                    return None;
                }
                (true, id, *lang.children(id).first()?)
            }
            NodeKind::Assign => {
                if lang.assign_op(id).is_some() {
                    return None;
                }
                (false, id, *lang.children(id).get(1)?)
            }
            NodeKind::ExprStmt => {
                let ch = lang.children(id);
                if ch.len() == 1 && lang.kind(ch[0]) == NodeKind::Assign {
                    let a = ch[0];
                    if lang.assign_op(a).is_some() {
                        return None;
                    }
                    (false, id, *lang.children(a).get(1)?)
                } else {
                    return None;
                }
            }
            _ => return None,
        };
        let name = if is_decl {
            lang.var_name(id)?.to_string()
        } else {
            let a = match lang.kind(id) {
                NodeKind::Assign => id,
                _ => lang.children(id)[0],
            };
            let t = *lang.children(a).first()?;
            if lang.kind(t) != NodeKind::VarRef || !lang.is_local_var(t) {
                return None;
            }
            lang.var_name(t)?.to_string()
        };

        let parent = walk.parent(stmt_node)?;
        if lang.kind(parent) != NodeKind::Block {
            return None;
        }
        let idx = walk.index(stmt_node)?;
        let stmts = lang.children(parent).to_vec();

        // 扫描：读（任意路径）→ 保活；支配写 → 击杀；条件写 → 继续
        let mut killed_by: Option<(usize, L::Id)> = None;
        for (si, &s) in stmts.iter().enumerate().skip(idx + 1) {
            match scan_kill_event(&*lang, s, &name, true) {
                Some(KillEvent::Read) | Some(KillEvent::Shadow) => return None,
                Some(KillEvent::DomWrite(assign)) => {
                    killed_by = Some((si, assign));
                    break;
                }
                None => continue,
            }
        }
        let (si, kill_assign) = killed_by?;

        if is_decl {
            // 击杀赋值的值是字面量 → 提升进声明 init
            // （原 init 被替换丢弃——同样必须可丢弃，副作用调用不得删！）
            if lang.effect(first_value) > Effect::MayRead {
                return None;
            }
            let value = *lang.children(kill_assign).get(1)?;
            if is_literal(lang, value) {
                let assign_stmt = stmts[si];
                return Some(Edit::Multi(vec![
                    Edit::Splice {
                        node: id,
                        index: 0,
                        remove: 1,
                        insert: vec![value],
                    },
                    Edit::Delete { node: assign_stmt },
                ]));
            }
            // 否则：剥除 init（裸声明）——init 必须可丢弃（副作用调用不得删！）
            if lang.effect(first_value) > Effect::MayRead {
                return None;
            }
            return Some(Edit::Splice {
                node: id,
                index: 0,
                remove: 1,
                insert: Vec::new(),
            });
        }
        // 赋值形态：首值必须可丢弃
        if lang.effect(first_value) > Effect::MayRead {
            return None;
        }
        Some(Edit::Delete { node: stmt_node })
    }
}

/// value 是否为字面量（提升安全：无求值位置问题）。
fn is_literal<L: Lang>(lang: &L, n: L::Id) -> bool {
    matches!(lang.kind(n), NodeKind::Literal)
}

// ---------------------------------------------------------------------------
// 注册表
// ---------------------------------------------------------------------------

/// 默认规则（保守集：不做 DCE 类清理）。
pub fn default_rules<L: Lang>() -> Vec<Box<dyn Rule<L>>> {
    vec![
        Box::new(DoubleNegFold),
        Box::new(BlockFlatten),
        Box::new(ParenRemoval),
        Box::new(ConstCondition),
        Box::new(BooleanReturn),
        Box::new(IfToTernary),
        Box::new(IfAssignTernary),
        Box::new(IfElseEmpty),
        Box::new(BoolCompare),
        Box::new(DoubleNot),
        Box::new(BoolNotFold),
        Box::new(BoolShortCircuit),
        Box::new(NotCompare),
        Box::new(TernaryFold),
        Box::new(TernaryBool),
        Box::new(TernaryBoolOp),
        Box::new(ConstFoldBin),
        Box::new(CmpConstFold),
        Box::new(BitIdentity),
        Box::new(ArithReassoc),
        Box::new(SelfAssign),
        Box::new(ArithIdentity),
        Box::new(ArithZero),
        Box::new(DeclAssignMerge),
        Box::new(LocalPropagation),
        Box::new(AssignPropagation),
        Box::new(MultiUseCopyPropagation),
        Box::new(TrailingReturn),
        Box::new(DeadStore),
        Box::new(StoreKill),
    ]
}

/// 全量规则（含 DCE 类选配项；按设计文档"可选配，默认不启用"）。
pub fn all_rules<L: Lang>() -> Vec<Box<dyn Rule<L>>> {
    let mut rules = default_rules();
    rules.push(Box::new(UnreachableAfterTerminal));
    rules
}
