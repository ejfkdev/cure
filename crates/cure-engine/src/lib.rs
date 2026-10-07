//! # cure-engine
//!
//! 语言无关的代码语义简化引擎。核心抽象：
//!
//! - [`Lang`]：语言适配层（结构 + 语义查询 + 构造）；
//! - [`Effect`]：副作用格，语义保守性的基石；
//! - [`Edit`] / [`Rule`]：改写事务（提案 → 成本校验 → 原子应用）；
//! - [`simplify`]：fixed-point pass runner，成本单调下降保证收敛；
//! - [`rules`]: 只依赖 `Lang` 的通用规则集（布尔、自赋值、常量条件、局部传播等）。
//!
//! 语言 crate（如 cure-java-ast）实现 [`Lang`] 并注册规则；本 crate 零外部依赖。

pub mod analysis;
pub use analysis::{subtree_metrics, TreeMetrics};
pub mod cost;
pub mod effect;
pub mod kind;
pub mod lang;
pub mod pass;
pub mod pattern;
pub mod rule;
pub mod rules;
pub mod walk;

pub use effect::Effect;
pub use kind::{BinOp, LitRef, NodeKind, UnOp};
pub use lang::{Lang, ReassocOutcome};
pub use pass::{simplify, Config, Report, fold_root};
pub use rule::{Edit, Rule, RewriteCtx};
