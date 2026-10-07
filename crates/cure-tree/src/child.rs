//! 子节点容器（内联优化）。

/// 子节点容器：≤3 个孩子零堆分配（AST 绝大多数节点——
/// Binary/Assign=2、Member/MethodRef=1、VarDecl≤1、Literal=0……
/// 大语料 malloc 采样的剩余大头是每节点一次 Vec 堆分配）；
/// ≥4 溢出到堆。Block/Call 大参数列表溢出属预期路径。
///
/// `Id` 需 `Copy + Default`（`Default` 作为空槽哨兵值；语言侧通常用
/// `u32::MAX` 之类的非法下标 newtype）。
#[derive(Clone, Debug)]
pub struct ChildList<Id: Copy + Default> {
    inline_len: u8,
    inline: [Id; 3],
    heap: Vec<Id>,
}

impl<Id: Copy + Default> Default for ChildList<Id> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Id: Copy + Default + PartialEq> PartialEq for ChildList<Id> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<Id: Copy + Default> ChildList<Id> {
    pub fn new() -> Self {
        Self { inline_len: 0, inline: [Id::default(); 3], heap: Vec::new() }
    }
    pub fn from_vec(v: Vec<Id>) -> Self {
        let n = v.len();
        if n <= 3 {
            let mut cl = Self::new();
            for (i, id) in v.into_iter().enumerate() {
                cl.inline[i] = id;
            }
            cl.inline_len = n as u8;
            cl
        } else {
            Self { inline_len: 0, inline: [Id::default(); 3], heap: v }
        }
    }
    pub fn as_slice(&self) -> &[Id] {
        if self.heap.is_empty() {
            &self.inline[..self.inline_len as usize]
        } else {
            &self.heap
        }
    }
    pub fn len(&self) -> usize {
        if self.heap.is_empty() {
            self.inline_len as usize
        } else {
            self.heap.len()
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// 按索引替换（保持同一形态）。
    pub fn set(&mut self, index: usize, new: Id) {
        if self.heap.is_empty() {
            self.inline[index] = new;
        } else {
            self.heap[index] = new;
        }
    }
    /// 移除一个孩子（内联区左移；堆走 Vec::remove）。
    pub fn remove(&mut self, index: usize) {
        if self.heap.is_empty() {
            for i in index..self.inline_len as usize - 1 {
                self.inline[i] = self.inline[i + 1];
            }
            self.inline_len -= 1;
        } else {
            self.heap.remove(index);
            // 缩回 ≤3：搬回内联
            if self.heap.len() <= 3 {
                let v = std::mem::take(&mut self.heap);
                *self = Self::from_vec(v);
            }
        }
    }
    /// 区间替换（Splice 语义：删 [index, index+remove) 插 insert）。
    pub fn splice(&mut self, index: usize, remove: usize, insert: Vec<Id>) {
        let new_len = self.len() - remove + insert.len();
        if new_len <= 3 && self.heap.is_empty() {
            // 纯内联区间的手工搬移
            let mut result: Vec<Id> = self.as_slice().to_vec();
            let end = (index + remove).min(result.len());
            result.splice(index..end, insert);
            *self = Self::from_vec(result);
        } else {
            // 堆路径：先物化成 Vec，操作，再回填
            let mut v = std::mem::take(&mut self.heap);
            if v.is_empty() {
                v = self.as_slice().to_vec();
                self.inline_len = 0;
            }
            let end = (index + remove).min(v.len());
            v.splice(index..end, insert);
            *self = Self::from_vec(v);
        }
    }
}
