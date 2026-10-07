//! `Lang` trait：语言适配层。引擎的一切通用逻辑只通过它访问语言 AST。
//!
//! 结构契约：
//! - 节点用稳定句柄 `Id`（语言侧 arena 索引），arena 只追加不回收，`Id` 永不失效；
//! - children 顺序 = 书写/求值顺序（各 kind 的布局见 [`crate::kind::NodeKind`]）；
//! - `check` 阶段允许**追加**新节点（arena push），但不得改动既有结构；
//!   结构变更只允许通过 `Edit` 在应用阶段发生。

use std::fmt::Debug;
use std::hash::Hash;

use crate::effect::Effect;
use crate::kind::{BinOp, LitRef, NodeKind, UnOp};

pub trait Lang {
    /// 稳定节点句柄（通常是 `Copy` 的索引 newtype）。
    type Id: Copy + Eq + Hash + Ord + Debug;
    /// 名字等价类句柄（Java：u32 intern id）。区域扫描的全部名字比较
    /// 走它——语言侧应保证同名字同键、比较为整数等值（无字符串 memcmp）。
    type NameKey: Copy + Eq + Ord + Debug;
    /// 节点在语言 arena 中的稠密下标（供引擎位图/数组索引）。
    fn node_index(&self, id: Self::Id) -> usize;
    /// `node_index` 的逆映射（下标 → Id）。cure-tree 工具包的线性
    /// 扫描（效果表重建、名字键扩展）需要按 arena 下标遍历。
    fn id_of_index(&self, idx: usize) -> Self::Id;

    // ---- 结构 ----

    fn kind(&self, id: Self::Id) -> NodeKind;
    fn children(&self, id: Self::Id) -> &[Self::Id];

    /// 把 `parent` 的第 `index` 个孩子替换为 `new`。
    fn set_child(&mut self, parent: Self::Id, index: usize, new: Self::Id);
    /// 删除 `parent` 的第 `index` 个孩子。
    fn remove_child(&mut self, parent: Self::Id, index: usize);
    /// 从 `node` 的第 `index` 个孩子开始删 `remove` 个、插入 `insert`。
    fn splice(&mut self, node: Self::Id, index: usize, remove: usize, insert: Vec<Self::Id>);

    // ---- 构造 ----

    fn build_return(&mut self, value: Option<Self::Id>) -> Self::Id;
    fn build_block(&mut self, stmts: Vec<Self::Id>) -> Self::Id;
    fn build_unary(&mut self, op: UnOp, operand: Self::Id) -> Self::Id;
    fn build_bool(&mut self, v: bool) -> Self::Id;
    /// 构造整数/long 字面量（常量折叠产物）。`long` 为 true 时带 L 语义。
    fn build_int(&mut self, v: i64, long: bool) -> Self::Id;
    /// 构造二元运算节点（常量折叠 / 取反重写等需要重建比较运算时使用）。
    fn build_bin(&mut self, op: BinOp, l: Self::Id, r: Self::Id) -> Self::Id;
    /// 构造三元表达式 `c ? a : b`（if→ternary 归并）。
    fn build_ternary(&mut self, c: Self::Id, a: Self::Id, b: Self::Id) -> Self::Id;
    /// 构造简单赋值 `target = value`（if→三元赋值归并）。
    fn build_assign(&mut self, target: Self::Id, value: Self::Id) -> Self::Id;
    /// 构造字符串字面量（字符串常量折叠产物）。
    fn build_str(&mut self, s: &str) -> Self::Id;
    /// 构造字符字面量（部分求值产物：`(char)('a'+1)` → 'b'）。
    fn build_char(&mut self, c: char) -> Self::Id;
    /// 深拷贝子树（返回新 Id）。
    fn copy_subtree(&mut self, id: Self::Id) -> Self::Id;

    // ---- 语义查询 ----

