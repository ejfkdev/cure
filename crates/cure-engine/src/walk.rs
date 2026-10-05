//! 迭代式遍历（显式栈，避免深树递归爆栈），并收集父指针表。

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

use crate::lang::Lang;

/// 稠密整数句柄的快速定长 hasher（FxHash 风格）：`L::Id` 的 Hash 实现
/// 通常只写一个 u32/u64，Sip13 的全量安全哈希在这里纯属浪费。
/// 冲突安全性由 HashMap 的探测保证，这里只提供廉价混合。
#[derive(Default)]
pub(crate) struct IdHasher(u64);

impl IdHasher {
    #[inline]
    fn mix(&mut self, v: u64) {
        self.0 = (self.0.rotate_left(5) ^ v).wrapping_mul(0x517c_c1b7_2722_0a95);
    }
}

impl Hasher for IdHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.mix(b as u64);
        }
    }
    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.mix(i as u64);
    }
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.mix(i);
    }
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.mix(i as u64);
    }
}

pub(crate) type IdBuild = BuildHasherDefault<IdHasher>;
pub(crate) type IdMap<K, V> = HashMap<K, V, IdBuild>;

/// 先序遍历的快照：id 列表 + 每个非根节点的 (parent, child_index)。
///
/// 快照在遍历时一次性取出；之后 apply 编辑不影响本次快照的正确性
/// （arena 追加式，Id 永不失效；被改动节点的旧信息由“命中即重启扫描”策略规避）。
pub struct Walk<L: Lang> {
    pub ids: Vec<L::Id>,
    pub(crate) parents: IdMap<L::Id, (L::Id, usize)>,
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
    let mut parents: IdMap<L::Id, (L::Id, usize)> = HashMap::default();
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
