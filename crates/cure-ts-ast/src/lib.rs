//! # cure-ts-ast
//!
//! TypeScript/JavaScript AST（arena + TsId）+ 模块/类/函数结构 + builder
//! + [`cure_engine::Lang`] 实现。
//!
//! 设计原则（与 cure-java-ast 同构，词汇面向 JS）：
//! - **JS 值级语法**映射引擎词汇：箭头/函数表达式 → Lambda（params_raw），
//!   for-of/for-in → ForEach，`?.`/`??`/`**`/`in`/`instanceof`/typeof/
//!   await/delete/void → 引擎 BinOp/UnOp 扩展变体，`++`/`--` → Pre/Post
//!   Inc/Dec，逻辑赋值 `&&=`/`||=`/`??=` → Assign{op:Some}（复合赋值
//!   读语义恰好覆盖「读+条件写」的保守面）。
//! - **TS/JS 特有构造**走 NodeData 专属变体 + `NodeKind::Custom(u16)`
//!   （引擎规则永不触碰；区域事件/作用域扫描正常下钻孩子）：模板
//!   字面量、对象字面量、展开、解构模式、TS as/satisfies/非空断言、
//!   嵌套函数声明。
//! - **TS 类型系统**（注解/泛型/装饰器/接口/枚举/命名空间）以**原文**
//!   字符串保真（ty_raw/sig_raw/head_raw），与 Java mods/throws 同策略
//!   ——最新语法靠原始保真，值级简化不受影响。
//! - **JS 语义守卫**内建：`var` 提升陷阱（函数级作用域 × 块级扫描的
//!   裂缝——prepare 计算块外引用并标记 is_opaque）、可选链不透明、
//!   NaN/undefined/Infinity 是全局名而非字面量。
//!
//! 零外部依赖（仅 cure-engine / cure-tree）。
//!
//! arena 不变量：children 永远先于父节点入 arena（index 递增）。

use std::collections::{HashMap, HashSet};

use cure_engine::analysis::{subtree_metrics, TreeMetrics};
use cure_engine::lang::{ReassocOutcome, StmtKill};
use cure_engine::Lang;
pub use cure_tree::ChildList;
use cure_tree::{EventStore, NameTable};

pub use cure_engine::kind::{BinOp, LitRef, NodeKind, UnOp};
pub use cure_engine::Effect;

// ---------------------------------------------------------------------------
// 基础类型
// ---------------------------------------------------------------------------

/// arena 句柄。
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct TsId(pub u32);
impl Default for TsId {
    fn default() -> Self {
        TsId(u32::MAX)
    }
}

/// 声明种类（JS 三种 + TS using/await using 的字面量保真）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VarKind {
    Var,
    Let,
    Const,
    /// `using`/`await using`（显式资源管理）：块尾 dispose 语义，
    /// 声明不可删除/搬移（dispose 时机改变）。
    Using,
}

/// 绑定种类（resolver 事实层）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingKind {
    Var,
    Let,
    Const,
    Using,
    Param,
    CatchParam,
    Function,
    /// 具名函数表达式的内部名字：不可变绑定（sloppy 赋值静默忽略，
    /// strict 抛错）——不能当普通局部变量做写删/传播。
    FunctionExprName,
}

/// 一个词法绑定（resolver 产出）。`decl` 指向声明节点；参数/模式绑定的
/// `decl` 指向函数/循环节点。`module_level` 表示绑定在脚本/模块顶层
/// （跨根不可见，规则不得当局部变量传播）。
#[derive(Clone, Debug)]
pub struct Binding {
    pub name: String,
    pub kind: BindingKind,
    pub decl: Option<TsId>,
    /// 解析到该绑定的 VarRef 数（含读写）。
    pub ref_count: u32,
    /// 写次数（赋值目标/++--/裸 for-in）。
    pub write_count: u32,
    /// 被嵌套函数按引用捕获（延迟读）。
    pub captured: bool,
    /// 顶层绑定。
    pub module_level: bool,
}

/// 数值字面量的解析值。
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NumVal {
    Int(i64),
    Double(f64),
}

/// JS 字面量。
#[derive(Clone, PartialEq, Debug)]
pub enum JsLit {
    Bool(bool),
    Int(i64),
    Double(f64),
    /// 保原文的数值字面量（0x1F / 1_000 / 1.5e2 / 12n 之外的 number 形态）。
    NumRaw { text: String, val: NumVal },
    /// BigInt 原文（含 n 后缀）。值域无界 → 不折叠，仅保真。
    BigInt(String),
    Str(String),
    /// 正则字面量原文（含 flags）。
    Regex(String),
    Null,
}

/// 符号（intern 名字 id）——与 Java 后端同构（高重复率名字的零分配 clone）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Sym(pub u32);

mod resolve;

// Custom(u16) 判别值注册表（本语言专属、引擎规则不触碰的构造）。
// u16 值语言侧自定；引擎只做相等性比较。
pub mod custom {
    /// 模板字面量 `` `a${x}b` ``（tagged 时孩子 0 为 tag）。
    pub const TEMPLATE: u16 = 1;
    /// 展开运算符 `...expr`（实参/数组字面量/调用位）。
    pub const SPREAD: u16 = 2;
    /// 解构模式位（声明/赋值目标 `const {a} = o` 的 target 侧）。
    pub const DESTRUCT: u16 = 3;
    /// 对象字面量 `{ a: 1, b, ...rest }`。
    pub const OBJ_LIT: u16 = 4;
    /// TS `expr as T`。
    pub const TS_AS: u16 = 5;
    /// TS `expr satisfies T`。
    pub const TS_SATISFIES: u16 = 6;
    /// TS 非空断言 `expr!`。
    pub const TS_NON_NULL: u16 = 7;
    /// 嵌套函数声明语句 `function f(…) { … }`。
    pub const FUNC_STMT: u16 = 8;
    /// 类表达式（原文保真——v1 不简化类表达式体）。
    pub const CLASS_EXPR: u16 = 9;
    /// 解构声明/赋值语句 `const {a} = o` / `[x, y] = arr`。
    pub const DESTRUCT_STMT: u16 = 10;
    /// 逗号序列表达式 `a, b, c`。
    pub const SEQ: u16 = 11;
}

/// 节点负载：children 统一放 [`Node::children`]，这里只放非节点数据。
#[derive(Clone, PartialEq, Debug)]
pub enum NodeData {
    // ---- 语句 ----
    Block,
    /// 合成分组（无花括号、无作用域）：if/for 头多语句的承载。
    /// **模块语句合成根**（simplify 门面构建）：顶层语句统一拼成一条
    /// 语句列表使引擎规则获得完整父链（if-true 解包等跨语句规则）。
    /// 模块级名字**不绑定**（跨根读不可见——保守非局部）；直接
    /// VarDecl 孩子标记不透明（死存/传播锚定由 const 表规则接管）。
    ModuleBlock,
    Group,
    Empty,
    ExprStmt,
    /// `let/const/var name (: ty)? = init?`；export 修饰原文并入 kind_raw。
    /// children: `[init]?`
    VarDecl { name: String, kind: VarKind, ty_raw: Option<String> },
    /// `target op? value`；op=Some(And/Or/Nullish) 表示逻辑赋值
    /// &&=/||=/??=（复合赋值读语义覆盖「读目标+条件写」）。
    Assign { op: Option<BinOp> },
    If,
    While,
    DoWhile,
    /// children: `[init…, (cond)?, step…, body]`（inits/has_cond/steps 计数）。
    For { inits: u8, has_cond: bool, steps: u8 },
    /// for (const x of it) / for (x in obj)。decl=None 表示裸名形态；
    /// await=true 为 for-await-of。
    /// children: `[iterable, body]`
    ForEach { name: String, of: bool, decl: Option<VarKind>, ty_raw: Option<String>, await_: bool },
    Return,
    Break { label: Option<String> },
    Continue { label: Option<String> },
    Throw,
    /// children: `[block, catch…, (finally)?]`
    Try,
    /// `catch (param_raw? )`；TS `catch (e: unknown)` 整体原文。
    /// children: `[block]`
    Catch { param_raw: Option<String> },
    /// children: `[subject, case…]`
    Switch,
    /// children: `[label…, stmts…]`（前 labels 个是 case 表达式）
    Case { labels: u16, is_default: bool, arrow: bool },
    /// children: `[stmt]`
    Label { name: String },
    /// 不可解析/不建模的原文区域：原样保留，规则不触碰。
    Raw { text: String },

