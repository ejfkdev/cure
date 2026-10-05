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

    /// 在节点 `id` 处检查是否可改写。
    /// 契约：只读既有结构（可向 arena 追加新节点）；结构变更只经 `Edit` 提案。
    fn check(&self, ctx: RewriteCtx<'_, L>, id: L::Id) -> Option<Edit<L>>;
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
