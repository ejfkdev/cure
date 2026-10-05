//! 规则 trait 与编辑描述。
//!
//! 事务模型：`check`（只读 + 允许追加节点）产出 `Edit` 提案 →
//! runner 计算成本降幅（必须 > 0）→ 原子应用。任何 edit 都必须严格降低成本，
//! 这保证 fixed-point 收敛、杜绝 rewrite 振荡。

use crate::cost;
use crate::lang::Lang;
use crate::walk::Walk;

/// 一次结构编辑提案。
#[derive(Debug)]
pub enum Edit<L: Lang> {
    /// 用 `with` 子树替换 `target` 节点（target 必须有父节点）。
    Replace { target: L::Id, with: L::Id },
    /// 删除一个节点（其父从 children 中移除该槽位）。
    Delete { node: L::Id },
    /// 对 `node` 自身的 children 做切片编辑。
    Splice {
        node: L::Id,
        index: usize,
        remove: usize,
        insert: Vec<L::Id>,
    },
    /// 多个编辑按序原子应用。
    /// 约束：前面的编辑不得使后面编辑的 (parent, index) 失效——
    /// 先替换靠后的节点、后删除靠前的节点是安全顺序。
    Multi(Vec<Edit<L>>),
}

impl<L: Lang> Edit<L> {
    /// 应用本编辑会带来的成本降幅（严格 > 0 才允许生效）。
    pub fn cost_reduction(&self, lang: &mut L) -> u64 {
        match self {
            Edit::Replace { target, with } => {
                let a = cost::subtree_cost(lang, *target);
                let b = cost::subtree_cost(lang, *with);
                a.saturating_sub(b)
            }
            Edit::Delete { node } => cost::subtree_cost(lang, *node),
            Edit::Splice {
                node,
                index,
                remove,
                insert,
            } => {
                let children = lang.children(*node);
                let mut removed = 0u64;
                for &c in &children[*index..(*index + remove).min(children.len())] {
                    removed += cost::subtree_cost(lang, c);
                }
                let mut added = 0u64;
                for &n in insert.iter() {
                    added += cost::subtree_cost(lang, n);
                }
                removed.saturating_sub(added)
            }
            Edit::Multi(edits) => edits.iter().map(|e| e.cost_reduction(lang)).sum(),
        }
    }
}

/// 规则上下文：语言 AST 的可变访问 + 当前扫描的父/序号信息。
pub struct RewriteCtx<'a, L: Lang> {
    pub lang: &'a mut L,
    pub(crate) walk: &'a Walk<L>,
}

impl<'a, L: Lang> RewriteCtx<'a, L> {
    pub fn parent(&self, id: L::Id) -> Option<L::Id> {
        self.walk.parent(id)
    }
    pub fn index(&self, id: L::Id) -> Option<usize> {
        self.walk.index(id)
    }
    pub fn root(&self) -> L::Id {
        self.walk.root
    }
}

/// 一条简化规则。
pub trait Rule<L: Lang> {
    fn name(&self) -> &'static str;

    /// 本规则关注的节点类别（**空 = 全部类别**）。pass 循环按类别建分派桶，
    /// 无关节点直接跳过检查——"全部规则 × 每节点一次虚调用"是扫描主开销。
    /// 声明必须覆盖 check() 入口的真实类别判定：多报只是浪费检查（正确），
    /// 漏报会让规则失效（漏改写，回归测试会暴露）。
    fn kinds(&self) -> &'static [crate::kind::NodeKind] {
        &[]
    }

    /// 在节点 `id` 处检查是否可改写。
    /// 契约：只读既有结构（可向 arena 追加新节点）；结构变更只经 `Edit` 提案。
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>>;

    /// 结构性规则：**豁免成本严格下降约束**。
    /// 适用于"内联后暂时变贵、由后续折叠回本"的变换（如方法内联）。
    /// 实现方必须自证终止（如：每次消费一类节点且不引入新的同类节点）。
    fn structural(&self) -> bool {
        false
    }
}

