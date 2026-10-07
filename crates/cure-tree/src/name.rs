//! 名字 intern 表 + 每节点名字键。

use std::collections::HashMap;

/// 无名节点的键（哨兵：intern id 从 0 只增，实际不可达）。
pub const KEY_NONE: u32 = u32::MAX;

/// 名字表：单元级「名字 ↔ u32」intern（只增、永不回收）+ 每节点的
/// 名字键（槽位 = arena 下标，`KEY_NONE` = 无名）。
///
/// 语言的 `Lang::NameKey` 采用 intern u32 时（推荐——区域扫描的名字
/// 比较退化为整数等值）直接复用本表。
#[derive(Default, Debug)]
pub struct NameTable {
    /// 名字 → 键（intern）。
    name_to_key: HashMap<String, u32>,
    /// 键 → 名字原文（intern 顺序稠密）。
    names: Vec<String>,
    /// 节点名字键（槽位 = arena 下标；`KEY_NONE` = 无名）。
    /// prepare 增量扩展（新节点 intern；既有节点键与名字恒同）。
    node_key: Vec<u32>,
}

impl NameTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// 名字 → 键（intern：同名字同键）。
    pub fn intern(&mut self, name: &str) -> u32 {
        if let Some(&k) = self.name_to_key.get(name) {
            return k;
        }
        let k = self.names.len() as u32;
        self.name_to_key.insert(name.to_string(), k);
        self.names.push(name.to_string());
        k
    }

    /// 键 → 名字原文（未知键返回空串，与调用方 `sn` 惯例一致）。
    pub fn name(&self, key: u32) -> &str {
        self.names.get(key as usize).map(|s| s.as_str()).unwrap_or("")
    }

    /// 节点名字键（`KEY_NONE` → None；供 `Lang::var_key`）。
    pub fn var_key(&self, idx: usize) -> Option<u32> {
        match self.node_key.get(idx) {
            Some(&k) if k != KEY_NONE => Some(k),
            _ => None,
        }
    }

    /// 原始键（含 `KEY_NONE`；供兼容旧路径）。
    pub fn raw_key(&self, idx: usize) -> u32 {
        self.node_key.get(idx).copied().unwrap_or(KEY_NONE)
    }

    /// 已填充的节点键槽数（与 arena 下标对齐检查用）。
    pub fn key_slots(&self) -> usize {
        self.node_key.len()
    }

    /// 增量扩展节点键表：为下标 `[base, base + names.len())` 的**新**
    /// 节点填键（名字已由语言侧先收集——`Lang::var_name` 的借用安全
    /// 形态）。既有节点的键与其名字恒同，无需重填。
    pub fn extend_owned(&mut self, base: usize, names: &[Option<String>]) {
        if self.node_key.len() < base + names.len() {
            self.node_key.resize(base + names.len(), KEY_NONE);
        }
        for (i, name) in names.iter().enumerate() {
            if let Some(n) = name {
                self.node_key[base + i] = self.intern(n);
            }
        }
    }
}