    // ---- 表达式 ----
    Binary { op: BinOp },
    Unary { op: UnOp },
    /// children: `[callee, arg…]`；callee 为 Custom(TEMPLATE) 时是标签模板。
    /// optional 元素（`a?.(…)`）参与链式短路 → is_opaque。
    Call { optional: bool },
    /// `obj.name` / `obj?.name`。children: `[object]`。
    /// optional 元素参与链式短路 → is_opaque（保守区域拒绝）。
    Member { name: Sym, optional: bool },
    /// `arr[index]` / `arr?.[index]`。children: `[array, index]`。
    Index { optional: bool },
    Paren,
    Ternary,
    /// `new Callee(args…)`。children: `[callee, arg…]`（callee 为表达式）。
    New,
    ArrayLit,
    VarRef { name: Sym },
    Literal(JsLit),
    This,
    /// 函数表达式 / 箭头函数。children: `[body]`（Block 或表达式）。
    /// params_raw 保真（含 TS 类型注解/模式参数原文）。
    Lambda { params_raw: String, name: String, generics_raw: String, space: bool, arrow: bool, is_async: bool, is_generator: bool },
    /// 模板字面量：quasis 长度 = 孩子数+1（tagged 时孩子 0 是 tag）。
    Template { quasis: Vec<String>, tag: bool },
    /// `...expr`。children: `[expr]`
    Spread,
    /// 解构模式（声明/赋值目标位）。children: `[value]`。
    /// 模式原文含绑定名（事件系统盲区）→ is_opaque。
    Destruct { pattern_raw: String },
    /// 对象字面量。children 与 props 对齐（值表达式；props 含键原文）。
    ObjLit { props: Vec<String> },
    /// `expr as T`。children: `[expr]`
    TsAs { ty_raw: String },
    /// `expr satisfies T`。children: `[expr]`
    TsSatisfies { ty_raw: String },
    /// `expr!`。children: `[expr]`
    TsNonNull,
    /// 嵌套函数声明语句。children: `[body]`（Block）。params_raw 为
    /// 圆括号内的参数原文（作用域分析用）。
    FuncStmt { sig_raw: String, name: String, params_raw: String },
    /// 类表达式（原文）。children: `[]`
    ClassExpr { raw: String },
    /// 解构声明/赋值语句。children: `[value]`。kw 为 var/let/const（赋值
    /// 形态为空串）。模式绑定名在原文 → is_opaque。
    DestructStmt { kw: String, pattern: String },
    /// 逗号序列表达式。children = 各元素。
    Seq,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Node {
    pub data: NodeData,
    pub children: ChildList<TsId>,
}

// ---------------------------------------------------------------------------
// 外部事实（front-end facts）——TS 载体（结构与 Java 版同构）。
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VarFacts {
    pub single_def: bool,
    pub single_use: bool,
    pub bool_valued: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TsFacts {
    pub vars: HashMap<String, VarFacts>,
    pub call_effects: HashMap<String, HashSet<cure_engine::Effect>>,
}

impl TsFacts {
    pub fn var(&self, name: &str) -> VarFacts {
        self.vars.get(name).copied().unwrap_or_default()
    }
}

/// Per-statement 名字事实（与 Java 版同构；键 = intern id）。
#[derive(Clone, Copy, Debug)]
struct StmtFact {
    key: u32,
    refs: u32,
    lambda_cap: bool,
    shadow: bool,
}

/// Per-statement kill 事件（1=Read 2=Shadow 3=DomWrite）。
#[derive(Clone, Copy, Debug)]
struct StmtKillFact {
    key: u32,
    kind: u8,
    assign: TsId,
}

#[derive(Clone, Copy, Debug)]
struct StmtOpq {
    opaque: bool,
}

// ---------------------------------------------------------------------------
// 模块结构（签名层：签名保真、函数体进 arena）
// ---------------------------------------------------------------------------

/// 顶层项。
#[derive(Clone, PartialEq, Debug)]
pub enum TopItem {
    /// 普通语句（含 import/export 语句的值级部分？否——import/export 走原文）。
    Stmt(TsId),
    /// 顶层函数声明（export/default 修饰并入 sig_raw）。
    Func { sig_raw: String, name: String, body: Option<TsId> },
    /// 类/接口/枚举/命名空间声明。
    Class(ClassDecl),
    /// import 语句原文（不建模——与外部模块的绑定关系不可分析）。
    Import(String),
    /// export 语句原文（export * from / export = / export default <非函数/类> 等）。
    ExportRaw(String),
    /// 不可解析的原文区域。
    Raw(String),
    /// `export <语句>`（const/let/var/解构声明——值级照常简化，
    /// 打印加 export 前缀）。函数/类导出走 Func/Class 的 sig 前缀。
    ExportStmt(TsId),
}

/// 类形态声明（class/interface/enum/namespace——head_raw 保真全部头部）。
#[derive(Clone, PartialEq, Debug)]
pub struct ClassDecl {
    /// class 关键字**之前**的原文（装饰器 / `export` / `export default` /
    /// `abstract`）。
    pub prefix: String,
    /// class 关键字与名字**之后**的头部原文（类型参数 / extends /
    /// implements）。
    pub head_raw: String,
    /// class/interface/enum/namespace 关键字本身（打印用）。
    pub kind_kw: String,
    pub name: String,
    pub members: Vec<ClassMember>,
}

/// 类成员。
#[derive(Clone, PartialEq, Debug)]
pub enum ClassMember {
    /// 方法/构造器/重载签名。sig_raw 保真（修饰符/装饰器/泛型/参数/
    /// 返回类型）；body=None 表示抽象/重载声明。
    Method { sig_raw: String, body: Option<TsId> },
    /// 字段声明（sig_raw 含 `static readonly #x: T` 等全部前缀）。
    Field { sig_raw: String, name: String, init: Option<TsId> },
    /// `static { … }` 静态块。
    StaticBlock { body: TsId },
    /// 嵌套类型/索引签名/其他不建模成员（原文）。
    Raw(String),
}

/// 模块单元。
#[derive(Clone, PartialEq, Debug, Default)]
pub struct ModuleUnit {
    pub shebang: String,
    pub items: Vec<TopItem>,
}

/// 解析错误（1-based 行/列）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

// ---------------------------------------------------------------------------
// TsAst 主体
// ---------------------------------------------------------------------------

#[derive(Default, Debug)]
pub struct TsAst {
    pub(crate) nodes: Vec<Node>,
    /// 外部事实（前端注入；None = 纯源码保守模式）。
    pub facts: Option<TsFacts>,
    /// 模块级常量表（collect_unit_consts 填充）：名字 → init 节点。
    /// 收集条件：零写、零遮蔽、同名唯一、init 纯净（字面量/字面量
    /// 数组/先前常量的纯表达式链——混淆器拷贝链的核心武器）。
    pub const_fields: HashMap<String, TsId>,
    /// 可安全做 VarRef → 字面量替换的常量名（标量字面量 init）。
    pub const_scalars: HashSet<String>,
    /// 可内联的模块级单 return 函数：名字 → (参数名表, 返回表达式)。
    /// 常量解密 helper（字符串数组访问器）的调用点展开材料。
    pub inline_funcs: HashMap<String, (Vec<String>, TsId)>,
    /// prepare() 重建：TsId → 聚合效果（稠密槽）。增量维护：编辑只把
    /// 受影响祖先标脏，prepare 仅重算脏子树。
    effect_cache: Vec<Option<Effect>>,
    /// 脏 effect 节点（去重标记 + 列表）。
    effect_dirty: Vec<TsId>,
    effect_dirty_mark: Vec<bool>,
    /// effect_cache 上次完整覆盖的根（换根时补算可达节点）。
    effects_root: Option<TsId>,
    /// prepare() 重建：VarRef → 是否解析到**局部绑定**（作用域栈解析）。
    /// JS 无声明类型 → var_types 退化为局部性布尔（is_local_var 的依据）。
    local_marks: Vec<bool>,
    /// 当前根（函数体）的参数名（作用域分析种子）。含 TS 注解剥除后的
    /// 裸名；模式参数（解构）不进种子（保守：其中的名字不判局部）。
    param_scope: Vec<String>,
    /// var 提升陷阱标记（prepare 计算）：块内 `var` 声明的名字在**本根
    /// 内其块子树之外**仍被引用（或同名再声明）→ 该声明节点 is_opaque。
    /// JS var 是函数级作用域；引擎规则按块扫描，块外引用不可见——
    /// 不标记则删除/传播会静默改变提升语义（块外读到 undefined）。
    var_trap: Vec<bool>,
    /// 合成 Group 的直接 VarDecl 孩子（见 is_opaque 注释）。
    group_decls: Vec<bool>,
    /// resolver 产出：全部绑定事实（每次 prepare 重建）。
    bindings: Vec<Binding>,
    /// 节点 → 解析到的绑定（VarRef 与声明节点；未解析/非名字节点 None）。
    binding_of: Vec<Option<u32>>,
    /// 根内被**改写**的名字（赋值目标/++--/裸 for-in 绑定）：局部传播
    /// 评估「参数值在声明与使用之间是否稳定」。
    mutated_names: std::collections::HashSet<String>,
    /// 直接 eval 污染区（prepare 计算）：含 `eval(...)` 调用的**函数**其
    /// 全部后代（形参默认值、函数体、嵌套闭包）标记为不可触碰——直接
    /// eval 的 `var` 声明进入调用方函数作用域，静态作用域分析完全失效
    /// （Test262 scope-param-elem-var-close / scope-body-lex-distinct 抓获：
    /// 常量被传播成外层值、`let x;` 声明被当死代码删除）。顶层 eval 则
    /// 整棵 AST 污染（可向脚本/全局注入绑定）。
    eval_taint: Vec<bool>,
    /// 被 lambda/嵌套函数**按引用捕获**的局部名（根级集合）：写杀/传播
    /// 的扫描窗口看不见「λ 创建于窗口之前、调用在之后」的延迟读
    ///（node 差分 closure 抓获：`x = 2` 曾被 store_kill 误杀——闭包
    /// 在调用点读到旧值）。捕获名一律判非局部（保守拒绝触碰）。
    captured_names: std::collections::HashSet<String>,
    /// 名字 intern 表 + 节点名字键（NameKey = u32）。
    names: NameTable,
    /// 区域事件索引（B3 惰性重建）。
    events: EventStore<TsId, u32>,
    stmt_facts: Vec<Option<Box<[StmtFact]>>>,
    stmt_kills: Vec<Option<Box<[StmtKillFact]>>>,
    stmt_opq: Vec<Option<StmtOpq>>,
    prepared_root: Option<TsId>,
}

impl TsAst {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clone_node(&mut self, id: TsId, children: Vec<TsId>) -> TsId {
        let data = self.nodes[id.0 as usize].data.clone();
        self.push(data, children)
    }

    fn push(&mut self, data: NodeData, children: Vec<TsId>) -> TsId {
        let id = TsId(self.nodes.len() as u32);
        self.nodes.push(Node { data, children: ChildList::from_vec(children) });
        id
    }

    pub fn node(&self, id: TsId) -> &Node {
        &self.nodes[id.0 as usize]
    }
    pub fn data(&self, id: TsId) -> &NodeData {
        &self.node(id).data
    }

    /// 固有 kind（与 Lang trait 同名；供不依赖引擎的下游使用，如 printer）。
    pub fn kind(&self, id: TsId) -> NodeKind {
        match &self.nodes[id.0 as usize].data {
            NodeData::Block | NodeData::Group | NodeData::ModuleBlock => NodeKind::Block,
            NodeData::Empty => NodeKind::Empty,
            NodeData::ExprStmt => NodeKind::ExprStmt,
            NodeData::VarDecl { .. } => NodeKind::VarDecl,
            NodeData::Assign { .. } => NodeKind::Assign,
            NodeData::If => NodeKind::If,
            NodeData::While => NodeKind::While,
            NodeData::DoWhile => NodeKind::DoWhile,
            NodeData::For { .. } => NodeKind::For,
            NodeData::ForEach { .. } => NodeKind::ForEach,
            NodeData::Return => NodeKind::Return,
            NodeData::Break { .. } => NodeKind::Break,
            NodeData::Continue { .. } => NodeKind::Continue,
            NodeData::Throw => NodeKind::Throw,
            NodeData::Try => NodeKind::Try,
            NodeData::Catch { .. } => NodeKind::Catch,
            NodeData::Switch => NodeKind::Switch,
            NodeData::Case { .. } => NodeKind::Case,
            NodeData::Label { .. } => NodeKind::Label,
            NodeData::Raw { .. } => NodeKind::Raw,
            NodeData::Binary { .. } => NodeKind::Binary,
            NodeData::Unary { .. } => NodeKind::Unary,
            NodeData::Call { .. } => NodeKind::Call,
            NodeData::Member { .. } => NodeKind::Member,
            NodeData::Index { .. } => NodeKind::Index,
            NodeData::Paren => NodeKind::Paren,
            NodeData::Ternary => NodeKind::Ternary,
            NodeData::New => NodeKind::New,
            NodeData::ArrayLit => NodeKind::ArrayLit,
            NodeData::VarRef { .. } => NodeKind::VarRef,
            NodeData::Literal(_) => NodeKind::Literal,
            NodeData::This => NodeKind::This,
            NodeData::Lambda { .. } => NodeKind::Lambda,
            NodeData::Template { .. } => NodeKind::Custom(custom::TEMPLATE),
            NodeData::Spread => NodeKind::Custom(custom::SPREAD),
            NodeData::Destruct { .. } => NodeKind::Custom(custom::DESTRUCT),
            NodeData::ObjLit { .. } => NodeKind::Custom(custom::OBJ_LIT),
            NodeData::TsAs { .. } => NodeKind::Custom(custom::TS_AS),
            NodeData::TsSatisfies { .. } => NodeKind::Custom(custom::TS_SATISFIES),
            NodeData::TsNonNull => NodeKind::Custom(custom::TS_NON_NULL),
            NodeData::FuncStmt { .. } => NodeKind::Custom(custom::FUNC_STMT),
            NodeData::ClassExpr { .. } => NodeKind::Custom(custom::CLASS_EXPR),
            NodeData::DestructStmt { .. } => NodeKind::Custom(custom::DESTRUCT_STMT),
            NodeData::Seq => NodeKind::Custom(custom::SEQ),
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn children(&self, id: TsId) -> &[TsId] {
        self.nodes[id.0 as usize].children.as_slice()
    }

    /// 固有 set_child（供不依赖引擎 trait 的下游使用）。
    pub fn set_child(&mut self, parent: TsId, index: usize, new: TsId) {
        self.nodes[parent.0 as usize].children.set(index, new);
    }

    // ---- builder：语句 ----

    pub fn block(&mut self, stmts: Vec<TsId>) -> TsId {
        self.push(NodeData::Block, stmts)
    }
    pub fn group(&mut self, stmts: Vec<TsId>) -> TsId {
        self.push(NodeData::Group, stmts)
    }

    /// 模块语句合成根。
    pub fn module_block(&mut self, stmts: Vec<TsId>) -> TsId {
        self.push(NodeData::ModuleBlock, stmts)
    }
    pub fn empty(&mut self) -> TsId {
        self.push(NodeData::Empty, vec![])
    }
    pub fn expr_stmt(&mut self, e: TsId) -> TsId {
        self.push(NodeData::ExprStmt, vec![e])
    }
    pub fn var_decl(
        &mut self,
        name: &str,
        kind: VarKind,
        ty_raw: Option<&str>,
        init: Option<TsId>,
    ) -> TsId {
        self.push(
            NodeData::VarDecl {
                name: name.into(),
                kind,
                ty_raw: ty_raw.map(Into::into),
            },
            init.into_iter().collect(),
        )
    }
    pub fn assign(&mut self, target: TsId, value: TsId) -> TsId {
        self.push(NodeData::Assign { op: None }, vec![target, value])
    }
    pub fn assign_op(&mut self, op: BinOp, target: TsId, value: TsId) -> TsId {
        self.push(NodeData::Assign { op: Some(op) }, vec![target, value])
    }
    pub fn if_(&mut self, cond: TsId, then: TsId, els: Option<TsId>) -> TsId {
        let mut ch = vec![cond, then];
        ch.extend(els);
        self.push(NodeData::If, ch)
    }
    pub fn while_(&mut self, cond: TsId, body: TsId) -> TsId {
        self.push(NodeData::While, vec![cond, body])
    }
    pub fn do_while(&mut self, body: TsId, cond: TsId) -> TsId {
        self.push(NodeData::DoWhile, vec![body, cond])
    }
    pub fn for_(
        &mut self,
        inits: Vec<TsId>,
        cond: Option<TsId>,
        steps: Vec<TsId>,
        body: TsId,
    ) -> TsId {
        let ni = inits.len();
        let ns = steps.len();
        let mut ch = inits;
        let has_cond = cond.is_some();
        ch.extend(cond);
        ch.extend(steps);
        ch.push(body);
        self.push(
            NodeData::For {
                inits: ni as u8,
                has_cond,
                steps: ns as u8,
            },
            ch,
        )
    }
    pub fn for_each(
        &mut self,
        name: &str,
        of: bool,
        decl: Option<VarKind>,
        ty_raw: Option<&str>,
        iterable: TsId,
        body: TsId,
        await_: bool,
    ) -> TsId {
        self.push(
            NodeData::ForEach {
                name: name.into(),
                of,
                decl,
                ty_raw: ty_raw.map(Into::into),
                await_,
            },
            vec![iterable, body],
        )
    }
    pub fn ret(&mut self, value: Option<TsId>) -> TsId {
        self.push(NodeData::Return, value.into_iter().collect())
    }
    pub fn break_(&mut self, label: Option<&str>) -> TsId {
        self.push(
            NodeData::Break { label: label.map(Into::into) },
            vec![],
        )
    }
    pub fn continue_(&mut self, label: Option<&str>) -> TsId {
        self.push(
            NodeData::Continue { label: label.map(Into::into) },
            vec![],
        )
    }
    pub fn throw(&mut self, e: TsId) -> TsId {
        self.push(NodeData::Throw, vec![e])
    }
    pub fn try_(
        &mut self,
        try_block: TsId,
        catches: Vec<TsId>,
        finally: Option<TsId>,
    ) -> TsId {
        let mut ch = vec![try_block];
        ch.extend(catches);
        ch.extend(finally);
        self.push(NodeData::Try, ch)
    }
    pub fn catch_(&mut self, param_raw: Option<&str>, block: TsId) -> TsId {
        self.push(
            NodeData::Catch { param_raw: param_raw.map(Into::into) },
            vec![block],
        )
    }
    pub fn switch_(&mut self, subject: TsId, cases: Vec<TsId>) -> TsId {
        let mut ch = vec![subject];
        ch.extend(cases);
        self.push(NodeData::Switch, ch)
    }
    pub fn case_(
        &mut self,
        labels: Vec<TsId>,
        is_default: bool,
        arrow: bool,
        stmts: Vec<TsId>,
    ) -> TsId {
        let n = labels.len() as u16;
        let mut ch = labels;
        ch.extend(stmts);
        self.push(
            NodeData::Case {
                labels: n,
                is_default,
                arrow,
            },
            ch,
        )
    }
    pub fn label(&mut self, name: &str, stmt: TsId) -> TsId {
        self.push(NodeData::Label { name: name.into() }, vec![stmt])
    }
    pub fn raw(&mut self, text: &str) -> TsId {
        self.push(NodeData::Raw { text: text.into() }, vec![])
    }

    // ---- builder：表达式 ----

    pub fn bin(&mut self, op: BinOp, l: TsId, r: TsId) -> TsId {
        self.push(NodeData::Binary { op }, vec![l, r])
    }
    pub fn un(&mut self, op: UnOp, e: TsId) -> TsId {
        self.push(NodeData::Unary { op }, vec![e])
    }
    pub fn call(&mut self, callee: TsId, args: Vec<TsId>) -> TsId {
        let mut ch = vec![callee];
        ch.extend(args);
        self.push(NodeData::Call { optional: false }, ch)
    }

    /// 可选调用 `callee?.(args)`。
    pub fn opt_call(&mut self, callee: TsId, args: Vec<TsId>) -> TsId {
        let mut ch = vec![callee];
        ch.extend(args);
        self.push(NodeData::Call { optional: true }, ch)
    }

    /// 解构声明/赋值语句。kw: "var"/"let"/"const"（赋值形态空串）。
    pub fn destruct_stmt(&mut self, kw: &str, pattern: &str, value: TsId) -> TsId {
        self.push(
            NodeData::DestructStmt {
                kw: kw.into(),
                pattern: pattern.into(),
            },
            vec![value],
        )
    }

    /// 逗号序列表达式。
    pub fn seq(&mut self, elems: Vec<TsId>) -> TsId {
        self.push(NodeData::Seq, elems)
    }
    pub fn member(&mut self, obj: TsId, name: &str, optional: bool) -> TsId {
        let sym = self.intern_sym(name);
        self.push(NodeData::Member { name: sym, optional }, vec![obj])
    }
    pub fn index(&mut self, arr: TsId, idx: TsId, optional: bool) -> TsId {
        self.push(NodeData::Index { optional }, vec![arr, idx])
    }
    pub fn paren(&mut self, e: TsId) -> TsId {
        self.push(NodeData::Paren, vec![e])
    }
    pub fn ternary(&mut self, c: TsId, a: TsId, b: TsId) -> TsId {
        self.push(NodeData::Ternary, vec![c, a, b])
    }
    pub fn new_(&mut self, callee: TsId, args: Vec<TsId>) -> TsId {
        let mut ch = vec![callee];
        ch.extend(args);
        self.push(NodeData::New, ch)
    }
    pub fn array_lit(&mut self, elems: Vec<TsId>) -> TsId {
        self.push(NodeData::ArrayLit, elems)
    }
    pub fn var(&mut self, name: &str) -> TsId {
        let sym = self.intern_sym(name);
        self.push(NodeData::VarRef { name: sym }, vec![])
    }
    pub fn lit(&mut self, l: JsLit) -> TsId {
        self.push(NodeData::Literal(l), vec![])
    }

    /// Double 字面量节点（规则层折叠结果构造用）。
    pub fn clone_node_of_double(&mut self, v: f64) -> TsId {
        self.push(NodeData::Literal(JsLit::Double(v)), vec![])
    }
    pub fn this(&mut self) -> TsId {
        self.push(NodeData::This, vec![])
    }
    pub fn lambda(
        &mut self,
        params_raw: &str,
        body: TsId,
        arrow: bool,
        is_async: bool,
        is_generator: bool,
    ) -> TsId {
        self.lambda_named(params_raw, body, arrow, is_async, is_generator, "", false)
    }
    /// 具名函数表达式：名字是函数体内的只读绑定（`function f(){…f…}`），
    /// 打印必须保留（`fn.name` 与内部绑定语义；Test262 NFE 家族抓获）。
    pub fn lambda_named(
        &mut self,
        params_raw: &str,
        body: TsId,
        arrow: bool,
        is_async: bool,
        is_generator: bool,
        name: &str,
        space: bool,
    ) -> TsId {
        self.lambda_named_generic(
            "",
            params_raw,
            body,
            arrow,
            is_async,
            is_generator,
            name,
            space,
        )
    }

    /// 带 TS 类型参数（`<T extends …>`）的函数/箭头表达式。
    #[allow(clippy::too_many_arguments)]
    pub fn lambda_named_generic(
        &mut self,
        generics_raw: &str,
        params_raw: &str,
        body: TsId,
        arrow: bool,
        is_async: bool,
        is_generator: bool,
        name: &str,
        space: bool,
    ) -> TsId {
        self.push(
            NodeData::Lambda {
                params_raw: params_raw.into(),
                name: name.into(),
                generics_raw: generics_raw.into(),
                space,
                arrow,
                is_async,
                is_generator,
            },
            vec![body],
        )
    }

    pub fn lambda_generic(
        &mut self,
        generics_raw: &str,
        params_raw: &str,
        body: TsId,
        arrow: bool,
        is_async: bool,
        is_generator: bool,
    ) -> TsId {
        self.lambda_named_generic(
            generics_raw,
            params_raw,
            body,
            arrow,
            is_async,
            is_generator,
            "",
            false,
        )
    }
    /// 模板字面量。quasis.len() 必须 == children.len() + 1（无 tag）。
    pub fn template(&mut self, quasis: Vec<String>, interps: Vec<TsId>) -> TsId {
        self.push(NodeData::Template { quasis, tag: false }, interps)
    }
    /// 标签模板 `` tag`a${x}` ``。quasis.len() == children.len()（孩子 0 为 tag）。
    pub fn tagged_template(
        &mut self,
        tag: TsId,
        quasis: Vec<String>,
        interps: Vec<TsId>,
    ) -> TsId {
        let mut ch = vec![tag];
        ch.extend(interps);
        self.push(NodeData::Template { quasis, tag: true }, ch)
    }
    pub fn spread(&mut self, e: TsId) -> TsId {
        self.push(NodeData::Spread, vec![e])
    }
    pub fn destruct(&mut self, pattern_raw: &str, value: TsId) -> TsId {
        self.push(
            NodeData::Destruct { pattern_raw: pattern_raw.into() },
            vec![value],
        )
    }
    /// 对象字面量。props[i] 为键原文（含冒号后的空格约定由 printer 处理）；
    /// children[i] 为值。
    pub fn obj_lit(&mut self, props: Vec<String>, values: Vec<TsId>) -> TsId {
        debug_assert_eq!(props.len(), values.len());
        self.push(NodeData::ObjLit { props }, values)
    }
    pub fn ts_as(&mut self, ty_raw: &str, e: TsId) -> TsId {
        self.push(NodeData::TsAs { ty_raw: ty_raw.into() }, vec![e])
    }
    pub fn ts_satisfies(&mut self, ty_raw: &str, e: TsId) -> TsId {
        self.push(NodeData::TsSatisfies { ty_raw: ty_raw.into() }, vec![e])
    }
    pub fn ts_non_null(&mut self, e: TsId) -> TsId {
        self.push(NodeData::TsNonNull, vec![e])
    }
    /// 嵌套函数声明语句。
    pub fn func_stmt(&mut self, sig_raw: &str, name: &str, params_raw: &str, body: TsId) -> TsId {
        self.push(
            NodeData::FuncStmt {
                sig_raw: sig_raw.into(),
                name: name.into(),
                params_raw: params_raw.into(),
            },
            vec![body],
        )
    }
    pub fn class_expr(&mut self, raw: &str) -> TsId {
        self.push(NodeData::ClassExpr { raw: raw.into() }, vec![])
    }

    // ---- 便捷 ----

    /// `obj.name(args…)`。
    pub fn method_call(&mut self, obj: TsId, name: &str, args: Vec<TsId>) -> TsId {
        let m = self.member(obj, name, false);
        self.call(m, args)
    }
    /// 裸调用 `name(args…)`。
    pub fn plain_call(&mut self, name: &str, args: Vec<TsId>) -> TsId {
        let f = self.var(name);
        self.call(f, args)
    }

    /// 参数作用域种子注入（simplify 门面按体设置）。
    pub fn param_scope_set(&mut self, params: &[String]) {
        self.param_scope = params.to_vec();
    }
    /// 参数作用域清空。
    pub fn param_scope_clear(&mut self) {
        self.param_scope.clear();
    }

    /// intern 名字 → Sym。
    pub fn intern_sym(&mut self, name: &str) -> Sym {
        Sym(self.names.intern(name))
    }
    /// Sym → 名字原文。
    pub fn sn(&self, sym: Sym) -> &str {
        self.names.name(sym.0)
    }

    /// 结构等价（忽略 `Paren` 包装、字面量拼写与原文段空白）：
    /// 用于「解析 → 打印 → 再解析」的往返校验。**不用于语义判定**——
    /// 化简会改变结构，这里只回答“打印器有没有改变程序形态”。
    pub fn equiv(&self, a: TsId, other: &TsAst, b: TsId) -> bool {
        let mut stack = vec![(a, b)];
        while let Some((mut x, mut y)) = stack.pop() {
            x = self.unparen(x);
            y = other.unparen(y);
            // 语句容器：Group（多声明符合成）与 Block 在打印后形态不同
            // （`if (x) var a=1,b=2;` → 加花括号的两条语句），按展平后的
            // 语句序列比较。Block 只有全部为 VarDecl 时才可能与 Group
            // 等价——这里不额外检查：Group 本身只承载 var 声明。
            if is_stmt_container(self.data(x)) && is_stmt_container(other.data(y)) {
                let ca = self.flat_stmt_children(x);
                let cb = other.flat_stmt_children(y);
                if ca.len() != cb.len() {
                    return false;
                }
                for (u, v) in ca.into_iter().zip(cb) {
                    stack.push((u, v));
                }
                continue;
            }
            if !self.data_equiv(x, other, y) {
                if std::env::var("CURE_EQUIV_DEBUG").is_ok() {
                    eprintln!("[equiv] {:?} != {:?}", self.data(x), other.data(y));
                }
                return false;
            }
            let ca = self.children(x);
            let cb = other.children(y);
            if ca.len() != cb.len() {
                return false;
            }
            for (u, v) in ca.iter().zip(cb.iter()) {
                stack.push((*u, *v));
            }
        }
        true
    }

    /// 语句容器的直接孩子序列；Group 递归展开（无花括号分组）。
    fn flat_stmt_children(&self, id: TsId) -> Vec<TsId> {
        let mut out = Vec::new();
        for &c in self.children(id) {
            if matches!(self.data(c), NodeData::Group) {
                out.extend(self.flat_stmt_children(c));
            } else {
                out.push(c);
            }
        }
        out
    }

    fn unparen(&self, mut id: TsId) -> TsId {
        loop {
            if !matches!(self.data(id), NodeData::Paren) {
                return id;
            }
            let Some(&c) = self.children(id).first() else { return id };
            id = c;
        }
    }

    fn data_equiv(&self, a: TsId, other: &TsAst, b: TsId) -> bool {
        match (self.data(a), other.data(b)) {
            (NodeData::Literal(x), NodeData::Literal(y)) => lit_equiv(x, y),
            (NodeData::VarRef { name: x }, NodeData::VarRef { name: y }) => {
                self.sn(*x) == other.sn(*y)
            }
            (
                NodeData::Member { name: x, optional: ox },
                NodeData::Member { name: y, optional: oy },
            ) => ox == oy && self.sn(*x) == other.sn(*y),
            (
                NodeData::VarDecl { name: x, kind: kx, ty_raw: tx },
                NodeData::VarDecl { name: y, kind: ky, ty_raw: ty },
            ) => x == y && kx == ky && norm_opt(tx) == norm_opt(ty),
            (
                NodeData::ForEach { name: x, of: ox, decl: dx, ty_raw: tx, await_: ax },
                NodeData::ForEach { name: y, of: oy, decl: dy, ty_raw: ty, await_: ay },
            ) => x == y && ox == oy && dx == dy && ax == ay && norm_opt(tx) == norm_opt(ty),
            (NodeData::Catch { param_raw: x }, NodeData::Catch { param_raw: y }) => {
                norm_opt(x) == norm_opt(y)
            }
            (NodeData::Break { label: x }, NodeData::Break { label: y })
            | (NodeData::Continue { label: x }, NodeData::Continue { label: y }) => x == y,
            (NodeData::Raw { text: x }, NodeData::Raw { text: y }) => {
                norm_code(x) == norm_code(y)
            }
            // `(yield)` / `yield;`：无操作数 yield 在解析器里有时走 Raw
            // （换行/无操作数形态）、有时走 VarRef（分号形态）。两者
            // 打印文本相同，生成器语境外的 token 流也相同——作为同一
            // 源形态比较（仅限 yield，避免泛化掩盖真实漂移）。
            (NodeData::Raw { text }, NodeData::VarRef { name }) => {
                text.trim() == "yield" && other.sn(*name) == "yield"
            }
            (NodeData::VarRef { name }, NodeData::Raw { text }) => {
                text.trim() == "yield" && self.sn(*name) == "yield"
            }
            (
                NodeData::Lambda {
                    params_raw: px,
                    name: nx,
                    generics_raw: gex,
                    space: sx,
                    arrow: ax,
                    is_async: asyncx,
                    is_generator: gx,
                },
                NodeData::Lambda {
                    params_raw: py,
                    name: ny,
                    generics_raw: gey,
                    space: sy,
                    arrow: ay,
                    is_async: asyncy,
                    is_generator: gy,
                },
            ) => {
                nx == ny
                    && sx == sy
                    && ax == ay
                    && asyncx == asyncy
                    && gx == gy
                    && norm_code(px) == norm_code(py)
                    && norm_code(gex) == norm_code(gey)
            }
            (
                NodeData::Template { quasis: qx, tag: tx },
                NodeData::Template { quasis: qy, tag: ty },
            ) => tx == ty && qx.len() == qy.len()
                && qx.iter().zip(qy).all(|(a, b)| norm_code(a) == norm_code(b)),
            (NodeData::Destruct { pattern_raw: x }, NodeData::Destruct { pattern_raw: y }) => {
                norm_code(x) == norm_code(y)
            }
            (NodeData::ObjLit { props: x }, NodeData::ObjLit { props: y }) => {
                x.len() == y.len()
                    && x.iter().zip(y).all(|(a, b)| norm_code(a) == norm_code(b))
            }
            (NodeData::TsAs { ty_raw: x }, NodeData::TsAs { ty_raw: y })
            | (NodeData::TsSatisfies { ty_raw: x }, NodeData::TsSatisfies { ty_raw: y }) => {
                norm_code(x) == norm_code(y)
            }
            (
                NodeData::FuncStmt { sig_raw: sx, name: nx, params_raw: px },
                NodeData::FuncStmt { sig_raw: sy, name: ny, params_raw: py },
            ) => nx == ny && norm_code(sx) == norm_code(sy) && norm_code(px) == norm_code(py),
            (NodeData::ClassExpr { raw: x }, NodeData::ClassExpr { raw: y }) => {
                norm_code(x) == norm_code(y)
            }
            (
                NodeData::DestructStmt { kw: kx, pattern: px },
                NodeData::DestructStmt { kw: ky, pattern: py },
            ) => kx == ky && norm_code(px) == norm_code(py),
            _ => self.data(a) == other.data(b),
        }
    }

    /// 模块单元等价：语句/函数/类逐项比较；import/export 原文按归一化
    /// 文本比较。顶层语句顺序必须一致（打印器不得重排）。
    pub fn unit_equiv(&self, ua: &ModuleUnit, other: &TsAst, ub: &ModuleUnit) -> bool {
        if ua.shebang != ub.shebang {
            return false;
        }
        let mut fa = Vec::new();
        self.flat_items(&ua.items, &mut fa);
        let mut fb = Vec::new();
        other.flat_items(&ub.items, &mut fb);
        if fa.len() != fb.len() {
            return false;
        }
        fa.iter().zip(&fb).all(|(a, b)| match (a, b) {
            (FlatTop::Stmt { export: ea, id: x }, FlatTop::Stmt { export: eb, id: y }) => {
                ea == eb && self.equiv(*x, other, *y)
            }
            (FlatTop::Item(x), FlatTop::Item(y)) => self.item_equiv(x, other, y),
            _ => false,
        })
    }

    /// 顶层项展平：`export var a=1,b=2`（ExportStmt(Group)）与打印后的
    /// 多条 `export var …` 对齐。
    fn flat_items<'a>(&self, items: &'a [TopItem], out: &mut Vec<FlatTop<'a>>) {
        for it in items {
            match it {
                TopItem::Stmt(s) => self.push_flat_item(*s, false, out),
                TopItem::ExportStmt(s) => self.push_flat_item(*s, true, out),
                _ => out.push(FlatTop::Item(it)),
            }
        }
    }

    fn push_flat_item<'a>(&self, id: TsId, export: bool, out: &mut Vec<FlatTop<'a>>) {
        if matches!(self.data(id), NodeData::Group) {
            for &c in self.children(id) {
                self.push_flat_item(c, export, out);
            }
        } else {
            out.push(FlatTop::Stmt { export, id });
        }
    }

    fn item_equiv(&self, a: &TopItem, other: &TsAst, b: &TopItem) -> bool {
        match (a, b) {
            (TopItem::Stmt(x), TopItem::Stmt(y))
            | (TopItem::ExportStmt(x), TopItem::ExportStmt(y)) => self.equiv(*x, other, *y),
            (
                TopItem::Func { sig_raw: sx, name: nx, body: bx },
                TopItem::Func { sig_raw: sy, name: ny, body: by },
            ) => {
                nx == ny
                    && norm_code(sx) == norm_code(sy)
                    && match (bx, by) {
                        (None, None) => true,
                        (Some(x), Some(y)) => self.equiv(*x, other, *y),
                        _ => false,
                    }
            }
            (TopItem::Class(x), TopItem::Class(y)) => self.class_equiv(x, other, y),
            (TopItem::Import(x), TopItem::Import(y))
            | (TopItem::ExportRaw(x), TopItem::ExportRaw(y))
            | (TopItem::Raw(x), TopItem::Raw(y)) => norm_code(x) == norm_code(y),
            _ => false,
        }
    }

    fn class_equiv(&self, a: &ClassDecl, other: &TsAst, b: &ClassDecl) -> bool {
        if norm_code(&a.prefix) != norm_code(&b.prefix)
            || norm_code(&a.head_raw) != norm_code(&b.head_raw)
            || a.kind_kw != b.kind_kw
            || a.name != b.name
            || a.members.len() != b.members.len()
        {
            return false;
        }
        a.members.iter().zip(&b.members).all(|(x, y)| match (x, y) {
            (ClassMember::Method { sig_raw: sx, body: bx }, ClassMember::Method { sig_raw: sy, body: by }) => {
                norm_code(sx) == norm_code(sy)
                    && match (bx, by) {
                        (None, None) => true,
                        (Some(x), Some(y)) => self.equiv(*x, other, *y),
                        _ => false,
                    }
            }
            (
                ClassMember::Field { sig_raw: sx, name: nx, init: ix },
                ClassMember::Field { sig_raw: sy, name: ny, init: iy },
            ) => {
                nx == ny
                    && norm_code(sx) == norm_code(sy)
                    && match (ix, iy) {
                        (None, None) => true,
                        (Some(x), Some(y)) => self.equiv(*x, other, *y),
                        _ => false,
                    }
            }
            (ClassMember::StaticBlock { body: x }, ClassMember::StaticBlock { body: y }) => {
                self.equiv(*x, other, *y)
            }
            (ClassMember::Raw(x), ClassMember::Raw(y)) => norm_code(x) == norm_code(y),
            _ => false,
        })
    }

    /// 作用域解析后的 VarRef 局部性（prepare 之后有效）。
    pub fn is_local_ref(&self, id: TsId) -> bool {
        self.local_marks.get(id.0 as usize).copied().unwrap_or(false)
    }

    /// resolver 产出的全部绑定（按 BindingId 索引）。
    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    /// BindingId → 绑定事实。
    pub fn binding(&self, bid: u32) -> Option<&Binding> {
        self.bindings.get(bid as usize)
    }

    /// 节点解析到的绑定（VarRef/声明节点）。未解析或非名字节点 → None。
    pub fn binding_of(&self, id: TsId) -> Option<&Binding> {
        self.binding_of
            .get(id.0 as usize)
            .and_then(|b| b.and_then(|bid| self.binding(bid)))
    }

    /// 节点解析到的 BindingId。
    pub fn binding_id(&self, id: TsId) -> Option<u32> {
        self.binding_of.get(id.0 as usize).copied().flatten()
    }

    // ---- 语句事实收集（与 Java 版同构）----

    fn collect_stmt_facts(&self, root: TsId) -> Box<[StmtFact]> {
        let mut counts: HashMap<u32, u32> = HashMap::new();
        let mut caps: HashSet<u32> = HashSet::new();
        let mut shadows: HashSet<u32> = HashSet::new();
        let mut stack: Vec<(TsId, bool)> = vec![(root, false)];
        while let Some((id, in_lambda)) = stack.pop() {
            let node = &self.nodes[id.0 as usize];
            let is_lambda = matches!(node.data, NodeData::Lambda { .. } | NodeData::FuncStmt { .. });
            let is_named_decl = matches!(
                node.data,
                NodeData::VarDecl { .. }
                    | NodeData::ForEach { .. }
                    | NodeData::Catch { .. }
                    | NodeData::Destruct { .. }
            );
            if matches!(node.data, NodeData::VarRef { .. }) {
                if let Some(key) = self.names.var_key(id.0 as usize) {
                    *counts.entry(key).or_insert(0) += 1;
                    if in_lambda {
                        caps.insert(key);
                    }
                }
            } else if is_named_decl {
                if let Some(key) = self.names.var_key(id.0 as usize) {
                    shadows.insert(key);
                }
            }
            let child_in_lambda = in_lambda || is_lambda;
            for &c in node.children.as_slice() {
                stack.push((c, child_in_lambda));
            }
        }
        let mut keys: HashSet<u32> = counts.keys().copied().collect();
        keys.extend(shadows.iter().copied());
        let mut facts: Vec<StmtFact> = keys
            .into_iter()
            .map(|key| StmtFact {
                key,
                refs: counts.get(&key).copied().unwrap_or(0),
                lambda_cap: caps.contains(&key),
                shadow: shadows.contains(&key),
            })
            .collect();
        facts.sort_unstable_by_key(|f| f.key);
        facts.into_boxed_slice()
    }

    fn stmt_opaque_fact(&self, root: TsId) -> Option<StmtOpq> {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            if self.is_opaque(id) {
                return Some(StmtOpq { opaque: true });
            }
            for &c in self.nodes[id.0 as usize].children.as_slice() {
                stack.push(c);
            }
        }
        Some(StmtOpq { opaque: false })
    }

    fn collect_stmt_kill_facts(&self, root: TsId) -> Box<[StmtKillFact]> {
        let mut facts: Vec<StmtKillFact> = cure_engine::rules::collect_stmt_kill_events(self, root)
            .into_iter()
            .map(|(key, event)| match event {
                StmtKill::None => unreachable!("collector only records real events"),
                StmtKill::Read => StmtKillFact { key, kind: 1, assign: TsId(0) },
                StmtKill::Shadow => StmtKillFact { key, kind: 2, assign: TsId(0) },
                StmtKill::DomWrite(assign) => StmtKillFact { key, kind: 3, assign },
            })
            .collect();
        facts.sort_unstable_by_key(|f| f.key);
        facts.into_boxed_slice()
    }

    /// root 相关作用域分析：VarRef → 局部性 + var 提升陷阱标记。
    ///
    /// 作用域栈：Block/For/Try/Lambda/FuncStmt 推新层（JS switch 与
    /// 合成 Group **不**推——switch 的 case 共享一个块作用域，Group 是
    /// 解析器合成容器非真实花括号）；let/const 声明插入当前层；简单参数
    /// 进新层（闭包可见外层）；模式参数不入（保守）。
    ///
    /// var 提升陷阱：`var` 是**函数级**作用域而引擎规则按**块**扫描。
    /// 声明 d（在块 C 的第 di 个孩子位）的名字若在 root 内出现在
    /// 「C 的第 di+1.. 个孩子的子树」之外（更早兄弟/祖先块/兄弟块/
    /// 其他嵌套函数），块扫描看不见该出现 → 声明标记 is_opaque（规则
    /// 拒绝触碰；var_trap）。全部出现都在可见窗口内 → 正常局部处理。
    /// 直接 eval 污染区计算（见 `eval_taint` 字段注释）。
    ///
    /// 根 = 无父节点的节点（模块合成根 / 独立 TopItem / 类方法体）。
    /// 若某个根**不是**函数节点且子树含 `eval(...)` 调用 → 顶层 eval，
    /// 整棵 AST 污染；否则含 eval 的每个函数节点其子树全部污染。
    fn rebuild_eval_taint(&mut self, root: TsId) {
        let n = self.nodes.len();
        self.eval_taint.clear();
        self.eval_taint.resize(n, false);
        self.group_decls.clear();
        self.group_decls.resize(n, false);
        if n == 0 {
            return;
        }
        // 只处理**从当前根可达**的节点：解析器在 parse_block 展开 Group 后
        // 仍把 Group 节点留在 arena（孩子已上提）——若扫描全部 arena，
        // 会把这些已脱离树的孩子误标为 group_decls 不透明，导致首轮
        // 无法优化、打印重解析后才生效（非幂等）。
        let mut reachable = vec![false; n];
        let mut stack = vec![root];
        while let Some(x) = stack.pop() {
            let xi = x.0 as usize;
            if xi >= n || reachable[xi] {
                continue;
            }
            reachable[xi] = true;
            if matches!(self.nodes[xi].data, NodeData::Group) {
                for c in self.nodes[xi].children.as_slice() {
                    // 只标记 var/using：语句表规则（dead_decl/dead_store）
                    // 对它们没有安全的组外引用视野；let/const 的删除已被
                    // dead_decl_allowed 拒绝，标记会连带禁止声明初始化式的
                    // 纯折叠（首轮不折、打印重解析后二轮才折——非幂等）。
                    if (c.0 as usize) < n
                        && matches!(
                            self.nodes[c.0 as usize].data,
                            NodeData::VarDecl { kind: VarKind::Var | VarKind::Using, .. }
                        )
                    {
                        self.group_decls[c.0 as usize] = true;
                    }
                }
            }
            for c in self.nodes[xi].children.as_slice() {
                stack.push(*c);
            }
        }
        let contains_eval = {
            let mut ce = vec![false; n];
            let mut stack = vec![(root, false)];
            while let Some((x, exiting)) = stack.pop() {
                let xi = x.0 as usize;
                if exiting {
                    let mut hit = self.is_direct_eval_call(x);
                    if !hit {
                        for c in self.nodes[xi].children.as_slice() {
                            if (c.0 as usize) < n && ce[c.0 as usize] {
                                hit = true;
                                break;
                            }
                        }
                    }
                    ce[xi] = hit;
                } else {
                    stack.push((x, true));
                    for c in self.nodes[xi].children.as_slice() {
                        stack.push((*c, false));
                    }
                }
            }
            ce
        };
        if contains_eval[root.0 as usize] && !self.is_function_root(root) {
            // 根作用域里的直接 eval → 整个根子树污染
            for i in 0..n {
                if reachable[i] {
                    self.eval_taint[i] = true;
                }
            }
            return;
        }
        let mut stack = vec![(root, self.is_function_root(root) && contains_eval[root.0 as usize])];
        while let Some((x, tainted)) = stack.pop() {
            let xi = x.0 as usize;
            let t = tainted || (self.is_function_root(x) && contains_eval[xi]);
            if t {
                self.eval_taint[xi] = true;
            }
            for c in self.nodes[xi].children.as_slice() {
                stack.push((*c, t));
            }
        }
    }

    fn is_direct_eval_call(&self, id: TsId) -> bool {
        // 形参默认值以原文保存（`((a = eval("x")) => …)`）——原文里的
        // 直接 eval 同样污染函数（Test262 scope-param-elem-var-close）。
        match &self.nodes[id.0 as usize].data {
            NodeData::Lambda { params_raw, .. } => return raw_has_direct_eval(params_raw),
            NodeData::FuncStmt { sig_raw, params_raw, .. } => {
                return raw_has_direct_eval(params_raw) || raw_has_direct_eval(sig_raw)
            }
            _ => {}
        }
        // `arguments` 对象：参数与自变量别名（mapped arguments）——闭包/
        // 赋值规则的作用域模型看不到这种联动，函数子树整体保守化
        //（Test262 arguments-object/mapped 抓获）。
        if let NodeData::VarRef { name } = &self.nodes[id.0 as usize].data {
            if self.sn(*name) == "arguments" {
                return true;
            }
        }
        if self.kind(id) != NodeKind::Call {
            return false;
        }
        let Some(&callee) = self.nodes[id.0 as usize].children.as_slice().first() else {
            return false;
        };
        // `(eval)(...)` 仍是**直接** eval（括号不改变 Reference）——
        // 曾因 Paren 包裹漏判污染，函数内 var 被当死存储删除
        //（Test262 parenthesized-eval-is-direct 抓获）
        let mut c = callee;
        while let NodeData::Paren = self.nodes[c.0 as usize].data {
            let Some(&inner) = self.nodes[c.0 as usize].children.as_slice().first() else {
                return false;
            };
            c = inner;
        }
        matches!(self.nodes[c.0 as usize].data, NodeData::VarRef { name } if self.sn(name) == "eval")
    }

    fn is_function_root(&self, id: TsId) -> bool {
        matches!(self.nodes[id.0 as usize].data, NodeData::FuncStmt { .. })
            || self.kind(id) == NodeKind::Lambda
    }

    pub(crate) fn rebuild_local_marks(&mut self, root: TsId) {
        self.rebuild_eval_taint(root);
        crate::resolve::rebuild(self, root);
    }
}

/// 节点的最近 Block 祖先（Block/Group 同映射 NodeKind::Block）及其
/// 在其中的孩子下标。
fn ancestor_block_pos(nodes: &[Node], parents: &[(TsId, usize)], node: TsId) -> Option<(TsId, usize)> {
    let mut cur = node;
    loop {
        let (p, i) = parents[cur.0 as usize];
        if i == usize::MAX {
            return None; // 到根
        }
        if matches!(
            &nodes[p.0 as usize].data,
            NodeData::Block | NodeData::Group
        ) {
            return Some((p, i));
        }
        cur = p;
    }
}

/// o 是否在块 block 的第 dpos+1.. 个孩子的子树内（规则扫描可见窗口）。
fn inside_later_sibling(parents: &[(TsId, usize)], o: TsId, block: TsId, dpos: usize) -> bool {
    let mut cur = o;
    loop {
        let (p, i) = parents[cur.0 as usize];
        if i == usize::MAX {
            return false;
        }
        if p == block {
            return i > dpos;
        }
        cur = p;
    }
}

// ---------------------------------------------------------------------------
// 辅助：简单参数/绑定名提取（模式 → None）
// ---------------------------------------------------------------------------

/// `catch` 参数原文 → 简单绑定名（`e` → Some("e")；`{a}`/`e: T` → None）。
pub fn simple_binding_name(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    let ident: String = t
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
        .collect();
    if !ident.is_empty() && ident.chars().next().is_some_and(|c| !c.is_ascii_digit())
        && t == ident {
            return Some(ident);
        }
    None
}

/// 参数原文 → 简单参数名列表（全部为裸标识符时；含模式/默认值/注解 → None）。
pub fn simple_params(raw: &str) -> Option<Vec<String>> {
    let t = raw.trim();
    if t.is_empty() {
        return Some(vec![]);
    }
    // 顶层逗号拆分（尖/圆/花括号深度感知）
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    for c in t.chars() {
        match c {
            '(' | '[' | '{' | '<' => {
                depth += 1;
                cur.push(c);
            }
            ')' | ']' | '}' | '>' => {
                depth -= 1;
                cur.push(c);
            }
            ',' if depth == 0 => {
                parts.push(cur.clone());
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    parts.push(cur);
    let mut names = Vec::new();
    for p in parts {
        // 剥除类型注解（顶层 `:`——TS `x: T`）
        let mut base = p.as_str();
        if let Some(colon) = find_top_colon(p.as_str()) {
            base = &p[..colon];
        }
        let base = base.trim();
        if base.is_empty() {
            return None; // 尾随逗号等畸形
        }
        let ident: String = base
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
            .collect();
        if ident != base {
            return None; // 模式/默认值/rest/修饰——保守放弃
        }
        names.push(ident);
    }
    Some(names)
}

fn find_top_colon(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' | '>' => depth -= 1,
            ':' if depth == 0 => return Some(i),
            '?' if depth == 0 && s[i..].starts_with('?') => {
                // `x?: T` 的可选标记——继续找冒号
            }
            _ => {}
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Lang 实现
// ---------------------------------------------------------------------------

impl Lang for TsAst {
    type Id = TsId;
    type NameKey = u32;

    fn kind(&self, id: TsId) -> NodeKind {
        TsAst::kind(self, id)
    }
    fn children(&self, id: TsId) -> &[TsId] {
        TsAst::children(self, id)
    }
    fn set_child(&mut self, parent: TsId, index: usize, new: TsId) {
        self.nodes[parent.0 as usize].children.set(index, new);
    }
    fn remove_child(&mut self, parent: TsId, index: usize) {
        self.nodes[parent.0 as usize].children.remove(index);
    }
    fn splice(&mut self, node: TsId, index: usize, remove: usize, insert: Vec<TsId>) {
        self.nodes[node.0 as usize].children.splice(index, remove, insert);
    }

    fn build_return(&mut self, value: Option<TsId>) -> TsId {
        self.ret(value)
    }
    fn build_block(&mut self, stmts: Vec<TsId>) -> TsId {
        self.block(stmts)
    }
    fn build_unary(&mut self, op: UnOp, operand: TsId) -> TsId {
        self.un(op, operand)
    }
    fn build_bool(&mut self, v: bool) -> TsId {
        self.lit(JsLit::Bool(v))
    }
    fn build_int(&mut self, v: i64, _long: bool) -> TsId {
        self.lit(JsLit::Int(v))
    }
    fn build_bin(&mut self, op: BinOp, l: TsId, r: TsId) -> TsId {
        self.bin(op, l, r)
    }
    fn build_ternary(&mut self, c: TsId, a: TsId, b: TsId) -> TsId {
        self.ternary(c, a, b)
    }
    fn build_assign(&mut self, target: TsId, value: TsId) -> TsId {
        self.assign(target, value)
    }
    fn build_str(&mut self, s: &str) -> TsId {
        self.lit(JsLit::Str(s.to_string()))
    }
    fn build_char(&mut self, c: char) -> TsId {
        self.lit(JsLit::Str(c.to_string()))
    }

    // ---- 字面量折叠语义（ECMAScript；引擎侧只做形态匹配）----

    fn fold_lit_bin(&mut self, op: BinOp, l: TsId, r: TsId) -> Option<TsId> {
        let ll = self.literal(l)?;
        let rr = self.literal(r)?;
        // NaN / ±Inf / -0 操作数（IEEE 雷区：NaN 比较、-0 打印/1÷-0）不折
        if num_hazard(ll) || num_hazard(rr) {
            return None;
        }
        // 字符串拼接（Str+Str / Str+Int —— Int 走 JS 整数字符串化）
        if op == BinOp::Add {
            if let (LitRef::Str(a), LitRef::Str(b)) = (ll, rr) {
                return Some(self.build_str(&format!("{a}{b}")));
            }
            if let (LitRef::Str(a), LitRef::Int(v)) = (ll, rr) {
                if js_int_representable(v) {
                    return Some(self.build_str(&format!("{a}{v}")));
                }
            }
            if let (LitRef::Int(v), LitRef::Str(b)) = (ll, rr) {
                if js_int_representable(v) {
                    return Some(self.build_str(&format!("{v}{b}")));
                }
            }
            // 整值/有限 Double 的字符串化：`1.0 + ''` 的 JS 结果是
            // `"1"`（首轮 Double 未折、二轮打印成 Int 才折——幂等差分
            // 抓获）。format_js_number 与打印器同一 ECMA Number→String。
            if let (LitRef::Str(a), LitRef::Double(v)) = (ll, rr) {
                if v.is_finite() {
                    return Some(self.build_str(&format!("{a}{}", format_js_number(v))));
                }
            }
            if let (LitRef::Double(v), LitRef::Str(b)) = (ll, rr) {
                if v.is_finite() {
                    return Some(self.build_str(&format!("{}{b}", format_js_number(v))));
                }
            }
        }
        // 混合 Int/Double 算术（42 + 3.5 —— 解析后的字面量折叠链）：
        // f64 域求值（Int 精确可表示时无损失），结果 Double——打印走
        // JS 格式化。仅 + - * / % **（比较另走 fold_lit_cmp）。
        {
            let dbl_of = |x: LitRef<'_>| -> Option<f64> {
                match x {
                    LitRef::Int(v) if js_int_representable(v) => Some(v as f64),
                    LitRef::Double(v) => Some(v),
                    _ => None,
                }
            };
            let mixed = !matches!((ll, rr), (LitRef::Int(_), LitRef::Int(_)));
            if mixed {
                if let (Some(a), Some(b)) = (dbl_of(ll), dbl_of(rr)) {
                    let r = match op {
                        BinOp::Add => Some(a + b),
                        BinOp::Sub => Some(a - b),
                        BinOp::Mul => Some(a * b),
                        BinOp::Div => {
                            if b == 0.0 {
                                None
                            } else {
                                Some(a / b)
                            }
                        }
                        BinOp::Rem => {
                            if b == 0.0 {
                                None
                            } else {
                                Some(a % b)
                            }
                        }
                        BinOp::Pow => {
                            if b.fract() == 0.0 && b.abs() <= 1024.0 {
                                Some(a.powf(b))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    if let Some(r) = r {
                        if !r.is_nan() && !r.is_infinite() && !(r == 0.0 && r.is_sign_negative()) {
                            // 整数值保持 Int（打印形态统一）
                            if r.fract() == 0.0 && r.abs() <= (1u64 << 53) as f64 {
                                return Some(self.build_int(r as i64, false));
                            }
                            return Some(self.lit(JsLit::Double(r)));
                        }
                    }
                }
            }
        }
        // 数值运算：JS number 域 = f64。位运算/移位走 ToInt32/ToUint32
        // 环绕域；算术在 ±2^53 精确域（非精确/非整数 → Double 或拒绝）。
        // 整值 Double 也进入 ToInt32 域：`1e3 >> 1` 与 `1000 >> 1` 同为
        // 500。只接受 ±2^53 内整值——`v as i64` 精确，随后 `as i32`
        // 即 JS 的模 2^32 环绕（曾拒折 → 常量传播后首轮不折、二轮打印
        // 成 Int 才折，幂等差分抓获）。
        let int_of = |x: LitRef<'_>| -> Option<i64> {
            match x {
                LitRef::Int(v) => Some(v),
                LitRef::Double(v)
                    if v.is_finite() && v.fract() == 0.0 && v.abs() <= (1u64 << 53) as f64 =>
                {
                    Some(v as i64)
                }
                _ => None,
            }
        };
        let (a, b) = (int_of(ll)?, int_of(rr)?);
        if !js_int_representable(a) || !js_int_representable(b) {
            return None;
        }
        // i32 域（ToInt32 环绕）
        let i32v: Option<i64> = match op {
            BinOp::BitAnd => Some(((a as i32) & (b as i32)) as i64),
            BinOp::BitOr => Some(((a as i32) | (b as i32)) as i64),
            BinOp::BitXor => Some(((a as i32) ^ (b as i32)) as i64),
            BinOp::Shl => {
                let s = (b as i32) & 31;
                Some(((a as i32).wrapping_shl(s as u32)) as i64)
            }
            BinOp::Shr => {
                let s = (b as i32) & 31;
                Some(((a as i32).wrapping_shr(s as u32)) as i64)
            }
            BinOp::UShr => {
                let s = (b as i32) & 31;
                Some((((a as i32) as u32).wrapping_shr(s as u32)) as i64)
            }
            _ => None,
        };
        if let Some(v) = i32v {
            return Some(self.build_int(v, false));
        }
        // f64 精确域算术（Add/Sub/Mul/Pow/Div/Rem）
        let r = arith_f64(op, a, b)?;
        if let Some(v) = r {
            return Some(self.build_int(v, false));
        }
        // 非整/超域结果 → Double 字面量（打印走 JS number 格式化）
        let rf = match op {
            BinOp::Add => a as f64 + b as f64,
            BinOp::Sub => a as f64 - b as f64,
            BinOp::Mul => a as f64 * b as f64,
            BinOp::Pow => {
                let e = b.clamp(-1024, 1024);
                (a as f64).powi(e as i32)
            }
            BinOp::Div => {
                if b == 0 {
                    return None; // ±Infinity——保真不折
                }
                a as f64 / b as f64
            }
            BinOp::Rem => {
                if b == 0 {
                    return None; // NaN
                }
                let v = (a as f64) % (b as f64);
                if v == 0.0 && v.is_sign_negative() {
                    return None; // -0（fmod(-5,5)）——打印敏感
                }
                v
            }
            _ => return None,
        };
        if rf.is_nan() || rf.is_infinite() {
            return None;
        }
        if rf == 0.0 && rf.is_sign_negative() {
            return None; // -0（-2*0 等）
        }
        Some(self.lit(JsLit::Double(rf)))
    }

    fn fold_lit_cmp(&mut self, op: BinOp, l: TsId, r: TsId) -> Option<bool> {
        let ll = self.literal(l)?;
        let rr = self.literal(r)?;
        // JS ===/== 的字面量域：number-string 不等、NaN 恒不等、
        // undefined 非字面量。这里只折「同类型字面量」：
        // number(整数域) 与 number、bool 与 bool、string 与 string（
        // 值相等）、null 与 null。
        let num_bad = |x: LitRef<'_>| match x {
            LitRef::Double(d) => d.is_nan() || d.is_infinite() || (d == 0.0 && d.is_sign_negative()),
            _ => false,
        };
        if num_bad(ll) || num_bad(rr) {
            return None;
        }
        let eq_like = matches!(op, BinOp::Eq | BinOp::Ne | BinOp::StrictEq | BinOp::StrictNe);
        if !eq_like && !matches!(op, BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge) {
            return None;
        }
        // 整值 Double 参与比较折叠：`1e3 === 1000` 为真。±2^53 内整值
        // 转 i64 精确；任一侧非整值（1000.5）→ 不折（不落到 i64 比较）。
        let int_of = |x: LitRef<'_>| -> Option<i64> {
            match x {
                LitRef::Int(v) => Some(v),
                LitRef::Double(v)
                    if v.is_finite() && v.fract() == 0.0 && v.abs() <= (1u64 << 53) as f64 =>
                {
                    Some(v as i64)
                }
                _ => None,
            }
        };
        if let (Some(x), Some(y)) = (int_of(ll), int_of(rr)) {
            if eq_like {
                return Some(if matches!(op, BinOp::Eq | BinOp::StrictEq) { x == y } else { x != y });
            }
            return cmp_i64(op, x, y);
        }
        // 布尔相等（JS true/false 是规范值——== 与 === 一致）
        if let (LitRef::Bool(x), LitRef::Bool(y)) = (ll, rr) {
            if matches!(op, BinOp::Eq | BinOp::Ne | BinOp::StrictEq | BinOp::StrictNe) {
                return Some(if matches!(op, BinOp::Eq | BinOp::StrictEq) { x == y } else { x != y });
            }
        }
        // 字符串值比较（== 与 === 对 string 都是值比较）
        if let (LitRef::Str(x), LitRef::Str(y)) = (ll, rr) {
            if matches!(op, BinOp::Eq | BinOp::Ne | BinOp::StrictEq | BinOp::StrictNe) {
                return Some(if matches!(op, BinOp::Eq | BinOp::StrictEq) { x == y } else { x != y });
            }
            // JS 字符串次序 = **UTF-16 码元**字典序（不是 Unicode
            // 标量序）：U+10000 的首码元 D800 < FFFF，标量序相反
            //（Test262 greater-than/less-than 家族抓获）
            let xu = x.encode_utf16().collect::<Vec<u16>>();
            let yu = y.encode_utf16().collect::<Vec<u16>>();
            let ord = xu.cmp(&yu);
            return match op {
                BinOp::Lt => Some(ord.is_lt()),
                BinOp::Le => Some(ord.is_le()),
                BinOp::Gt => Some(ord.is_gt()),
                BinOp::Ge => Some(ord.is_ge()),
                _ => None,
            };
        }
        // null == null
        if let (LitRef::Null, LitRef::Null) = (ll, rr) {
            if matches!(op, BinOp::Eq | BinOp::StrictEq) {
                return Some(true);
            }
            if matches!(op, BinOp::Ne | BinOp::StrictNe) {
                return Some(false);
            }
        }
        None
    }

    fn fold_lit_neg(&mut self, v: i64, _wide: bool) -> Option<i64> {
        // JS：数值字面量取负仍是精确 number（-0 走 Double 路径不进这）。
        // 2^53 边界：-v 超域 → f64 不精确 → 拒绝
        let n = -v;
        if !js_int_representable(n) {
            return None;
        }
        Some(n)
    }

    fn reassoc_delta(
        &mut self,
        op1: BinOp,
        k1: i64,
        op2: BinOp,
        k2: i64,
        _wide: bool,
    ) -> Option<ReassocOutcome<TsId>> {
        use BinOp::*;
        // JS 无 int 环绕——加/减 delta 合并须全程在 ±2^53 精确域
        let raw: i64 = match (op1, op2) {
            (a, b) if matches!(a, Add | Sub) && matches!(b, Add | Sub) => {
                let sign = |o: BinOp| if o == Sub { -1i64 } else { 1i64 };
                sign(a) * k1 + sign(b) * k2
            }
            // 位异或不结合于 JS 的 f64 语义域？——JS ^ 是 i32 位运算，
            // 环绕恒等成立；但操作数须同为整数域（规则侧已保证字面量）
            (BitXor, BitXor) => k1 ^ k2,
            _ => return None,
        };
        if !js_int_representable(raw) {
            return None;
        }
        let d = raw;
        if d == 0 {
            return Some(ReassocOutcome::Neutral);
        }
        if op1 == BitXor && op2 == BitXor {
            return Some(ReassocOutcome::Lit(self.build_int(d, false), BitXor));
        }
        if d > 0 {
            Some(ReassocOutcome::Lit(self.build_int(d, false), Add))
        } else {
            let nd = -d;
            if nd > 0 {
                Some(ReassocOutcome::Lit(self.build_int(nd, false), Sub))
            } else {
                Some(ReassocOutcome::Lit(self.build_int(d, false), Add))
            }
        }
    }

    fn copy_subtree(&mut self, id: TsId) -> TsId {
        let old_children: Vec<TsId> = self.nodes[id.0 as usize].children.as_slice().to_vec();
        let mut children = Vec::with_capacity(old_children.len());
        for c in old_children {
            children.push(self.copy_subtree(c));
        }
        let data = self.nodes[id.0 as usize].data.clone();
        self.push(data, children)
    }

    fn effect(&self, id: TsId) -> Effect {
        if let Some(e) = self.effect_cache.get(id.0 as usize).and_then(|x| *x) {
            return e;
        }
        let mut e = self.own_effect(id);
        for &c in self.nodes[id.0 as usize].children.as_slice() {
            e = e.worst(self.effect(c));
        }
        e
    }

    fn invalidate_effect(&mut self, id: TsId) {
        let i = id.0 as usize;
        if let Some(slot) = self.effect_cache.get_mut(i) {
            *slot = None;
        }
        if i < self.effect_dirty_mark.len() && !self.effect_dirty_mark[i] {
            self.effect_dirty_mark[i] = true;
            self.effect_dirty.push(id);
        }
        self.events.invalidate(id.0 as usize);
        if let Some(slot) = self.stmt_facts.get_mut(id.0 as usize) {
            *slot = None;
        }
        if let Some(slot) = self.stmt_kills.get_mut(id.0 as usize) {
            *slot = None;
        }
        if let Some(slot) = self.stmt_opq.get_mut(id.0 as usize) {
            *slot = None;
        }
    }

    fn region_events(&self, id: TsId) -> Option<&[cure_engine::kind::RegionEvent<TsId, u32>]> {
        self.events.slot(id.0 as usize)
    }

    fn var_key(&self, id: TsId) -> Option<u32> {
        self.names.var_key(id.0 as usize)
    }

    fn same_binding(&self, a: TsId, b: TsId) -> bool {
        match (self.binding_id(a), self.binding_id(b)) {
            (Some(x), Some(y)) => x == y,
            // 未解析（模式绑定/手工 AST/未 prepare）退回名字键——与旧
            // 行为一致；一侧有一侧无说明不是同一个可解析绑定。
            (None, None) => self.var_key(a).is_some() && self.var_key(a) == self.var_key(b),
            _ => false,
        }
    }

    fn stmt_var_ref_count(&self, stmt: TsId, key: u32) -> Option<u32> {
        let slot = self.stmt_facts.get(stmt.0 as usize)?.as_deref()?;
        Some(
            slot.binary_search_by_key(&key, |f| f.key)
                .ok()
                .map_or(0, |i| slot[i].refs),
        )
    }

    fn stmt_lambda_captures(&self, stmt: TsId, key: u32) -> Option<bool> {
        let slot = self.stmt_facts.get(stmt.0 as usize)?.as_deref()?;
        Some(
            slot.binary_search_by_key(&key, |f| f.key)
                .ok()
                .is_some_and(|i| slot[i].lambda_cap),
        )
    }

    fn stmt_kill_event(&self, stmt: TsId, key: u32) -> Option<StmtKill<TsId>> {
        let slot = self.stmt_kills.get(stmt.0 as usize)?.as_deref()?;
        Some(match slot.binary_search_by_key(&key, |f| f.key) {
            Ok(i) => match slot[i].kind {
                1 => StmtKill::Read,
                2 => StmtKill::Shadow,
                _ => StmtKill::DomWrite(slot[i].assign),
            },
            Err(_) => StmtKill::None,
        })
    }

    fn stmt_mentions(&self, stmt: TsId, key: u32) -> Option<bool> {
        let slot = self.stmt_facts.get(stmt.0 as usize)?.as_deref()?;
        Some(
            slot.binary_search_by_key(&key, |f| f.key)
                .ok()
                .is_some_and(|i| slot[i].refs > 0 || slot[i].shadow),
        )
    }

    fn stmt_opaque(&self, stmt: TsId) -> Option<bool> {
        self.stmt_opq.get(stmt.0 as usize)?.map(|f| f.opaque)
    }

    // ---- 外部事实 ----

    fn fact_single_def(&self, key: u32) -> bool {
        match (&self.facts, self.names.name(key)) {
            (Some(f), n) => f.var(n).single_def,
            _ => false,
        }
    }
    fn fact_single_use(&self, key: u32) -> bool {
        match (&self.facts, self.names.name(key)) {
            (Some(f), n) => f.var(n).single_use,
            _ => false,
        }
    }
    fn fact_bool_valued(&self, key: u32) -> bool {
        match (&self.facts, self.names.name(key)) {
            (Some(f), n) => f.var(n).bool_valued,
            _ => false,
        }
    }
    fn fact_call_effect(&self, name: &str) -> Option<cure_engine::Effect> {
        self.facts.as_ref().and_then(|f| {
            f.call_effects.get(name).map(|set| {
                set.iter().copied().fold(Effect::Pure, Effect::worst)
            })
        })
    }

    fn node_index(&self, id: TsId) -> usize {
        id.0 as usize
    }
    fn id_of_index(&self, idx: usize) -> TsId {
        TsId(idx as u32)
    }

    fn is_opaque(&self, id: TsId) -> bool {
        match self.data(id) {
            // 不可解析原文 / 类表达式原文
            NodeData::Raw { .. } | NodeData::ClassExpr { .. } => true,
            // 解构模式：绑定名在原文里（事件盲区）——含模式的名字必须
            // 以不透明拒绝传播/删除
            NodeData::Destruct { .. } | NodeData::DestructStmt { .. } => true,
            // 可选链元素：链式短路语义（`a?.b` 短路整链）——区域分析
            // 保守拒绝（含可选元素的子树不可搬移/内联）
            NodeData::Member { optional: true, .. }
            | NodeData::Index { optional: true, .. }
            | NodeData::Call { optional: true, .. } => true,
            // `using` 声明：块尾 dispose 是隐藏副作用——删除/搬移都改变
            // 可观察顺序（Test262 statements/using 族抓获）。
            NodeData::VarDecl { kind: VarKind::Using, .. } => true,
            // 合成 Group 的直接 VarDecl 孩子：Group 的 kind 是 Block，
            // 引擎的语句表规则（dead_decl/dead_store/assign_back_fold）
            // 会把 Group 当作用域，只扫描组内后继兄弟 → 漏掉组外引用，
            // 曾把 `let val, err;` 整条删除（Test262 BigInt large-bit-length
            // 与 annexB 家族抓获）。这些声明不参与优化。
            _ if self.group_decls.get(id.0 as usize).copied().unwrap_or(false) => true,
            // 直接 eval 污染区：整个函数子树（含形参默认值/嵌套闭包）
            _ if self.eval_taint.get(id.0 as usize).copied().unwrap_or(false) => true,
            // var 提升陷阱（prepare 计算的块外引用标记）
            _ => self.var_trap.get(id.0 as usize).copied().unwrap_or(false),
        }
    }

    fn own_effect(&self, id: TsId) -> Effect {
        match self.data(id) {
            NodeData::Literal(_)
            | NodeData::This
            | NodeData::Paren
            | NodeData::Block
            | NodeData::ModuleBlock
            | NodeData::Group
            | NodeData::Empty
            | NodeData::VarDecl { .. }
            | NodeData::Break { .. }
            | NodeData::Continue { .. }
            | NodeData::Return
            | NodeData::Throw
            | NodeData::If
            | NodeData::While
            | NodeData::DoWhile
            | NodeData::For { .. }
            | NodeData::ForEach { .. }
            | NodeData::Try
            | NodeData::Catch { .. }
            | NodeData::Switch
            | NodeData::Case { .. }
            | NodeData::Label { .. }
            | NodeData::ExprStmt
            | NodeData::Ternary
            | NodeData::ArrayLit
            | NodeData::ObjLit { .. }
            | NodeData::Template { .. }
            | NodeData::TsAs { .. }
            | NodeData::TsSatisfies { .. }
            | NodeData::TsNonNull
            | NodeData::Lambda { .. }
            | NodeData::FuncStmt { .. }
            | NodeData::Seq
            | NodeData::DestructStmt { .. } => Effect::Pure,
            // typeof 对未声明名不抛（与 Java 不同——member/new 访问可抛）
            NodeData::Unary { op: UnOp::TypeOf } => Effect::Pure,
            NodeData::Unary { op: UnOp::PreInc | UnOp::PreDec | UnOp::PostInc | UnOp::PostDec } => {
                Effect::MayWrite
            }
            NodeData::Unary { op: UnOp::Delete } => Effect::Unknown,
            // await：暂停期间任意副作用
            NodeData::Unary { op: UnOp::Await } => Effect::Unknown,
            NodeData::Unary { .. } => Effect::Pure,
            NodeData::VarRef { .. } => Effect::MayRead,
            // 成员读：null 基座 TypeError；optional 元素短路（子树效果
            // 经孩子聚合）——MayThrow 与 Java 对齐
            NodeData::Member { .. } | NodeData::Index { .. } => Effect::MayThrow,
            // new：构造器任意副作用
            NodeData::New => Effect::Unknown,
            NodeData::Call { .. } => Effect::Unknown,
            NodeData::Assign { .. } => Effect::MayWrite,
            NodeData::Binary { op } => {
                // JS 除法/模：除 0 不抛（±Infinity/NaN）；但 `in` /
                // instanceof 对 null 基座抛 TypeError（与 Member 同判）
                if matches!(op, BinOp::In | BinOp::Instanceof) {
                    Effect::MayThrow
                } else {
                    Effect::Pure
                }
            }
            NodeData::Raw { .. } | NodeData::ClassExpr { .. } => Effect::Unknown,
            NodeData::Destruct { .. } => Effect::MayThrow,
            NodeData::Spread => Effect::Pure,
        }
    }

    fn is_bool(&self, id: TsId) -> bool {
        // JS 语法级布尔值：比较/逻辑表达式、!、布尔字面量、instanceof、
            // in。VarRef 一律 false（JS 变量无类型——真值不确定性交给
        // 规则的保守守卫）
        match self.data(id) {
            NodeData::Literal(JsLit::Bool(_)) => true,
            NodeData::Binary { op } => {
                op.is_comparison() || matches!(op, BinOp::And | BinOp::Or | BinOp::Nullish)
            }
            NodeData::Unary { op: UnOp::Not } => true,
            NodeData::Paren => self.is_bool(self.children(id)[0]),
            NodeData::Ternary => {
                let ch = self.children(id);
                self.is_bool(ch[1]) && self.is_bool(ch[2])
            }
            _ => false,
        }
    }

    fn is_exact_int(&self, id: TsId) -> bool {
        // JS 整数域：|v| ≤ 2^53（f64 精确整数）。self_compare 等规则的
        // 前提。NaN 场景不可能出现在 Int。
        match self.data(id) {
            NodeData::Literal(JsLit::Int(v)) => js_int_representable(*v),
            NodeData::Literal(JsLit::NumRaw { val: NumVal::Int(v), .. }) => {
                js_int_representable(*v)
            }
            _ => false,
        }
    }

    fn is_local_var(&self, id: TsId) -> bool {
        // VarRef 须解析到局部绑定（let/const/参数/for 绑定/catch 参数）
        // 且**未被 λ 按引用捕获**（捕获名的延迟读对块级扫描不可见——
        // node 差分 closure 抓获）。未解析（全局/模块级/模式绑定/坏
        // 代码）→ false（保守）。
        if !matches!(self.data(id), NodeData::VarRef { .. }) {
            return false;
        }
        // 绑定级捕获判定：同名但不同绑定不再互相牵连（旧实现按名字
        // 全局集合，λ 自己的参数也会被误标捕获）。
        if let Some(b) = self.binding_of(id) {
            if b.kind == BindingKind::FunctionExprName {
                return false;
            }
            return !b.module_level && !b.captured;
        }
        if let NodeData::VarRef { name } = self.data(id) {
            if self.captured_names.contains(self.sn(*name)) {
                return false;
            }
        }
        self.is_local_ref(id)
    }

    fn is_inferred_decl(&self, decl: TsId) -> bool {
        // JS：全部声明无类型标注（ty_raw 只是 TS 注解）→ 剥 init 合法
        //（`let x;` = undefined）。返回 false 让 store_kill 剥 init 生效。
        let _ = decl;
        false
    }

    fn has_pattern_bindings(&self, root: TsId) -> bool {
        // 解构模式（Destruct 不透明已拦截区域分析）——本钩子用于引擎侧
        // 模式绑定守卫（Java instanceof bind 的位置）；TS 侧解构一律
        // 走不透明通道，返回 false 即可
        let _ = root;
        false
    }

    fn bin_op(&self, id: TsId) -> Option<BinOp> {
        match self.data(id) {
            NodeData::Binary { op } => Some(*op),
            _ => None,
        }
    }

    fn un_op(&self, id: TsId) -> Option<UnOp> {
        match self.data(id) {
            NodeData::Unary { op } => Some(*op),
            _ => None,
        }
    }

    fn literal(&self, id: TsId) -> Option<LitRef<'_>> {
        match self.data(id) {
            NodeData::Literal(JsLit::Bool(b)) => Some(LitRef::Bool(*b)),
            NodeData::Literal(JsLit::Int(v)) => Some(LitRef::Int(*v)),
            NodeData::Literal(JsLit::Double(v)) => Some(LitRef::Double(*v)),
            NodeData::Literal(JsLit::NumRaw { val: NumVal::Int(v), .. }) => Some(LitRef::Int(*v)),
            NodeData::Literal(JsLit::NumRaw { val: NumVal::Double(v), .. }) => {
                Some(LitRef::Double(*v))
            }
            NodeData::Literal(JsLit::Str(s)) => Some(LitRef::Str(s.as_str())),
            NodeData::Literal(JsLit::Null) => Some(LitRef::Null),
            // BigInt/Regex 无值语义（引擎词汇外）
            _ => None,
        }
    }

    fn var_name(&self, id: TsId) -> Option<&str> {
        match self.data(id) {
            NodeData::VarRef { name } => Some(self.sn(*name)),
            NodeData::VarDecl { name, .. } => Some(name),
            NodeData::ForEach { name, .. } => Some(name),
            // catch 参数：简单绑定名才入名字表（模式 → None，保守不可见）
            NodeData::Catch { param_raw } => match param_raw.as_deref() {
                // 简单名返回原文（prepare 收集期借用 &self——克隆进 intern
                // 由 NameTable::extend_owned 完成）
                Some(_) => None, // 见下方 catch_simple_name 预处理
                None => None,
            },
            NodeData::FuncStmt { name, .. } => Some(name),
            _ => None,
        }
    }

    fn member_name(&self, id: TsId) -> Option<&str> {
        match self.data(id) {
            NodeData::Member { name, .. } => Some(self.sn(*name)),
            _ => None,
        }
    }

    fn payload_equal(&self, a: TsId, b: TsId) -> bool {
        // 结构相等性用的负载比较：模板 quasis、对象键、类型原文、
        // 字面量值（±0.0 区分——JS 1/-0 语义）
        match (self.data(a), self.data(b)) {
            (NodeData::Literal(x), NodeData::Literal(y)) => js_lit_value_eq(x, y),
            (NodeData::Template { quasis: q1, tag: t1 }, NodeData::Template { quasis: q2, tag: t2 }) => {
                q1 == q2 && t1 == t2
            }
            (NodeData::ObjLit { props: p1 }, NodeData::ObjLit { props: p2 }) => p1 == p2,
            (NodeData::Member { name: n1, optional: o1 }, NodeData::Member { name: n2, optional: o2 }) => {
                n1 == n2 && o1 == o2
            }
            (NodeData::Index { optional: o1 }, NodeData::Index { optional: o2 }) => o1 == o2,
            (NodeData::TsAs { ty_raw: t1 }, NodeData::TsAs { ty_raw: t2 }) => t1 == t2,
            (NodeData::TsSatisfies { ty_raw: t1 }, NodeData::TsSatisfies { ty_raw: t2 }) => t1 == t2,
            (NodeData::Destruct { pattern_raw: p1 }, NodeData::Destruct { pattern_raw: p2 }) => {
                p1 == p2
            }
            (NodeData::VarDecl { name: n1, kind: k1, ty_raw: t1 }, NodeData::VarDecl { name: n2, kind: k2, ty_raw: t2 }) => {
                n1 == n2 && k1 == k2 && t1 == t2
            }
            (NodeData::ForEach { name: n1, of: o1, decl: d1, ty_raw: t1, await_: a1 },
             NodeData::ForEach { name: n2, of: o2, decl: d2, ty_raw: t2, await_: a2 }) => {
                n1 == n2 && o1 == o2 && d1 == d2 && t1 == t2 && a1 == a2
            }
            (NodeData::Catch { param_raw: p1 }, NodeData::Catch { param_raw: p2 }) => p1 == p2,
            (NodeData::Assign { op: o1 }, NodeData::Assign { op: o2 }) => o1 == o2,
            (NodeData::Binary { op: o1 }, NodeData::Binary { op: o2 }) => o1 == o2,
            (NodeData::Unary { op: o1 }, NodeData::Unary { op: o2 }) => o1 == o2,
            (NodeData::Break { label: l1 }, NodeData::Break { label: l2 }) => l1 == l2,
            (NodeData::Continue { label: l1 }, NodeData::Continue { label: l2 }) => l1 == l2,
            (NodeData::Label { name: n1 }, NodeData::Label { name: n2 }) => n1 == n2,
            (NodeData::Case { labels: l1, is_default: d1, arrow: a1 },
             NodeData::Case { labels: l2, is_default: d2, arrow: a2 }) => {
                l1 == l2 && d1 == d2 && a1 == a2
            }
            (NodeData::For { inits: i1, has_cond: c1, steps: s1 },
             NodeData::For { inits: i2, has_cond: c2, steps: s2 }) => {
                i1 == i2 && c1 == c2 && s1 == s2
            }
            (NodeData::Lambda { params_raw: p1, name: n1, generics_raw: e1, space: s1, arrow: a1, is_async: y1, is_generator: g1 },
             NodeData::Lambda { params_raw: p2, name: n2, generics_raw: e2, space: s2, arrow: a2, is_async: y2, is_generator: g2 }) => {
                p1 == p2 && n1 == n2 && e1 == e2 && s1 == s2 && a1 == a2 && y1 == y2 && g1 == g2
            }
            (NodeData::FuncStmt { sig_raw: s1, name: n1, params_raw: p1 },
             NodeData::FuncStmt { sig_raw: s2, name: n2, params_raw: p2 }) => {
                s1 == s2 && n1 == n2 && p1 == p2
            }
            (NodeData::Call { optional: o1 }, NodeData::Call { optional: o2 }) => o1 == o2,
            (NodeData::DestructStmt { kw: k1, pattern: p1 }, NodeData::DestructStmt { kw: k2, pattern: p2 }) => {
                k1 == k2 && p1 == p2
            }
            (NodeData::Seq, NodeData::Seq) => true,
            (NodeData::Raw { text: t1 }, NodeData::Raw { text: t2 }) => t1 == t2,
            (NodeData::ClassExpr { raw: r1 }, NodeData::ClassExpr { raw: r2 }) => r1 == r2,
            // unit 变体（std::mem::discriminant 兜底已覆盖——此处显式
            // 列出可读性更好，但 Rust 模式不允许同绑定重复 → 改用
            // discriminant 比较）
            _ if std::mem::discriminant(&self.nodes[a.0 as usize].data)
                == std::mem::discriminant(&self.nodes[b.0 as usize].data)
                && matches!(
                    self.data(a),
                    NodeData::Block
                        | NodeData::Group
                        | NodeData::Empty
                        | NodeData::ExprStmt
                        | NodeData::If
                        | NodeData::While
                        | NodeData::DoWhile
                        | NodeData::Return
                        | NodeData::Throw
                        | NodeData::Try
                        | NodeData::Switch
                        | NodeData::Paren
                        | NodeData::Ternary
                        | NodeData::New
                        | NodeData::ArrayLit
                        | NodeData::This
                        | NodeData::Spread
                        | NodeData::TsNonNull
                ) => true,
            _ => false,
        }
    }

    fn is_field_name(&self, _name: &str) -> bool {
        // JS 模块级 let/const 对其他函数可见（同 Java 字段语义）——
        // 但本实现把它们直接判为非局部（is_local_var false），
        // 无需再走字段读可移动性通道
        false
    }

    fn assign_op(&self, id: TsId) -> Option<BinOp> {
        match self.data(id) {
            NodeData::Assign { op: Some(op) } => Some(*op),
            _ => None,
        }
    }

    fn contains_hoisted_decl(&self, root: TsId) -> bool {
        // FuncStmt（嵌套函数声明）+ var 声明（提升语义在块搬移时变化）。
        // λ/函数体内部不下钻：那里的 var/函数声明不会漏到外层。
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            match self.data(n) {
                NodeData::FuncStmt { .. } => return true,
                // let/const 搬出分支会改变块级作用域/TDZ（if(true){const z}
                // 曾变成模块级重复声明）
                NodeData::VarDecl { .. } => return true,
                NodeData::Lambda { .. } => continue,
                _ => {}
            }
            for &c in self.children(n) {
                stack.push(c);
            }
        }
        false
    }

    fn local_propagation_value_ok(&self, _decl: TsId, value: TsId) -> bool {
        // 安全面：字面量；或**只读未改写参数**的纯标量表达式
        // （算术/逻辑/三元——参数未被赋值/捕获，值在声明与使用之间
        // 稳定）。全局名/成员读/调用一律拒绝：求值时刻的值可能变化
        //（Test262 eval/TypedArray/class 家族多次抓获）。
        let mut stack = vec![value];
        let mut saw_non_lit = false;
        while let Some(n) = stack.pop() {
            match self.data(n) {
                NodeData::Literal(_) | NodeData::Paren => {}
                NodeData::Unary { .. } | NodeData::Binary { .. } | NodeData::Ternary => {
                    saw_non_lit = true;
                }
                NodeData::VarRef { name } => {
                    let nm = self.sn(*name);
                    // 值稳定：局部绑定（参数/局部变量）且未被改写/捕获。
                    // 全局/模块名拒绝（可在声明与使用之间变化）。
                    // 捕获不额外拒绝：嵌套 λ 内的写已由 mutated 覆盖；
                    // 自身参数在 prepare 中 lambda_depth>0 也会被标「捕获」
                    // （对 store_kill 保守正确，但会误挡同 λ 内的标量内联）。
                    if !self.is_local_ref(n) || self.mutated_names.contains(nm) {
                        return false;
                    }
                    saw_non_lit = true;
                }
                _ => return false,
            }
            for &c in self.children(n) {
                stack.push(c);
            }
        }
        let _ = saw_non_lit;
        true
    }

    fn stmt_has_opaque_region(&self, stmt: TsId) -> bool {
        let mut stack = vec![stmt];
        while let Some(n) = stack.pop() {
            match self.data(n) {
                NodeData::Raw { .. }
                | NodeData::ClassExpr { .. }
                | NodeData::Destruct { .. }
                | NodeData::DestructStmt { .. } => return true,
                _ => {}
            }
            for &c in self.children(n) {
                stack.push(c);
            }
        }
        false
    }

    fn value_may_throw_unresolved(&self, value: TsId) -> bool {
        // 自由 VarRef（未解析到本根局部绑定）可能是未声明全局——
        // 读取会抛 ReferenceError，不可随死存储一并删除。
        let mut stack = vec![value];
        while let Some(n) = stack.pop() {
            if matches!(self.data(n), NodeData::VarRef { .. }) && !self.is_local_ref(n) {
                return true;
            }
            for &c in self.children(n) {
                stack.push(c);
            }
        }
        false
    }

    fn is_tagged_template(&self, id: TsId) -> bool {
        matches!(self.data(id), NodeData::Template { tag: true, .. })
    }

    fn assign_target_paren_required(&self, inner: TsId) -> bool {
        // `(fn) = function(){}` → name ''；删括号变 'fn'（Test262
        // fn-name-lhs-cover 抓获）
        matches!(self.data(inner), NodeData::VarRef { .. })
    }

    fn dead_decl_allowed(&self, decl: TsId) -> bool {
        matches!(self.data(decl), NodeData::VarDecl { kind: VarKind::Var, .. })
    }

    fn receiver_propagation_unsound(
        &self,
        _decl: TsId,
        value: TsId,
        use_parent: Option<TsId>,
        use_at_head: bool,
    ) -> bool {
        // `const f = obj.method; f()` → `obj.method()` 改变 this 绑定
        //（Test262 Object/prototype/valueOf 家族：自由调用 valueOf 的
        // this 是 undefined，成员调用是 Object.prototype）。
        if !use_at_head {
            return false;
        }
        let Some(p) = use_parent else { return false };
        if self.kind(p) != NodeKind::Call {
            return false;
        }
        matches!(
            self.data(value),
            NodeData::Member { .. } | NodeData::Index { .. } | NodeData::Raw { .. }
        )
    }

    fn is_function_like(&self, id: TsId) -> bool {
        matches!(self.data(id), NodeData::FuncStmt { .. }) || self.kind(id) == NodeKind::Lambda
    }

    fn block_hoists_decl(&self, block: TsId) -> bool {
        // 直接孩子里含函数声明：块展开会把 Annex B 块级函数声明变成
        // 语句级（提升/初始化时机改变——Test262 annexB 族抓获）。
        self.children(block).iter().any(|&c| {
            matches!(self.data(c), NodeData::FuncStmt { .. })
                // 解构声明（DestructStmt 是 Custom，BlockFlatten 的
                // declares 看不到）——展开会合并词法作用域
                //（Test262 Iterator drop 双 let 解构变重复声明）
                || matches!(self.data(c), NodeData::DestructStmt { .. })
                // 原文语句可能声明词法绑定（`await using z = …` 以 Raw
                // 保真）——展开块会合并作用域（await-using 模块用例
                // 同块重复声明 SyntaxError 抓获）
                || matches!(self.data(c), NodeData::Raw { .. })
                || matches!(
                    self.data(c),
                    NodeData::VarDecl { kind: VarKind::Var | VarKind::Using, .. }
                )
        })
    }

    fn grammar_paren_required(&self, inner: TsId) -> bool {
        // `({x} = v)`：Assign 目标为对象字面量时语句位必需括号
        //（否则解析成块——Iterator map 用例抓获）。
        if let NodeData::Assign { .. } = self.data(inner) {
            if let Some(&t) = self.children(inner).first() {
                // ObjLit 或原文对象模式（`{__proto__ = 0} = {}` 的
                // Destruct 形态 Raw）都需要语句位括号
                if matches!(self.data(t), NodeData::ObjLit { .. })
                    || matches!(self.data(t), NodeData::Raw { text } if text.trim_start().starts_with('{'))
                {
                    return true;
                }
            }
        }
        // `(yield)`：YieldExpression 不是 ShortCircuitExpression——
        // `yield ? a : b` / `'' in yield` 非法，括号必需（ParenRemoval
        // 曾删掉它；Test262 yield/in 家族抓获）。
        if matches!(self.data(inner), NodeData::VarRef { name } if self.sn(*name) == "yield") {
            return true;
        }
        // 私有名（`#f`）在 `in` 左操作数位**不能**加括号（语法要求
        // 裸 `#x in o`；`(#x) in o` 是 SyntaxError）。
        if matches!(self.data(inner), NodeData::Raw { text } if text.starts_with('#')) {
            return false;
        }
        // `super.x` / `super[x]` / `super(...)` 原文是成员/调用链基座：
        // 优先级已是 Member，括号纯属多余。曾导致 `await super.close()`
        // 打印成 `await (super.close)()`，二轮把 await 当标识符调用
        //（await 后随 `(`）→ `await(super.close)()`，幂等差分抓获。
        if matches!(self.data(inner), NodeData::Raw { text } if text.starts_with("super")) {
            return false;
        }
        matches!(
            self.kind(inner),
            NodeKind::Lambda | NodeKind::Custom(_) | NodeKind::Raw
        )
    }

    fn var_cast_position_ok(&self, _value: TsId, _typed: TsId) -> bool {
        // TS as/satisfies 断言不改变可编译性的判断极弱（TS 结构类型），
        // 且本语言无 Cast/InstanceOf 引擎节点 → 恒 true（守卫不触发）
        true
    }

    fn prepare(&mut self, root: TsId) {
        let n_nodes = self.nodes.len();
        let key_slots = self.names.key_slots();
        if key_slots < n_nodes {
            let names: Vec<Option<String>> = (key_slots..n_nodes)
                .map(|i| self.var_name(TsId(i as u32)).map(|n| n.to_string()))
                .collect();
            self.names.extend_owned(key_slots, &names);
        }
        self.events.resize(n_nodes);
        self.stmt_facts.resize_with(n_nodes, || None);
        self.stmt_kills.resize_with(n_nodes, || None);
        self.stmt_opq.resize_with(n_nodes, || None);
        if self.events.is_clean() {
            if self.prepared_root == Some(root) {
                return;
            }
            // 换根：区域事件清洁，但**别的根**（如同类字段 init）上次
            // 简化后本根的语句事实可能从未构建——缺事实时规则退回保守
            // 扫描，λ 捕获守卫漏判（class 字段存在时 `let self = this`
            // 被内联进内层函数；Test262 private-field-access 族抓获）。
            for sr in cure_tree::stmt_roots(self, root) {
                let idx = sr.0 as usize;
                if self.stmt_facts.get(idx).map_or(true, |f| f.is_none()) {
                    self.stmt_facts[idx] = Some(self.collect_stmt_facts(sr));
                }
                if self.stmt_kills.get(idx).map_or(true, |f| f.is_none()) {
                    self.stmt_kills[idx] = Some(self.collect_stmt_kill_facts(sr));
                }
                if self.stmt_opq.get(idx).map_or(true, |f| f.is_none()) {
                    if let Some(opq) = self.stmt_opaque_fact(sr) {
                        self.stmt_opq[idx] = Some(opq);
                    }
                }
            }
            self.rebuild_local_marks(root);
            self.prepared_root = Some(root);
            return;
        }
        self.prepared_root = Some(root);
        self.events.mark_clean();
        for sr in cure_tree::stmt_roots(self, root) {
            let idx = sr.0 as usize;
            if self.events.slot(idx).is_none() {
                if let Some(events) = cure_tree::collect_region_events(self, sr) {
                    self.events.put(idx, events);
                }
            }
            if self.stmt_facts.get(idx).map_or(true, |f| f.is_none()) {
                let facts = self.collect_stmt_facts(sr);
                self.stmt_facts[idx] = Some(facts);
            }
            if self.stmt_kills.get(idx).map_or(true, |f| f.is_none()) {
                let kills = self.collect_stmt_kill_facts(sr);
                self.stmt_kills[idx] = Some(kills);
            }
            if self.stmt_opq.get(idx).map_or(true, |f| f.is_none()) {
                if let Some(opq) = self.stmt_opaque_fact(sr) {
                    self.stmt_opq[idx] = Some(opq);
                }
            }
        }
        self.refresh_effects(root, n_nodes);
        self.rebuild_local_marks(root);
    }

    fn cost_weight(&self, kind: NodeKind) -> u64 {
        match kind {
            NodeKind::Raw => 60,
            NodeKind::Custom(_) => 30,
            NodeKind::Lambda => 20,
            NodeKind::New | NodeKind::Call => 12,
            NodeKind::Ternary | NodeKind::Throw | NodeKind::Try | NodeKind::Switch
            | NodeKind::Case | NodeKind::ForEach | NodeKind::For | NodeKind::While
            | NodeKind::DoWhile | NodeKind::If | NodeKind::Label => 6,
            NodeKind::Binary | NodeKind::Unary | NodeKind::Assign | NodeKind::Index
            | NodeKind::Member | NodeKind::VarDecl | NodeKind::InstanceOf => 3,
            NodeKind::Literal | NodeKind::VarRef | NodeKind::This | NodeKind::Paren
            | NodeKind::Block | NodeKind::Empty | NodeKind::ExprStmt | NodeKind::Return
            | NodeKind::Break | NodeKind::Continue | NodeKind::Catch
            | NodeKind::Synchronized | NodeKind::Yield | NodeKind::Assert
            | NodeKind::Cast | NodeKind::MethodRef | NodeKind::NewArray
            | NodeKind::ArrayLit => 1,
        }
    }
}

// ---------------------------------------------------------------------------
// 折叠辅助
// ---------------------------------------------------------------------------

/// 原文片段是否含直接 `eval(` 调用（词边界 + 跳过空白；字符串里的
/// 同名文本会误报——只导致保守不简化，可接受）。
pub fn raw_has_direct_eval(raw: &str) -> bool {
    let b = raw.as_bytes();
    let mut i = 0;
    while let Some(p) = raw[i..].find("eval") {
        let at = i + p;
        let before_ok = at == 0 || {
            let c = b[at - 1] as char;
            !(c.is_alphanumeric() || c == '_' || c == '$')
        };
        if before_ok {
            let mut j = at + 4;
            while j < b.len() && (b[j] as char).is_whitespace() {
                j += 1;
            }
            if j < b.len() && b[j] == b'(' {
                return true;
            }
        }
        i = at + 4;
    }
    false
}

/// JS number 域精确整数：|v| ≤ 2^53。
pub fn js_int_representable(v: i64) -> bool {
    v.unsigned_abs() <= (1u64 << 53)
}

/// f64 算术的**精确整数**回验（Add/Sub/Mul/Pow/Div/Rem）。结果非整、
/// 超域或为 -0 → None（调用方决定产 Double 或拒绝；-0 一律拒绝——
/// Int 字面量无符号零，而 JS 的 1/-0 = -Infinity）。
fn arith_f64(op: BinOp, a: i64, b: i64) -> Option<Option<i64>> {
    let r = match op {
        BinOp::Add => a as f64 + b as f64,
        BinOp::Sub => a as f64 - b as f64,
        BinOp::Mul => a as f64 * b as f64,
        BinOp::Pow => {
            if !(-1024..=1024).contains(&b) {
                return None;
            }
            (a as f64).powi(b as i32)
        }
        BinOp::Div => {
            if b == 0 {
                return Some(None); // ±Inf → 拒绝（调用方直接放弃）
            }
            a as f64 / b as f64
        }
        BinOp::Rem => {
            if b == 0 {
                return Some(None);
            }
            (a as f64) % (b as f64)
        }
        _ => return None,
    };
    if r == 0.0 && r.is_sign_negative() {
        return Some(None); // -0
    }
    if r.fract() == 0.0 && r.abs() <= (1u64 << 53) as f64 {
        Some(Some(r as i64))
    } else {
        Some(None) // 非整/超域 → 调用方走 Double 分支
    }
}

/// 数值字面量是否是 IEEE 雷区值（NaN/±Inf/-0）。
fn num_hazard(x: LitRef<'_>) -> bool {
    match x {
        LitRef::Double(d) => {
            d.is_nan() || d.is_infinite() || (d == 0.0 && d.is_sign_negative())
        }
        _ => false,
    }
}

fn cmp_i64(op: BinOp, x: i64, y: i64) -> Option<bool> {
    Some(match op {
        BinOp::Lt => x < y,
        BinOp::Le => x <= y,
        BinOp::Gt => x > y,
        BinOp::Ge => x >= y,
        _ => return None,
    })
}

/// JsLit 值级相等（±0.0 位区分；BigInt/Regex 原文比较）。
fn js_lit_value_eq(x: &JsLit, y: &JsLit) -> bool {
    fn r(l: &JsLit) -> Option<LitRef<'_>> {
        Some(match l {
            JsLit::Bool(b) => LitRef::Bool(*b),
            JsLit::Int(v) => LitRef::Int(*v),
            JsLit::Double(v) => LitRef::Double(*v),
            JsLit::NumRaw { val: NumVal::Int(v), .. } => LitRef::Int(*v),
            JsLit::NumRaw { val: NumVal::Double(v), .. } => LitRef::Double(*v),
            JsLit::Str(s) => LitRef::Str(s.as_str()),
            JsLit::Null => LitRef::Null,
            JsLit::BigInt(_) | JsLit::Regex(_) => return None,
        })
    }
    match (x, y) {
        (JsLit::BigInt(a), JsLit::BigInt(b)) => a == b,
        (JsLit::Regex(a), JsLit::Regex(b)) => a == b,
        _ => match (r(x), r(y)) {
            (Some(rx), Some(ry)) => {
                if let (LitRef::Double(a), LitRef::Double(b)) = (rx, ry) {
                    a == b && a.is_sign_negative() == b.is_sign_negative()
                } else {
                    rx == ry
                }
            }
            _ => false,
        },
    }
}

// ---------------------------------------------------------------------------
// 结构化指标
// ---------------------------------------------------------------------------

/// 模块全部可执行体（函数/方法/静态块/字段 init，含嵌套）的指标总和。
pub fn unit_metrics(ast: &TsAst, unit: &ModuleUnit) -> TreeMetrics {
    let mut total = TreeMetrics::default();
    for item in &unit.items {
        match item {
            TopItem::Stmt(s) => {
                total.add(subtree_metrics(ast, *s));
            }
            TopItem::Func { body: Some(b), .. } => {
                total.add(subtree_metrics(ast, *b));
            }
            TopItem::Class(c) => metrics_of_class(ast, c, &mut total),
            _ => {}
        }
    }
    total
}

fn metrics_of_class(ast: &TsAst, c: &ClassDecl, total: &mut TreeMetrics) {
    for m in &c.members {
        match m {
            ClassMember::Method { body: Some(b), .. } => {
                total.add(subtree_metrics(ast, *b));
            }
            ClassMember::Field { init: Some(i), .. } => {
                total.add(subtree_metrics(ast, *i));
            }
            ClassMember::StaticBlock { body } => {
                total.add(subtree_metrics(ast, *body));
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------


// ---------------------------------------------------------------------------
// ECMA Number::toString（数字字面形态——打印器与规则层共用）
// ---------------------------------------------------------------------------

/// ECMA Number::toString：最短往返数字（Rust `{:e}` 与 JS 同源算法）+
/// ECMA 指数阈值（|v| ≥ 1e21 或 < 1e-6 走指数形态）。
/// ±0（"-0"）/ NaN / ±Infinity 按 JS 字面形态输出。
pub fn format_js_number(v: f64) -> String {
    if v == 0.0 {
        return if v.is_sign_negative() { "-0".to_string() } else { "0".to_string() };
    }
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v < 0.0 { "-Infinity".to_string() } else { "Infinity".to_string() };
    }
    let s = format!("{:e}", v);
    let (mant, exp_s) = s.split_once('e').expect("exponent form");
    let exp: i32 = exp_s.parse().expect("exponent");
    let neg = mant.starts_with('-');
    let mant = mant.trim_start_matches('-');
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    let k = digits.len() as i32;
    let n = exp + 1;
    let body = if k <= n && n <= 21 {
        format!("{}{}", digits, "0".repeat((n - k) as usize))
    } else if 0 < n && n <= 21 {
        format!("{}.{}", &digits[..n as usize], &digits[n as usize..])
    } else if -6 < n && n <= 0 {
        format!("0.{}{}", "0".repeat((-n) as usize), digits)
    } else if k == 1 {
        format!("{}e{}{}", digits, if exp >= 0 { "+" } else { "-" }, exp.abs())
    } else {
        format!("{}.{}e{}{}", &digits[..1], &digits[1..], if exp >= 0 { "+" } else { "-" }, exp.abs())
    };
    if neg {
        format!("-{body}")
    } else {
        body
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_smoke() {
        let mut a = TsAst::new();
        let one = a.lit(JsLit::Int(1));
        let x = a.var_decl("x", VarKind::Let, None, Some(one));
        let xv = a.var("x");
        let r = a.ret(Some(xv));
        let body = a.block(vec![x, r]);
        assert_eq!(a.kind(body), NodeKind::Block);
        assert_eq!(a.children(body).len(), 2);
    }

    #[test]
    fn effect_and_locality() {
        let mut a = TsAst::new();
        let one = a.lit(JsLit::Int(1));
        let dx = a.var_decl("x", VarKind::Let, None, Some(one));
        let xv = a.var("x");
        let r = a.ret(Some(xv));
        let body = a.block(vec![dx, r]);
        a.prepare(body);
        assert!(a.is_local_var(xv));
        // console 是全局——非局部
        let c = a.var("console");
        assert!(!a.is_local_var(c));
    }

    #[test]
    fn js_int_folds() {
        let mut a = TsAst::new();
        let two = a.lit(JsLit::Int(2));
        let three = a.lit(JsLit::Int(3));
        let add = a.bin(BinOp::Add, two, three);
        a.prepare(add);
        // 直接调用 fold 钩子
        let folded = <TsAst as Lang>::fold_lit_bin(&mut a, BinOp::Add, two, three);
        assert!(matches!(a.literal(folded.unwrap()), Some(LitRef::Int(5))));
        // 2 ** 53 边界：(2^53-1)+2 = 2^53 恰好精确（f64 可表示）——
        // 与 JS 运行时一致，允许折成 Int(2^53)
        let big = a.lit(JsLit::Int((1i64 << 53) - 1));
        let two_b = a.lit(JsLit::Int(2));
        let folded = <TsAst as Lang>::fold_lit_bin(&mut a, BinOp::Add, big, two_b).unwrap();
        assert!(matches!(a.literal(folded), Some(LitRef::Int(v)) if v == 1i64 << 53));
        // 超出 2^53 的操作数（不可精确表示）拒绝折叠
        let huge = a.lit(JsLit::Int(1i64 << 60));
        assert!(<TsAst as Lang>::fold_lit_bin(&mut a, BinOp::Add, huge, two_b).is_none());
        // 移位走 i32 环绕域：1 << 31 = -2147483648（JS ToInt32 环绕）
        let one_l = a.lit(JsLit::Int(1));
        let sh31 = a.lit(JsLit::Int(31));
        let folded = <TsAst as Lang>::fold_lit_bin(&mut a, BinOp::Shl, one_l, sh31).unwrap();
        assert!(matches!(a.literal(folded), Some(LitRef::Int(-2147483648))));
        // 负零敏感：-2 * 0 = -0 —— 拒绝（Int 无符号零）
        let neg2 = a.lit(JsLit::Int(-2));
        let zero = a.lit(JsLit::Int(0));
        assert!(<TsAst as Lang>::fold_lit_bin(&mut a, BinOp::Mul, neg2, zero).is_none());
        // NaN 不折
        let nan = a.lit(JsLit::Double(f64::NAN));
        let one = a.lit(JsLit::Int(1));
        assert!(<TsAst as Lang>::fold_lit_bin(&mut a, BinOp::Add, nan, one).is_none());
        // 溢出环绕：0xFFFFFFFF + 1 按 f64 精确 = 4294967296（不是 i32 环绕）
        let f = a.lit(JsLit::Int(0xFFFF_FFFF));
        let o = a.lit(JsLit::Int(1));
        let folded = <TsAst as Lang>::fold_lit_bin(&mut a, BinOp::Add, f, o);
        assert!(matches!(a.literal(folded.unwrap()), Some(LitRef::Int(4294967296))));
    }
}

// ---------------------------------------------------------------------------
// 结构等价的归一化辅助（打印往返校验；不参与语义判定）
// ---------------------------------------------------------------------------

/// 打印器会把 Group 展开成多条同缩进语句（必要时包 Block）。
fn is_stmt_container(d: &NodeData) -> bool {
    matches!(d, NodeData::Block | NodeData::Group | NodeData::ModuleBlock)
}

/// 顶层项展平后的视图（Group/多声明符与打印后的多条语句对齐）。
enum FlatTop<'a> {
    Item(&'a TopItem),
    Stmt { export: bool, id: TsId },
}

/// 原文/签名归一化：去掉首尾空白与尾部 `;`，仅保留标识符之间的空白
///（标点周围的空白不影响 token 流）。字符串内的空白可能被折叠——这只
/// 用于等价校验，宁可漏报不误报。
fn norm_code(s: &str) -> String {
    let t = s.trim().trim_end_matches(';');
    let mut out = String::with_capacity(t.len());
    let mut pending_space = false;
    let mut prev_ident = false;
    for c in t.chars() {
        if c.is_whitespace() {
            pending_space = true;
            continue;
        }
        let is_ident =
            c.is_alphanumeric() || c == '_' || c == '$' || c == '#' || c == '\\';
        if pending_space && prev_ident && is_ident {
            out.push(' ');
        }
        out.push(c);
        prev_ident = is_ident;
        pending_space = false;
    }
    out
}

fn norm_opt(s: &Option<String>) -> Option<String> {
    s.as_deref().map(norm_code)
}

/// 数值字面量按值比较：`1e3`/`0x3e8`/`1000` 等价；非整 Double 走位比较。
fn num_parts(l: &JsLit) -> Option<(f64, Option<i64>)> {
    let (v, as_i) = match l {
        JsLit::Int(v) => (*v as f64, Some(*v)),
        JsLit::Double(v) => {
            let exact = v.is_finite()
                && v.fract() == 0.0
                && v.abs() <= (1u64 << 53) as f64;
            (*v, if exact { Some(*v as i64) } else { None })
        }
        JsLit::NumRaw { val, .. } => match val {
            NumVal::Int(v) => (*v as f64, Some(*v)),
            NumVal::Double(v) => {
                let exact = v.is_finite()
                    && v.fract() == 0.0
                    && v.abs() <= (1u64 << 53) as f64;
                (*v, if exact { Some(*v as i64) } else { None })
            }
        },
        _ => return None,
    };
    Some((v, as_i))
}

fn lit_equiv(a: &JsLit, b: &JsLit) -> bool {
    if let (Some((fa, ia)), Some((fb, ib))) = (num_parts(a), num_parts(b)) {
        return match (ia, ib) {
            (Some(x), Some(y)) => x == y,
            _ => fa.to_bits() == fb.to_bits(),
        };
    }
    match (a, b) {
        (JsLit::Bool(x), JsLit::Bool(y)) => x == y,
        (JsLit::Str(x), JsLit::Str(y)) => x == y,
        (JsLit::Regex(x), JsLit::Regex(y)) => x == y,
        (JsLit::BigInt(x), JsLit::BigInt(y)) => {
            let nx: String = x.chars().filter(|c| *c != '_').collect();
            let ny: String = y.chars().filter(|c| *c != '_').collect();
            nx == ny
        }
        (JsLit::Null, JsLit::Null) => true,
        _ => false,
    }
}

impl TsAst {
    /// 增量 effect 缓存：首次/换根补算可达节点；之后只重算 invalidate
    /// 标脏的子树。旧实现每 pass 全量 `rebuild_effects`（4×全部节点），
    /// 13MB 单文件 9k pass 时是主要 CPU 来源。
    fn refresh_effects(&mut self, root: TsId, n: usize) {
        if self.effect_cache.len() < n {
            self.effect_cache.resize(n, None);
        }
        if self.effect_dirty_mark.len() < n {
            self.effect_dirty_mark.resize(n, false);
        }
        if self.effects_root != Some(root) {
            // 可达节点后序计算；已缓存子树直接跳过（跨根共享/离开的
            // 节点不失效）。
            let mut stack = vec![(root, false)];
            while let Some((id, exiting)) = stack.pop() {
                let idx = id.0 as usize;
                if idx >= n {
                    continue;
                }
                if exiting {
                    if self.effect_cache[idx].is_none() {
                        let mut e = <TsAst as Lang>::own_effect(self, id);
                        for &c in self.nodes[idx].children.as_slice() {
                            e = e.worst(
                                self.effect_cache
                                    .get(c.0 as usize)
                                    .and_then(|x| *x)
                                    .unwrap_or(Effect::Unknown),
                            );
                        }
                        self.effect_cache[idx] = Some(e);
                    }
                } else if self.effect_cache[idx].is_none() {
                    stack.push((id, true));
                    for &c in self.nodes[idx].children.as_slice() {
                        stack.push((c, false));
                    }
                }
            }
            self.effects_root = Some(root);
        }
        let dirty = std::mem::take(&mut self.effect_dirty);
        for id in dirty {
            let idx = id.0 as usize;
            if idx < self.effect_dirty_mark.len() {
                self.effect_dirty_mark[idx] = false;
            }
            if idx >= n || self.effect_cache[idx].is_some() {
                continue;
            }
            let mut stack = vec![(id, false)];
            while let Some((x, exiting)) = stack.pop() {
                let xi = x.0 as usize;
                if xi >= n {
                    continue;
                }
                if exiting {
                    if self.effect_cache[xi].is_none() {
                        let mut e = <TsAst as Lang>::own_effect(self, x);
                        for &c in self.nodes[xi].children.as_slice() {
                            e = e.worst(
                                self.effect_cache
                                    .get(c.0 as usize)
                                    .and_then(|v| *v)
                                    .unwrap_or(Effect::Unknown),
                            );
                        }
                        self.effect_cache[xi] = Some(e);
                    }
                } else if self.effect_cache[xi].is_none() {
                    stack.push((x, true));
                    for &c in self.nodes[xi].children.as_slice() {
                        stack.push((c, false));
                    }
                }
            }
        }
    }

}
