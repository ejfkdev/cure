//! 成本模型：默认节点权重表。
//!
//! 目标是给“改写后是否更简单”一个可比较的量：语句与临时变量比表达式贵，
//! 字面量/变量引用最便宜。语言可经 `Lang::cost_weight` 覆盖。

use crate::kind::NodeKind;
use crate::lang::Lang;

pub fn default_weight(kind: NodeKind) -> u64 {
    use NodeKind::*;
    match kind {
        Literal | VarRef | Paren | This | Empty => 1,
        Binary | Unary | Break | Continue => 3,
        Member | Index | ExprStmt => 3,
        Block | ArrayLit | Ternary | Cast => 4,
        Return | Yield | Call => 4,
        If | While | DoWhile | Throw | New => 6,
        Assign => 8,
        For | ForEach => 8,
        VarDecl => 12,
        Switch | Case | Try | Catch => 10,
        Synchronized | Assert | Label | InstanceOf | Lambda | MethodRef | NewArray => 6,
        Raw => 20,
        Custom(_) => 5,
    }
}

/// 迭代式子树成本（防爆栈）。
pub fn subtree_cost<L: Lang>(lang: &L, id: L::Id) -> u64 {
    let mut total = 0u64;
    let mut stack = vec![id];
    while let Some(n) = stack.pop() {
        total += lang.cost_weight(lang.kind(n));
        for &c in lang.children(n) {
            stack.push(c);
        }
    }
    total
}