/// runner 内部：用父表应用编辑。
pub(crate) fn apply_edit<L: Lang>(
    lang: &mut L,
    walk: &Walk<L>,
    edit: &Edit<L>,
) -> Result<(), &'static str> {
    match edit {
        Edit::Replace { target, with } => {
            let (parent, index) = walk
                .parents
                .get(target)
                .copied()
                .ok_or("cannot replace root")?;
            lang.set_child(parent, index, *with);
            Ok(())
        }
        Edit::Delete { node } => {
            let (parent, index) = walk.parents.get(node).copied().ok_or("cannot delete root")?;
            lang.remove_child(parent, index);
            Ok(())
        }
        Edit::Splice {
            node,
            index,
            remove,
            insert,
        } => {
            lang.splice(*node, *index, *remove, insert.clone());
            Ok(())
        }
        Edit::Multi(edits) => {
            for e in edits {
                apply_edit(lang, walk, e)?;
            }
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// 同 pass 批量应用的支持设施：
//
// 正确性基石——**纯 Replace（含全 Replace 的 Multi）原位换孩子，不移动
// 兄弟下标** ⇒ 快照的 (parent, index) 全程有效，可立即应用并继续扫描；
// 含 Delete/Splice 的结构编辑会移动兄弟下标 ⇒ 延迟到 pass 末统一应用。
//
// 冲突域：每个编辑有「足迹」（它改动/删除的节点 + 其子树）。同 pass 内
// 已应用（立即路径）与已排队（延迟路径）编辑的足迹之并 = touched 集合：
//   - 扫描跳过 touched 节点（已脱离/将被删，提案无意义）；
//   - 新提案足迹与 touched 相交 → 放弃本轮（下一 pass 重新提案）。
//
// 延迟应用的对账：
//   - Replace/Delete 在父的孩子表中按节点身份搜索定位（对下标漂移稳健）；
//   - Splice 要求被删区间仍等于提案时记录的节点序列，否则丢弃整个编辑
//     （丢弃不影响成本单调性：已应用编辑各自严格降本）；
//   - Multi 原子性：先全部校验、后逐元素应用。
// 排序约定：Multi 内同父的 Splice 应先于 Delete/低下标编辑（既有规则
// 均遵守，见 CFF/LocalProp 注释）；跨编辑按 (parent, 首个结构位置) 降序
// 应用，最小化 Splice 漂移。
// ---------------------------------------------------------------------------

/// 纯 Replace 判定。
pub(crate) fn is_replace_only<L: Lang>(edit: &Edit<L>) -> bool {
    match edit {
        Edit::Replace { .. } => true,
        Edit::Multi(es) => es.iter().all(|e| matches!(e, Edit::Replace { .. })),
        _ => false,
    }
}

/// 编辑足迹：将被改动/删除的节点（含子树），提案时快照。
pub(crate) fn collect_footprint<L: Lang>(lang: &L, edit: &Edit<L>, out: &mut Vec<L::Id>) {
    match edit {
        Edit::Replace { target, .. } => collect_subtree(lang, *target, out),
        Edit::Delete { node } => collect_subtree(lang, *node, out),
        Edit::Splice {
            node, index, remove, ..
        } => {
            let ch = lang.children(*node);
            if *index <= ch.len() && *index + *remove <= ch.len() {
                for &c in &ch[*index..*index + *remove] {
                    collect_subtree(lang, c, out);
                }
            }
        }
        Edit::Multi(es) => {
            for e in es {
                collect_footprint(lang, e, out);
            }
        }
    }
}

fn collect_subtree<L: Lang>(lang: &L, id: L::Id, out: &mut Vec<L::Id>) {
    let mut stack = vec![id];
    while let Some(n) = stack.pop() {
        out.push(n);
        for &c in lang.children(n) {
            stack.push(c);
        }
    }
}

/// Splice 校验数据：提案时被删区间的孩子节点序列（按编辑树先序展平）。
pub(crate) fn collect_splice_expectations<L: Lang>(
    lang: &L,
    edit: &Edit<L>,
    out: &mut Vec<Vec<L::Id>>,
) {
    match edit {
        Edit::Splice {
            node, index, remove, ..
        } => {
            let ch = lang.children(*node);
            if *index <= ch.len() && *index + *remove <= ch.len() {
                out.push(ch[*index..*index + *remove].to_vec());
            } else {
                // 越界占位（保持展平顺序与编辑树对应）
                out.push(Vec::new());
            }
        }
        Edit::Multi(es) => {
            for e in es {
                collect_splice_expectations(lang, e, out);
            }
        }
        _ => {}
    }
}

/// 延迟路径应用前校验（只读）。`expects` 按消费推进。
/// Replace/Delete：目标仍在其快照父的孩子表中（节点身份匹配，无视下标）。
/// Splice：被删区间仍等于提案时记录的序列。
pub(crate) fn verify_deferred<L: Lang>(
    lang: &L,
    snap: &Walk<L>,
    edit: &Edit<L>,
    expects: &mut &[Vec<L::Id>],
) -> bool {
    match edit {
        Edit::Replace { target, .. } => match snap.parents.get(target) {
            Some((p, _)) => lang.children(*p).contains(target),
            None => false,
        },
        Edit::Delete { node } => match snap.parents.get(node) {
            Some((p, _)) => lang.children(*p).contains(node),
            None => false,
        },
        Edit::Splice {
            node, index, remove, ..
        } => {
            let Some(exp) = expects.first() else {
                return false;
            };
            *expects = &expects[1..];
            let ch = lang.children(*node);
            *index <= ch.len()
                && *index + *remove <= ch.len()
                && &ch[*index..*index + *remove] == exp
        }
        Edit::Multi(es) => es.iter().all(|e| verify_deferred(lang, snap, e, expects)),
    }
}

/// 延迟路径应用（校验已通过后）。Replace/Delete 按节点身份搜索定位；
/// Splice 用快照下标（校验已确认区间一致）。
pub(crate) fn apply_deferred<L: Lang>(lang: &mut L, snap: &Walk<L>, edit: &Edit<L>) {
    match edit {
        Edit::Replace { target, with } => {
            if let Some((p, _)) = snap.parents.get(target) {
                if let Some(i) = lang.children(*p).iter().position(|&c| c == *target) {
                    lang.set_child(*p, i, *with);
                }
            }
        }
        Edit::Delete { node } => {
            if let Some((p, _)) = snap.parents.get(node) {
                if let Some(i) = lang.children(*p).iter().position(|&c| c == *node) {
                    lang.remove_child(*p, i);
                }
            }
        }
        Edit::Splice {
            node, index, remove, insert,
        } => {
            lang.splice(*node, *index, *remove, insert.clone());
        }
        Edit::Multi(es) => {
            for e in es {
                apply_deferred(lang, snap, e);
            }
        }
    }
}

/// 结构编辑的排序键：(首个结构元素的父, 位置)。降序应用 ⇒ 同父先动
/// 高下标区间，低下标编辑的快照下标不被移动。
pub(crate) fn structural_sort_key<L: Lang>(snap: &Walk<L>, edit: &Edit<L>) -> (L::Id, usize) {
    fn first_pos<L: Lang>(snap: &Walk<L>, e: &Edit<L>) -> Option<(L::Id, usize)> {
        match e {
            Edit::Splice { node, index, .. } => Some((*node, *index)),
            Edit::Delete { node } => snap
                .parents
                .get(node)
                .map(|(p, i)| (*p, *i)),
            Edit::Replace { target, .. } => snap
                .parents
                .get(target)
                .map(|(p, i)| (*p, *i)),
            Edit::Multi(es) => es.first().and_then(|x| first_pos(snap, x)),
        }
    }
    first_pos(snap, edit).unwrap_or_else(|| (snap.root, 0))
}
