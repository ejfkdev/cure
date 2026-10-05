//! 迭代式遍历（显式栈，避免深树递归爆栈），并收集父指针表。

use std::collections::HashMap;

use crate::lang::Lang;

/// 先序遍历的快照：id 列表 + 每个非根节点的 (parent, child_index)。
///
/// 快照在遍历时一次性取出；之后 apply 编辑不影响本次快照的正确性
/// （arena 追加式，Id 永不失效；被改动节点的旧信息由“命中即重启扫描”策略规避）。
pub struct Walk<L: Lang> {
    pub ids: Vec<L::Id>,
    pub(crate) parents: HashMap<L::Id, (L::Id, usize)>,
    pub root: L::Id,
}

impl<L: Lang> Walk<L> {
    pub fn parent(&self, id: L::Id) -> Option<L::Id> {
        self.parents.get(&id).map(|p| p.0)
    }
    pub fn index(&self, id: L::Id) -> Option<usize> {
        self.parents.get(&id).map(|p| p.1)
    }
}

pub fn walk<L: Lang>(lang: &L, root: L::Id) -> Walk<L> {
    let mut ids = Vec::new();
    let mut parents: HashMap<L::Id, (L::Id, usize)> = HashMap::new();
    // 栈元素: (节点, 父, 序号)
    let mut stack: Vec<(L::Id, Option<(L::Id, usize)>)> = vec![(root, None)];
    while let Some((id, par)) = stack.pop() {
        if let Some((p, i)) = par {
            parents.insert(id, (p, i));
        }
        ids.push(id);
        let children = lang.children(id);
        // 逆序入栈保证先序
        for (i, &c) in children.iter().enumerate().rev() {
            stack.push((c, Some((id, i))));
        }
    }
    Walk {
        ids,
        parents,
        root,
    }
}