    /// 整棵子树的聚合效果（最坏情况）。语言侧应缓存（见 `prepare`）。
    fn effect(&self, id: Self::Id) -> Effect;
    /// 使 `id` 的聚合效果缓存失效（若有）。引擎在同 pass 批量应用编辑后
    /// 沿祖先链调用——祖先的聚合效果含被改子树，必须失效，
    /// 后续 `effect()` 读取回退为按需重算（子缓存仍有效，代价 O(孩子数)）。
    /// 默认空实现：无缓存的语言每次现算，天然正确。
    fn invalidate_effect(&mut self, _id: Self::Id) {}

    /// `id` 子树的区域事件序列（使用索引，prepare 期间预计算）。
    /// 事件序必须等于 scan_region 原递归的遍历序；返回 None 表示该
    /// 节点无索引（未构建/已失效）——scan_region 回退为原递归遍历。
    /// 引擎在应用编辑后沿祖先链调用 invalidate_effect 使索引失效。
    /// 事件索引一致性自检（惰性重建 B3 调试）：默认空实现。
    fn debug_verify_events(&self, _stmt: Self::Id) {}

    fn region_events(
        &self,
        _id: Self::Id,
    ) -> Option<&[crate::kind::RegionEvent<Self::Id, Self::NameKey>]> {
        None
    }
    /// 节点的名字键（VarRef/VarDecl/ForEach/Catch 等命名节点）；
    /// 无名节点返回 None。
    fn var_key(&self, _id: Self::Id) -> Option<Self::NameKey> {
        None
    }
    /// 节点是否不透明（内部含不可解析原文，读/写集不可证明）：
    /// Java 的 Raw 节点与匿名类体（anon_raw）。区域用量分析的可靠性
    /// 前提——漏判会导致捕获变量被误删（对抗波 4 抓获）。
    fn is_opaque(&self, _id: Self::Id) -> bool {
        false
    }
    /// 节点自身（不含 children）的效果贡献。
    fn own_effect(&self, id: Self::Id) -> Effect {
        match self.kind(id) {
            NodeKind::Literal
            | NodeKind::Paren
            | NodeKind::This
            | NodeKind::Block
            | NodeKind::Empty
            | NodeKind::VarDecl => Effect::Pure,
            NodeKind::VarRef => Effect::MayRead,
            NodeKind::Member => Effect::MayThrow,
            NodeKind::Index | NodeKind::New => Effect::MayThrow,
            NodeKind::Call => Effect::Unknown,
            NodeKind::Assign => Effect::MayWrite,
            NodeKind::Unary => {
                if self.un_op(id).is_some_and(|o| o.is_incdec()) {
                    Effect::MayWrite
                } else {
                    Effect::Pure
                }
            }
            NodeKind::Binary => {
                if self.bin_op(id).is_some_and(|o| o.may_div_zero()) {
                    Effect::MayThrow
                } else {
                    Effect::Pure
                }
            }
            NodeKind::Return | NodeKind::Throw => Effect::MayThrow,
            NodeKind::Try | NodeKind::ForEach | NodeKind::Assert => Effect::MayThrow,
            NodeKind::Catch | NodeKind::NewArray => Effect::MayThrow,
            NodeKind::Synchronized => Effect::MayWrite,
            NodeKind::Raw => Effect::Unknown,
            _ => Effect::Unknown,
        }
    }

    /// 表达式是否为**原始 boolean** 类型（装箱 Boolean 必须返回 false）。
    fn is_bool(&self, id: Self::Id) -> bool;
    /// 表达式是否具有精确整数类型（byte/short/int/long/char），用于算术恒等式守卫。
    fn is_exact_int(&self, id: Self::Id) -> bool;
    /// 是否为语言层的普通局部变量（可安全删除赋值；字段/全局必须 false）。
    fn is_local_var(&self, id: Self::Id) -> bool {
        self.kind(id) == NodeKind::VarRef
    }

    /// 声明（VarDecl 节点）的类型域是否容纳 int 算术结果。窄类型
    /// （Java byte/short/char）的复合赋值含**隐式收窄**
    /// （`b += K` ≡ `b = (byte)(b + K)`）——把 delta 折回声明会产出
    /// 非法（超域常量）或变义（丢静态类型：`char ch = 99` 内联进
    /// println 打数字而非字符）的字面量。默认 true（宽域语言）。
    fn is_wide_decl(&self, _decl: Self::Id) -> bool {
        true
    }

