//! 副作用格（effect lattice）。
//!
//! `Lang::effect(id)` 返回**整棵子树**的聚合效果（最坏情况），
//! 节点自身贡献由 `Lang::own_effect` 给出，引擎按 `worst(own, children)` 聚合。
//!
//! 排序约定（全序简化）：`Pure < MayRead < MayThrow < MayWrite < Unknown`。
//! `Pure` 蕴含**无任何可观察效果且结果确定**（同输入同值，可安全重求值/移动）。
//! `MayRead` 表示只读局部变量等确定值；局部变量读与调用之间没有未知写，
//! 因为调用无法修改语言层的局部变量。

use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Effect {
    /// 无可观察效果、结果确定。可自由移动/重求值。
    Pure,
    /// 只读（局部变量等）。与纯读之间可重排；被写依赖阻断。
    MayRead,
    /// 可能抛异常 / 读堆内存（字段、数组）等。
    MayThrow,
    /// 可能写内存。
    MayWrite,
    /// 完全未知（未知调用等）。一律视为不可简化。
    Unknown,
}

impl Effect {
    pub fn is_pure(self) -> bool {
        self == Effect::Pure
    }

    pub fn worst(self, other: Effect) -> Effect {
        if self.rank() >= other.rank() {
            self
        } else {
            other
        }
    }

    fn rank(self) -> u8 {
        match self {
            Effect::Pure => 0,
            Effect::MayRead => 1,
            Effect::MayThrow => 2,
            Effect::MayWrite => 3,
            Effect::Unknown => 4,
        }
    }
}

impl PartialOrd for Effect {
    fn partial_cmp(&self, other: &Effect) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Effect {
    fn cmp(&self, other: &Effect) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering() {
        assert!(Effect::Pure < Effect::MayRead);
        assert!(Effect::MayRead < Effect::MayThrow);
        assert!(Effect::MayThrow < Effect::MayWrite);
        assert!(Effect::MayWrite < Effect::Unknown);
        assert_eq!(Effect::Pure.worst(Effect::MayThrow), Effect::MayThrow);
    }
}
