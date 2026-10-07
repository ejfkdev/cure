//! # cure-tree
//!
//! 通用 arena 树工具包：为 [`cure_engine::Lang`] 的实现语言提供每个语言
//! 都要重复构建的基础设施。一个新语言前端 = 本工具包 + 语言自己的
//! NodeData / parser / printer / 规则包。
//!
//! 约定（工具包假设的语言形态）：
//! - 节点 = 稠密 arena 下标（`Lang::node_index` 与 `Lang::id_of_index`
//!   互逆；`Id: Copy`）
//! - 名字键 = `u32` intern id（`NameTable`：`名字 ↔ u32` 只增表 +
//!   每节点名字键表）
//! - children 布局遵循 [`cure_engine::kind::NodeKind`] 的文档
//!   （Assign = `[target, value]` 等——区域事件收集器按此遍历）
//!
//! 模块：
//! - [`ChildList`]：≤3 孩子内联零堆分配的容器（AST 扇出绝大多数 ≤3）
//! - [`NameTable`]：单元级名字 intern 表 + 节点名字键（增量扩展）
//! - [`EventStore`] + [`stmt_roots`] + [`collect_region_events`]：
//!   区域事件索引（B3 惰性重建：只重建被失效的空条目）与**泛型**事件
//!   收集器（只依赖 Lang 钩子：`is_opaque` / `var_key` / `kind` /
//!   `children` / `un_op`——与引擎 `scan_region` 递归严格同序）
//! - [`rebuild_effects`]：聚合效果表的不动点重建
//!   （children-先于-parent 索引序 + 注入节点违例的多轮收敛）
//!
//! 集成模式（语言侧 `Lang::prepare` / `invalidate_effect` 的标准骨架）：
//!
//! ```ignore
//! fn prepare(&mut self, root: JavaId) {
//!     // 1) 新节点名字键增量扩展（名字先收集，借用安全）
//!     let names = /* base..count 收集 var_name */;
//!     self.names.extend_owned(base, &names);
//!     // 2) 区域事件索引：B3 只重建空条目
//!     self.events.resize(self.nodes.len());
//!     if !self.events.is_clean() {
//!         self.events.mark_clean();
//!         for sr in cure_tree::stmt_roots(self, root) {
//!             if self.events.slot(sr).is_empty() {
//!                 let ev = cure_tree::collect_region_events(self, sr);
//!                 self.events.put(self.node_index(sr), ev);
//!             }
//!         }
//!     }
//!     // 3) 效果表
//!     self.effect_cache = cure_tree::rebuild_effects(self, self.nodes.len());
//!     // 4) 语言专属（作用域类型解析等）
//! }
//!
//! fn invalidate_effect(&mut self, id: JavaId) {
//!     self.events.invalidate(self.node_index(id));
//!     // + 语言自己的缓存失效
//! }
//! ```
//!
//! 零外部依赖（仅 cure-engine）。

mod child;
mod effect;
mod events;
mod name;

pub use child::ChildList;
pub use effect::rebuild_effects;
pub use events::{collect_region_events, stmt_roots, EventStore};
pub use name::{NameTable, KEY_NONE};