    /// 声明是否为字符类型。字符类型变量对 **int 字面量内联类型敏感**：
    /// `char c = 98; println(c)` 打印 'b'，而内联成 `println(98)` 打印
    /// 98——静态类型可被 println / 字符串拼接观察。窄域规则（如
    /// local_propagation）须拒绝非 Char 字面量值出字符声明。
    fn is_char_decl(&self, _decl: Self::Id) -> bool {
        false
    }

    fn bin_op(&self, id: Self::Id) -> Option<BinOp>;
    fn un_op(&self, id: Self::Id) -> Option<UnOp>;
    fn literal(&self, id: Self::Id) -> Option<LitRef<'_>>;
    /// 变量相关名字：VarRef / VarDecl 读写的名字，以及 ForEach / Catch 的**声明绑定名**。
    fn var_name(&self, id: Self::Id) -> Option<&str>;
    /// 赋值运算（`=` 之外的复合赋值也在此表达；None 表示简单赋值）。
    fn assign_op(&self, _id: Self::Id) -> Option<BinOp> {
        None
    }

    // ---- 字面量折叠语义（数值/字符串语义全权归属语言侧）----
    //
    // 引擎规则只做**形态**匹配（双字面量、算符类别、代价门槛）；一切
    // 数值语义——宽度与环绕、除零策略、移位掩码、字符串拼接的强制
    // 转换规则——由这些钩子的语言实现决定。默认全部不折（保守）。

    /// 字面量二元折叠：算术/位运算/移位（`+ - * / % << >> >>> & | ^`）
    /// 与字符串拼接。操作数为字面量节点 id；返回折叠产物节点。
    fn fold_lit_bin(&mut self, _op: BinOp, _l: Self::Id, _r: Self::Id) -> Option<Self::Id> {
        None
    }

    /// 字面量比较折叠（如 `1 < 2`）。可比较的字面量种类与字符串比较
    /// 语义（引用比较 vs 值比较）由语言侧决定；返回布尔结果。
    fn fold_lit_cmp(&mut self, _op: BinOp, _l: Self::Id, _r: Self::Id) -> Option<bool> {
        None
    }

    /// 负数值字面量的取反（`-(-lit)` 外层负消除）。宽度溢出时必须
    /// 返回 None 拒绝折叠（如 Java i32 的 `-(-2147483648)`）。
    /// 默认实现按 i64 拒绝 `i64::MIN` 取反（防 debug panic / release 回绕）。
    fn fold_lit_neg(&mut self, v: i64, _wide: bool) -> Option<i64> {
        v.checked_neg()
    }

    /// 常量重结合合并：`(x op1 K1) op2 K2` 的常量合并结果。
    /// 语言侧负责宽度回绕（Java i32 的 `K1+K2` 溢出必须回绕，否则
    /// 产物是非法字面量）。
    fn reassoc_delta(
        &mut self,
        _op1: BinOp,
        _k1: i64,
        _op2: BinOp,
        _k2: i64,
        _wide: bool,
    ) -> Option<ReassocOutcome<Self::Id>> {
        None
    }

    /// 每轮扫描前由引擎调用；语言侧在此刷新缓存（效果表、变量类型表等）。
    /// `root` 是本轮简化的根（通常是方法体），语言侧可据此做作用域分析。
    fn prepare(&mut self, _root: Self::Id) {}

    /// 节点成本权重（用于“改写必须严格降本”的门槛）。
    fn cost_weight(&self, kind: NodeKind) -> u64 {
        crate::cost::default_weight(kind)
    }
}

/// [`Lang::reassoc_delta`] 的合并结果。
#[derive(Clone, Copy, Debug)]
pub enum ReassocOutcome<Id> {
    /// 合并为中性元（`(x+K1)-K1` 类）→ 直接替换为 `x`。
    Neutral,
    /// 合并后的常量字面量节点 + 建议算符（`x op lit`；负常量由语言侧
    /// 决定用 `Sub` 正字面量还是 `Add` 负字面量表达）。
    Lit(Id, BinOp),
}
