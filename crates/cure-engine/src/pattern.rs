//! 最小 Pattern DSL：静态可构造的树形模式 + 绑定。
//!
//! 复杂规则仍可直接在 `Rule::check` 里手写结构判断；这里的目的是
//! 让“形状”类规则摆脱 if-let 嵌套，且 `Pat` 是纯数据、未来可序列化成声明式规则。

use crate::kind::{LitRef, NodeKind};
use crate::lang::Lang;

#[derive(Debug)]
pub enum Pat {
    /// 任意节点。
    Any,
    /// 绑定当前节点 id 到名字。
    Bind(&'static str),
    /// 节点 kind 匹配且 children 与 pats 一一同数量、按序匹配。
    Kind(NodeKind, &'static [Pat]),
    /// boolean 字面量。
    LitBool(bool),
    /// 整数字面量（Int/Long）值匹配。
    LitInt(i64),
}

pub type Binds<L> = Vec<(&'static str, <L as Lang>::Id)>;

pub fn match_pat<L: Lang>(lang: &L, id: L::Id, pat: &Pat, binds: &mut Binds<L>) -> bool {
    match pat {
        Pat::Any => true,
        Pat::Bind(name) => {
            binds.push((name, id));
            true
        }
        Pat::LitBool(v) => matches!(lang.literal(id), Some(LitRef::Bool(b)) if b == *v),
        Pat::LitInt(v) => lang.literal(id).and_then(|l| l.as_int()) == Some(*v),
        Pat::Kind(kind, pats) => {
            if lang.kind(id) != *kind {
                return false;
            }
            let children = lang.children(id);
            if children.len() != pats.len() {
                return false;
            }
            children
                .iter()
                .zip(pats.iter())
                .all(|(&c, p)| match_pat(lang, c, p, binds))
        }
    }
}

/// 便捷入口：匹配成功返回绑定表。
pub fn matches<L: Lang>(lang: &L, id: L::Id, pat: &Pat) -> Option<Binds<L>> {
    let mut binds: Binds<L> = Vec::new();
    if match_pat(lang, id, pat, &mut binds) {
        Some(binds)
    } else {
        None
    }
}

impl Pat {
    pub fn bind(name: &'static str) -> Pat {
        Pat::Bind(name)
    }
    pub fn kind(k: NodeKind, pats: &'static [Pat]) -> Pat {
        Pat::Kind(k, pats)
    }
}

#[macro_export]
/// 静态声明一个 Pat 常量，如：
/// `pat!(IF_RET_TF, Kind(If, [Bind("cond"), Kind(Return, [LitBool(true)])…]))`。
macro_rules! pat {
    ($name:ident, $($tok:tt)*) => {
        static $name: $crate::pattern::Pat = $crate::pattern::__pat!($($tok)*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pat {
    (Any) => { $crate::pattern::Pat::Any };
    (Bind($n:literal)) => { $crate::pattern::Pat::Bind($n) };
    (LitBool($v:literal)) => { $crate::pattern::Pat::LitBool($v) };
    (LitInt($v:literal)) => { $crate::pattern::Pat::LitInt($v) };
    (Kind($k:expr, [$($p:tt),* $(,)?])) => {
        $crate::pattern::Pat::Kind($k, &[$($crate::__pat!($p)),*])
    };
}
