//! 区域事件索引：存储 + 泛型收集器。

use cure_engine::kind::{EventKind, NodeKind, RegionEvent};
use cure_engine::Lang;

/// 事件存储（稠密：槽位 = arena 下标）+ B3 惰性重建的清洁标记。
///
/// 「空序列 = 未索引」是三重哨兵：未构建、被 `invalidate` 失效、或
/// 子树含不透明节点（不建索引——键排序分区看不见 Opaque 键，查询侧
/// 走引擎的 `scan_region` 递归路径，其 Opaque 分支保守拒绝）。
#[derive(Debug)]
pub struct EventStore<Id, NameKey> {
    events: Vec<Vec<RegionEvent<Id, NameKey>>>,
    clean: bool,
}

impl<Id, NameKey> Default for EventStore<Id, NameKey> {
    fn default() -> Self {
        Self { events: Vec::new(), clean: false }
    }
}

impl<Id: Clone, NameKey: Clone> EventStore<Id, NameKey> {
    pub fn new() -> Self {
        Self::default()
    }

    /// 槽位扩到 `count`（不缩；新槽为空 = 未索引）。
    pub fn resize(&mut self, count: usize) {
        if self.events.len() < count {
            self.events.resize(count, Vec::new());
        }
    }

    /// 空槽 = 未索引（调用方回落引擎递归路径）。
    pub fn slot(&self, idx: usize) -> &[RegionEvent<Id, NameKey>] {
        self.events.get(idx).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn put(&mut self, idx: usize, events: Vec<RegionEvent<Id, NameKey>>) {
        if self.events.len() <= idx {
            self.events.resize(idx + 1, Vec::new());
        }
        self.events[idx] = events;
    }

    pub fn is_clean(&self) -> bool {
        self.clean
    }
    pub fn mark_clean(&mut self) {
        self.clean = true;
    }

    /// 失效一个语句条目（编辑沿祖先链调用）：清空该槽 + 整表脏标记
    /// （B3：下次 prepare 只重建**空**条目——未失效条目的事件是子树
    /// 局部的，与位置无关）。
    pub fn invalidate(&mut self, idx: usize) {
        self.clean = false;
        if let Some(slot) = self.events.get_mut(idx) {
            slot.clear();
        }
    }
}

/// 语句级节点枚举：Block 的直接孩子（引擎规则只对这些调用
/// `scan_region`）。遍历整树，遇到 Block 就为其每个孩子登记。
pub fn stmt_roots<L: Lang>(lang: &L, root: L::Id) -> Vec<L::Id> {
    let mut stack = vec![root];
    let mut stmt_roots: Vec<L::Id> = Vec::new();
    while let Some(n) = stack.pop() {
        if lang.kind(n) == NodeKind::Block {
            for &c in lang.children(n) {
                stmt_roots.push(c);
            }
        }
        for &c in lang.children(n) {
            stack.push(c);
        }
    }
    stmt_roots
}

/// 收集 `node` 子树的事件序列，**按键稳定排序**：同键内保持遍历序
/// （查询侧只消费键内序——Use 的顺序、Write/Shadow 的命中，跨键顺序
/// 无关），二分定位键区间。
///
/// 返回空 Vec = 子树含不透明节点（调用方存空 = 未索引哨兵）。
/// 事件序与引擎 `scan_region` 原递归**严格一致**（分配器 dispatch 的
/// 是同一组 Lang 钩子）。
pub fn collect_region_events<L: Lang>(
    lang: &L,
    node: L::Id,
) -> Vec<RegionEvent<L::Id, L::NameKey>> {
    let mut out = Vec::new();
    let opaque = collect_events_into(lang, node, &mut out);
    if opaque {
        return Vec::new();
    }
    out.sort_by_key(|e| e.key);
    out
}

/// 返回值：子树是否含不透明节点（读/写集不可证明）。
fn collect_events_into<L: Lang>(
    lang: &L,
    node: L::Id,
    out: &mut Vec<RegionEvent<L::Id, L::NameKey>>,
) -> bool {
    if lang.is_opaque(node) {
        return true;
    }
    match lang.kind(node) {
        // 赋值：目标是简单变量 → 对其名字写入；事件节点记 target。
        // 只递归 RHS（target 的读不算事件——写即事件）。
        NodeKind::Assign => {
            let ch = lang.children(node);
            if let Some(&t) = ch.first() {
                if lang.kind(t) == NodeKind::VarRef {
                    if let Some(k) = lang.var_key(t) {
                        out.push(RegionEvent { kind: EventKind::Write, node: t, key: k });
                    }
                }
            }
            match ch.get(1) {
                Some(&v) => collect_events_into(lang, v, out),
                None => false,
            }
        }
        // 自增/自减：对目标写入（目标自身不再产生读事件）
        NodeKind::Unary if lang.un_op(node).is_some_and(|o| o.is_incdec()) => {
            if let Some(&t) = lang.children(node).first() {
                if lang.kind(t) == NodeKind::VarRef {
                    if let Some(k) = lang.var_key(t) {
                        out.push(RegionEvent { kind: EventKind::Write, node: t, key: k });
                    }
                }
            }
            false
        }
        // 声明：写 + 遮蔽（事件节点 = 声明节点自身），全孩子递归
        NodeKind::VarDecl | NodeKind::ForEach => {
            if let Some(k) = lang.var_key(node) {
                out.push(RegionEvent { kind: EventKind::Write, node, key: k });
                out.push(RegionEvent { kind: EventKind::Shadow, node, key: k });
            }
            let mut opaque = false;
            for &c in lang.children(node) {
                opaque |= collect_events_into(lang, c, out);
            }
            opaque
        }
        // catch 绑定：仅遮蔽
        NodeKind::Catch => {
            if let Some(k) = lang.var_key(node) {
                out.push(RegionEvent { kind: EventKind::Shadow, node, key: k });
            }
            let mut opaque = false;
            for &c in lang.children(node) {
                opaque |= collect_events_into(lang, c, out);
            }
            opaque
        }
        NodeKind::VarRef => {
            if let Some(k) = lang.var_key(node) {
                out.push(RegionEvent { kind: EventKind::Use, node, key: k });
            }
            false
        }
        _ => {
            let mut opaque = false;
            for &c in lang.children(node) {
                opaque |= collect_events_into(lang, c, out);
            }
            opaque
        }
    }
}
