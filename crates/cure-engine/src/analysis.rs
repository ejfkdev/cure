//! 通用轻量分析：局部变量读/写集合（只依赖 `Lang` 规范词汇表）。


use crate::kind::NodeKind;
use crate::lang::Lang;

/// `node` 子树中**显式写**的局部变量名集合。
/// 注意：`Call`/`Member` 等不可知的写不计入——但那些节点的 effect 会是
/// Unknown/MayThrow，调用方须先按 effect 分类，再对“只读局部”路径用本函数。
pub fn local_writes<'a, L: Lang>(lang: &'a L, id: L::Id, out: &mut crate::walk::StrSet<'a>) {
    match lang.kind(id) {
        NodeKind::Assign => {
            let children = lang.children(id);
            if let Some(&t) = children.first() {
                if lang.kind(t) == NodeKind::VarRef {
                    if let Some(n) = lang.var_name(t) {
                        out.insert(n);
                    }
                }
            }
            if let Some(&v) = children.get(1) {
                local_writes(lang, v, out);
            }
        }
        NodeKind::Unary => {
            if let Some(op) = lang.un_op(id) {
                if op.is_incdec() {
                    let children = lang.children(id);
                    if let Some(&t) = children.first() {
                        if lang.kind(t) == NodeKind::VarRef {
                            if let Some(n) = lang.var_name(t) {
                                out.insert(n);
                            }
                        }
                    }
                }
            }
            for &c in lang.children(id) {
                local_writes(lang, c, out);
            }
        }
        NodeKind::VarDecl => {
            if let Some(n) = lang.var_name(id) {
                out.insert(n);
            }
            for &c in lang.children(id) {
                local_writes(lang, c, out);
            }
        }
        _ => {
            for &c in lang.children(id) {
                local_writes(lang, c, out);
            }
        }
    }
}

/// `node` 子树中所有被**读**的变量名（VarRef）。
pub fn reads_vars<'a, L: Lang>(lang: &'a L, id: L::Id, out: &mut crate::walk::StrSet<'a>) {
    if lang.kind(id) == NodeKind::VarRef {
        if let Some(n) = lang.var_name(id) {
            out.insert(n);
        }
    }
    for &c in lang.children(id) {
        reads_vars(lang, c, out);
    }
}

/// 子树中是否存在满足谓词的节点（迭代，防爆栈）。
pub fn subtree_contains<L: Lang>(lang: &L, id: L::Id, pred: impl Fn(L::Id) -> bool) -> bool {
    let mut stack = vec![id];
    while let Some(n) = stack.pop() {
        if pred(n) {
            return true;
        }
        for &c in lang.children(n) {
            stack.push(c);
        }
    }
    false
}

/// 求值顺序上位于 `target` 之前的兄弟子树，效果是否全部 ≤ `MayRead`。
/// （局部变量读与“非写”效果之间可重排；任何 MayThrow 及以上都阻断。）
///
/// `target` 必须在 `root` 子树内。短路的右操作数按“会求值”保守处理。
pub fn prefix_effects_readable<L: Lang>(lang: &L, root: L::Id, target: L::Id) -> bool {
    fn go<L: Lang>(lang: &L, node: L::Id, target: L::Id) -> bool {
        if node == target {
            return true;
        }
        for &c in lang.children(node) {
            if crate::analysis::subtree_contains(lang, c, |n| n == target) {
                // target 在 c 内：c 之前的兄弟都必须 ≤ MayRead
                // （循环里已经在 return 前检查过前面的兄弟）
                return go(lang, c, target);
            } else {
                if lang.effect(c) > crate::effect::Effect::MayRead {
                    return false;
                }
            }
        }
        true
    }
    go(lang, root, target)
}

/// 两棵子树是否**语法结构相同**（kind/运算符/字面量/变量名/children 逐位一致）。
/// 用于 `c ? a : a` 这类同支折叠的判定。纯语法比较，不含语义等价判断。
pub fn structurally_equal<L: Lang>(lang: &L, a: L::Id, b: L::Id) -> bool {
    if lang.kind(a) != lang.kind(b) {
        return false;
    }
    if lang.bin_op(a) != lang.bin_op(b) || lang.un_op(a) != lang.un_op(b) {
        return false;
    }
    if lang.var_name(a) != lang.var_name(b) {
        return false;
    }
    match (lang.literal(a), lang.literal(b)) {
        (Some(x), Some(y)) if x == y => {}
        (None, None) => {}
        _ => return false,
    }
    let (ca, cb) = (lang.children(a), lang.children(b));
    if ca.len() != cb.len() {
        return false;
    }
    ca.iter().zip(cb.iter()).all(|(&x, &y)| structurally_equal(lang, x, y))
}

// ---------------------------------------------------------------------------
// 结构化简化指标：节点数 / 判定点（McCabe 决策点）/ 最大嵌套深度。
//
// 与行数不同，这三项**不受格式化影响**——只被语义简化改变。用于
// 区分「真简化」与「纯格式归一」，以及在大语料上定位未简化的文件
// （指标零变化 = 无规则命中）。
// ---------------------------------------------------------------------------

/// 一个根（通常是方法体）的结构指标。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TreeMetrics {
    /// 子树节点总数（从根可达的 arena 节点；不含编辑残留的游离节点）。
    pub nodes: u64,
    /// 判定点数：if / while / do / for / foreach / case / catch / 三元 /
    /// `&&` / `||`（McCabe 复杂度的决策点部分）。
    pub decisions: u64,
    /// 最大嵌套深度（语句容器嵌套层数；反编译嵌套块与扁平化状态机
    /// 的直接体现）。
    pub max_depth: u32,
}

impl TreeMetrics {
    pub fn add(&mut self, other: TreeMetrics) {
        self.nodes += other.nodes;
        self.decisions += other.decisions;
        self.max_depth = self.max_depth.max(other.max_depth);
    }
}

/// 是否为语句容器（进入其孩子算一层嵌套）。
fn is_container(k: NodeKind) -> bool {
    matches!(
        k,
        NodeKind::Block
            | NodeKind::If
            | NodeKind::While
            | NodeKind::DoWhile
            | NodeKind::For
            | NodeKind::ForEach
            | NodeKind::Try
            | NodeKind::Catch
            | NodeKind::Switch
            | NodeKind::Synchronized
    )
}

/// 是否为判定点（kind 级）。
fn is_decision_kind(k: NodeKind) -> bool {
    matches!(
        k,
        NodeKind::If
            | NodeKind::While
            | NodeKind::DoWhile
            | NodeKind::For
            | NodeKind::ForEach
            | NodeKind::Case
            | NodeKind::Catch
            | NodeKind::Ternary
    )
}

/// 从 `root`（方法体等）累计结构指标。
pub fn subtree_metrics<L: Lang>(lang: &L, root: L::Id) -> TreeMetrics {
    let mut m = TreeMetrics::default();
    walk_metrics(lang, root, 0, &mut m);
    m
}

fn walk_metrics<L: Lang>(lang: &L, id: L::Id, depth: u32, m: &mut TreeMetrics) {
    m.nodes += 1;
    let k = lang.kind(id);
    if is_decision_kind(k) {
        m.decisions += 1;
    }
    // 短路 && / || 是独立判定点（路径分叉）
    if k == NodeKind::Binary && lang.bin_op(id).is_some_and(|o| o.is_short_circuit()) {
        m.decisions += 1;
    }
    let child_depth = if is_container(k) { depth + 1 } else { depth };
    if child_depth > m.max_depth {
        m.max_depth = child_depth;
    }
    for &c in lang.children(id) {
        walk_metrics(lang, c, child_depth, m);
    }
}
