//! # cure-java-ast
//!
//! Java AST（arena + NodeId）+ 编译单元/成员结构 + builder + [`cure_engine::Lang`] 实现。
//!
//! 覆盖反编译器产出的常见 Java：完整语句集（含 try/catch、switch、for-each、
//! lambda 等）、完整表达式集、类/接口/枚举成员。签名里的泛型/注解以**原文**字符串
//! 保留（保真打印），方法体内才是可简化的 arena 节点。
//!
//! 零外部依赖（仅依赖 cure-engine），可单独作为"Java AST + 语义查询"基础库引用。
//!
//! arena 不变量：children 永远先于父节点入 arena（index 递增）。

use std::collections::{HashMap, HashSet};

use cure_engine::analysis::{subtree_metrics, TreeMetrics};
use cure_engine::lang::ReassocOutcome;
use cure_engine::Lang;
pub use cure_tree::ChildList;
use cure_tree::{EventStore, NameTable};

// 本 crate 自用 + 供下游 crate（如 cure-java-print/parser）免依赖引擎直接使用
pub use cure_engine::kind::{BinOp, LitRef, NodeKind, UnOp};
pub use cure_engine::Effect;

// ---------------------------------------------------------------------------
// 基础类型
// ---------------------------------------------------------------------------

/// arena 句柄。
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct JavaId(pub u32);
impl Default for JavaId {
    fn default() -> Self {
        JavaId(u32::MAX)
    }
}

/// Java 类型（第一阶段的最小集合；`Ref` 存类型名字符串，泛型参数并入字符串）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum JType {
    Bool,
    Byte,
    Short,
    Int,
    Long,
    Char,
    Float,
    Double,
    Void,
    /// 引用类型，存点分名或简单名（可含泛型参数原文，如 `Map<String, ?>`）。
    Ref(String),
    Array(Box<JType>),
    /// var / 推断类型 / 未知。
    Var,
}

impl JType {
    pub fn is_ref(&self) -> bool {
        matches!(self, JType::Ref(_) | JType::Array(_))
    }
    pub fn is_integral(&self) -> bool {
        matches!(
            self,
            JType::Byte | JType::Short | JType::Int | JType::Long | JType::Char
        )
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, JType::Bool)
    }
}

/// 数值字面量的解析值（用于语义查询；原文见 [`Lit::NumRaw`]）。
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NumVal {
    Int(i64),
    Long(i64),
    Float(f64),
    Double(f64),
}

/// 字面量。
#[derive(Clone, PartialEq, Debug)]
pub enum Lit {
    Bool(bool),
    Int(i64),
    Long(i64),
    Float(f64),
    Double(f64),
    Char(char),
    Str(String),
    /// 文本块（`"""..."""`），存去引号后的原文。
    TextBlock(String),
    /// 带原文的数值字面量：保留 0x1F / 1.5e2f / 1_000_000L 等原文形式。
    NumRaw { text: String, val: NumVal },
    Null,
}

/// 节点负载：children 统一放在 [`Node::children`]，这里只放非节点数据。
#[derive(Clone, PartialEq, Debug)]
pub enum NodeData {
    // ---- 语句 ----
    Block,
    /// 合成分组（无花括号、**无作用域**）：多声明符等"一句多语句"的承载。
    /// 语句列表处在解析期就地展开；若存活到打印，按同缩进无括号输出。
    /// kind() 映射为 Block（引擎规则透明），但语义上不引入词法作用域。
    Group,
    Empty,
    ExprStmt,
    VarDecl { name: String, ty: JType },
    Assign { op: Option<BinOp> },
    If,
    While,
    DoWhile,
    For { inits: u8, has_cond: bool, steps: u8 },
    ForEach { name: String, ty: JType },
    Return,
    Break { label: Option<String> },
    Continue { label: Option<String> },
    Throw,
    /// children: `[resource…, try_block, catch…, (finally)?]`
    Try,
    /// `catch (ty_raw name)`，children: `[block]`
    Catch { ty_raw: String, name: String },
    /// children: `[lock, block]`
    Synchronized,
    /// children: `[subject, case…]`
    Switch,
    /// children: `[label…, stmts…]`（前 `labels` 个是 case 标签表达式）
    Case { labels: u16, is_default: bool, arrow: bool },
    /// children: `[stmt]`
    Label { name: String },
    /// children: `[cond, (msg)?]`
    Assert,
    /// 不可解析的原文区域：原样保留，简化规则不触碰。
    Raw { text: String },

    // ---- 表达式 ----
    Binary { op: BinOp },
    Unary { op: UnOp },
    /// children: `[callee, arg…]`
    Call,
    /// children: `[object]`；`name` 为成员名。
    Member { name: Sym },
    /// children: `[array, index]`
    Index,
    Cast { ty: JType },
    Paren,
    Ternary,
    /// `new T(args…)`；匿名类体以原文附加。
    New { ty: JType, anon_raw: Option<String> },
    /// `new T[sz…][]… [init]`；ty 为元素类型，dims=总维数，sized=带尺寸维数；
    /// children: `[size…, (ArrayLit)?]`
    NewArray { ty: JType, dims: u16, sized: u16 },
    ArrayLit,
    VarRef { name: Sym },
    Literal(Lit),
    This,
    /// `super`（super.foo() / super(args) 的 callee）
    Super,
    /// children: `[expr]`；`x instanceof T bind`
    InstanceOf { ty: JType, bind: Option<String> },
    /// children: `[body]`（body 为 Block 或表达式）
    Lambda { params_raw: String },
    /// children: `[receiver]`；`recv::name`（name 含 `new`）
    MethodRef { name: Sym },
}

#[derive(Clone, PartialEq, Debug)]
pub struct Node {
    pub data: NodeData,
    pub children: ChildList<JavaId>,
}

/// 符号（intern 名字 id）。VarRef/Member/MethodRef 的名字字段——
/// 87% 重复率（实测 jdk-sources：4754 个名字节点仅 669 个唯一），
/// String→Sym 后：构造去重、比较 u32、clone 零分配（copy_subtree/
/// clone_node 全线受益）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Sym(pub u32);

/// 同名同元数方法候选（no-op 调用消解用）。
#[derive(Clone, Debug)]
pub struct NoopCandidate {
    /// 形参类型表。
    pub params: Vec<JType>,
    /// private + 空体（no-op）。
    pub noop: bool,
}


/// Java AST arena。只追加不回收，[`JavaId`] 永不失效。
#[derive(Default, Debug)]
pub struct JavaAst {
    pub(crate) nodes: Vec<Node>,
    /// prepare() 重建：JavaId → 聚合效果。
    /// 槽位 = arena 下标（稠密 u32）；None = 未算/失效。Vec 索引替代哈希。
    effect_cache: Vec<Option<Effect>>,
    /// prepare() 重建：VarRef 节点 → 声明类型（作用域解析）。
    /// 槽位 = arena 下标；None = 未解析。Vec 索引替代哈希。
    var_types: Vec<Option<JType>>,
    /// 方法参数类型（由 simplify 门面按方法设置，作为根作用域）。
    param_scope: Vec<(String, JType)>,
    /// 类级常量字段（static final 且字面量/字面量数组初始化、无写、无同名局部）
    /// → 初始化节点。由 simplify_unit 填充（跨方法解密的字符串表）。
    pub const_fields: HashMap<String, JavaId>,
    /// 可安全做 VarRef → 字面量传播的常量字段名（标量字面量且声明类型
    /// 与字面量种类精确匹配——过载解析/常量池身份不受影响）。
    /// 数组常量不进（只走下标折叠，整表引用不复制）。
    pub const_scalars: HashSet<String>,
    /// 空私有方法（no-op 调用清理用）：名字 → 元数 → 同名同元数全部
    /// 候选（参数类型表 + 是否 private 空体）。调用点做类型适用性消解
    ///（唯一适用者且为 private 空体 → no-op）。同名任意方法含 varargs →
    /// 该名整体放弃（varargs 可吸收任意元数）。由 simplify_unit 填充
    ///（--dead-code 门控）。
    pub noop_private_methods: HashMap<String, HashMap<usize, Vec<NoopCandidate>>>,
    /// 字段声明类型表（名字 → JType；同名二见移除）——调用点实参静态
    /// 类型解析用（VarRef 解析不到局部时查字段）。collect_unit_consts 填充。
    pub field_types: HashMap<String, JType>,
    /// final 字段名集合（同名二见移除——保守）。static_exec 截断守卫
    /// 用：前缀字段写被 rest 再赋值时，仅 final 才放弃（非 final 的
    /// 材料化写是死写，语义恒等、可编译）。
    pub final_fields: HashSet<String>,
    /// 可内联的单 return 方法：名字 → (参数名表, 返回表达式节点)。
    /// 由 simplify_unit 填充（解密 helper：d(0) → 方法体）。
    pub inline_methods: HashMap<String, (Vec<String>, JavaId)>,
    /// String 引用身份比较缓存：(root, 结果)。prepare() 清空（树已变），
    /// 首次查询时计算——供 new String(lit) 等折叠守卫复用（每轮至多一次全扫）。
    string_identity: std::cell::Cell<Option<(JavaId, bool)>>,
    /// 名字 intern 表 + 节点名字键（cure-tree `NameTable`；本语言
    /// `Lang::NameKey = u32` intern id）。单元级持久、只增；prepare
    /// 增量扩展新节点，既有节点键与其名字恒同。
    names: NameTable,
    /// 区域事件索引（cure-tree `EventStore`）：B3 惰性重建 + 编辑失效。
    /// 供引擎 scan_region 快路径——收集器在 cure-tree，只依赖 Lang
    /// 钩子，遍历序与引擎原递归严格一致。
    events: EventStore<JavaId, u32>,
    /// 上次 prepare 的根。B3 早退（events_clean）只对**树全局**缓存
    /// （事件索引/效果表）合法；var_types 是 **root 相关**的作用域分析
    /// ——根变化时必须重建，否则 is_local_var 全 false、传播规则停摆
    /// （deobfuscate 差分抓获：`t = bump(2)` 不再内联）。
    prepared_root: Option<JavaId>,
}

// ---------------------------------------------------------------------------
// 编译单元 / 成员（签名层：签名保真、方法体进 arena）
// ---------------------------------------------------------------------------

/// 方法/构造器参数。
#[derive(Clone, PartialEq, Debug)]
pub struct Param {
    /// `final` / 注解等原文（可为空串）。
    pub mods: String,
    pub ty: JType,
    /// `int... args`
    pub varargs: bool,
    pub name: String,
}

/// 字段声明中的一个声明符（`int a = 1, b[];` 里的 a、b）。
#[derive(Clone, PartialEq, Debug)]
pub struct Declarator {
    pub name: String,
    /// C 风格后缀维度 `int a[][]` 中名字后面的 `[]` 数。
    pub extra_dims: u16,
    /// 初始化表达式（arena 节点）。
    pub init: Option<JavaId>,
}

/// 类成员。
#[derive(Clone, PartialEq, Debug)]
pub enum Member {
    Field { mods: String, ty: JType, declarators: Vec<Declarator> },
    Method {
        mods: String,
        /// 泛型参数原文（含尖括号，可为空串）。
        ty_params: String,
        ret: JType,
        name: String,
        params: Vec<Param>,
        throws: Vec<String>,
        /// 方法体 Block；None 表示 abstract/native/接口声明。可能是 Raw 节点。
        body: Option<JavaId>,
    },
    Constructor {
        mods: String,
        ty_params: String,
        name: String,
        params: Vec<Param>,
        throws: Vec<String>,
        body: Option<JavaId>,
        /// record 紧凑构造器：`Name { … }`（无参数列表，签名继承自 record 头）
        compact: bool,
    },
    /// 实例/静态初始化块。
    Initializer { is_static: bool, body: JavaId },
    /// 嵌套类型。
    Type(Box<TypeDecl>),
    /// 不可解析成员（原文保真）。
    Raw(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TypeKind {
    Class,
    Interface,
    Enum,
    Record,
    Annotation,
}

/// 类型声明。
#[derive(Clone, PartialEq, Debug)]
pub struct TypeDecl {
    pub kind: TypeKind,
    /// 修饰符 + 注解原文。
    pub mods: String,
    pub name: String,
    /// 泛型参数原文（含尖括号，可为空串）；record 的头部参数原文存在 header。
    pub ty_params: String,
    /// record 组件头原文（`(int a, String b)`），非 record 为空串。
    pub header: String,
    pub extends: Vec<String>,
    pub implements: Vec<String>,
    /// sealed 类型的 permits 子句原文列表（打印 `permits A, B`）。
    pub permits: Vec<String>,
    /// 枚举常量原文（解析失败时兜底保真）。
    pub enum_constants: Vec<String>,
    pub members: Vec<Member>,
}

/// 编译单元。
#[derive(Clone, PartialEq, Debug, Default)]
pub struct CompilationUnit {
    /// package 名（不含 `package`/`;`）。
    pub package: Option<String>,
    /// import 项原文（如 `java.util.List` / `static x.y.*`）。
    pub imports: Vec<String>,
    pub types: Vec<TypeDecl>,
    /// 顶层不可解析区域原文（按出现顺序保真）。
    pub raws: Vec<String>,
}

/// 解析错误（位置为 1-based 行/列）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

// ---------------------------------------------------------------------------
// JavaAst 主体
// ---------------------------------------------------------------------------

impl JavaAst {
    pub fn new() -> Self {
        Self::default()
    }

    /// 以 node 的负载 + 指定 children 构造新节点（深拷贝/替换用）。
    pub fn clone_node(&mut self, id: JavaId, children: Vec<JavaId>) -> JavaId {
        let data = self.nodes[id.0 as usize].data.clone();
        self.push(data, children)
    }

    fn push(&mut self, data: NodeData, children: Vec<JavaId>) -> JavaId {
        let id = JavaId(self.nodes.len() as u32);
        self.nodes.push(Node { data, children: ChildList::from_vec(children) });
        id
    }

    pub fn node(&self, id: JavaId) -> &Node {
        &self.nodes[id.0 as usize]
    }
    pub fn data(&self, id: JavaId) -> &NodeData {
        &self.node(id).data
    }

    /// 固有方法（与 Lang trait 同名；供不依赖引擎的下游使用，如 printer）。
    pub fn kind(&self, id: JavaId) -> NodeKind {
        match &self.nodes[id.0 as usize].data {
            NodeData::Block | NodeData::Group => NodeKind::Block,
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
            NodeData::Synchronized => NodeKind::Synchronized,
            NodeData::Switch => NodeKind::Switch,
            NodeData::Case { .. } => NodeKind::Case,
            NodeData::Label { .. } => NodeKind::Label,
            NodeData::Assert => NodeKind::Assert,
            NodeData::Raw { .. } => NodeKind::Raw,
            NodeData::Binary { .. } => NodeKind::Binary,
            NodeData::Unary { .. } => NodeKind::Unary,
            NodeData::Call => NodeKind::Call,
            NodeData::Member { .. } => NodeKind::Member,
            NodeData::Index => NodeKind::Index,
            NodeData::Cast { .. } => NodeKind::Cast,
            NodeData::Paren => NodeKind::Paren,
            NodeData::Ternary => NodeKind::Ternary,
            NodeData::New { .. } => NodeKind::New,
            NodeData::NewArray { .. } => NodeKind::NewArray,
            NodeData::ArrayLit => NodeKind::ArrayLit,
            NodeData::VarRef { .. } => NodeKind::VarRef,
            NodeData::Literal(_) => NodeKind::Literal,
            NodeData::This => NodeKind::This,
            NodeData::Super => NodeKind::This,
            NodeData::InstanceOf { .. } => NodeKind::InstanceOf,
            NodeData::Lambda { .. } => NodeKind::Lambda,
            NodeData::MethodRef { .. } => NodeKind::MethodRef,
        }
    }
    /// arena 节点总数（诊断/深度测量用）。
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn children(&self, id: JavaId) -> &[JavaId] {
        self.nodes[id.0 as usize].children.as_slice()
    }

    // ---- builder：语句 ----

    pub fn block(&mut self, stmts: Vec<JavaId>) -> JavaId {
        self.push(NodeData::Block, stmts)
    }
    pub fn empty(&mut self) -> JavaId {
        self.push(NodeData::Empty, vec![])
    }
    pub fn expr_stmt(&mut self, e: JavaId) -> JavaId {
        self.push(NodeData::ExprStmt, vec![e])
    }
    pub fn var_decl(&mut self, name: &str, ty: JType, init: Option<JavaId>) -> JavaId {
        self.push(
            NodeData::VarDecl {
                name: name.into(),
                ty,
            },
            init.into_iter().collect(),
        )
    }
    pub fn assign(&mut self, target: JavaId, value: JavaId) -> JavaId {
        self.push(NodeData::Assign { op: None }, vec![target, value])
    }
    pub fn assign_op(&mut self, op: BinOp, target: JavaId, value: JavaId) -> JavaId {
        self.push(NodeData::Assign { op: Some(op) }, vec![target, value])
    }
    pub fn if_(&mut self, cond: JavaId, then: JavaId, els: Option<JavaId>) -> JavaId {
        let mut ch = vec![cond, then];
        ch.extend(els);
        self.push(NodeData::If, ch)
    }
    pub fn while_(&mut self, cond: JavaId, body: JavaId) -> JavaId {
        self.push(NodeData::While, vec![cond, body])
    }
    pub fn do_while(&mut self, body: JavaId, cond: JavaId) -> JavaId {
        self.push(NodeData::DoWhile, vec![body, cond])
    }
    pub fn for_(
        &mut self,
        inits: Vec<JavaId>,
        cond: Option<JavaId>,
        steps: Vec<JavaId>,
        body: JavaId,
    ) -> JavaId {
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
    pub fn for_each(&mut self, name: &str, ty: JType, iterable: JavaId, body: JavaId) -> JavaId {
        self.push(
            NodeData::ForEach {
                name: name.into(),
                ty,
            },
            vec![iterable, body],
        )
    }
    pub fn ret(&mut self, value: Option<JavaId>) -> JavaId {
        self.push(NodeData::Return, value.into_iter().collect())
    }
    pub fn break_(&mut self, label: Option<&str>) -> JavaId {
        self.push(
            NodeData::Break {
                label: label.map(Into::into),
            },
            vec![],
        )
    }
    pub fn continue_(&mut self, label: Option<&str>) -> JavaId {
        self.push(
            NodeData::Continue {
                label: label.map(Into::into),
            },
            vec![],
        )
    }
    pub fn throw(&mut self, e: JavaId) -> JavaId {
        self.push(NodeData::Throw, vec![e])
    }
    pub fn try_(
        &mut self,
        resources: Vec<JavaId>,
        try_block: JavaId,
        catches: Vec<JavaId>,
        finally: Option<JavaId>,
    ) -> JavaId {
        let mut ch = resources;
        ch.push(try_block);
        ch.extend(catches);
        ch.extend(finally);
        self.push(NodeData::Try, ch)
    }
    pub fn catch_(&mut self, ty_raw: &str, name: &str, block: JavaId) -> JavaId {
        self.push(
            NodeData::Catch {
                ty_raw: ty_raw.into(),
                name: name.into(),
            },
            vec![block],
        )
    }
    pub fn synchronized(&mut self, lock: JavaId, block: JavaId) -> JavaId {
        self.push(NodeData::Synchronized, vec![lock, block])
    }
    pub fn switch_(&mut self, subject: JavaId, cases: Vec<JavaId>) -> JavaId {
        let mut ch = vec![subject];
        ch.extend(cases);
        self.push(NodeData::Switch, ch)
    }
    pub fn case_(
        &mut self,
        labels: Vec<JavaId>,
        is_default: bool,
        arrow: bool,
        stmts: Vec<JavaId>,
    ) -> JavaId {
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
    pub fn label(&mut self, name: &str, stmt: JavaId) -> JavaId {
        self.push(NodeData::Label { name: name.into() }, vec![stmt])
    }
    pub fn assert_(&mut self, cond: JavaId, msg: Option<JavaId>) -> JavaId {
        let mut ch = vec![cond];
        ch.extend(msg);
        self.push(NodeData::Assert, ch)
    }
    pub fn raw(&mut self, text: &str) -> JavaId {
        self.push(NodeData::Raw { text: text.into() }, vec![])
    }

    // ---- builder：表达式 ----

    pub fn bin(&mut self, op: BinOp, l: JavaId, r: JavaId) -> JavaId {
        self.push(NodeData::Binary { op }, vec![l, r])
    }
    pub fn un(&mut self, op: UnOp, e: JavaId) -> JavaId {
        self.push(NodeData::Unary { op }, vec![e])
    }
    pub fn call(&mut self, callee: JavaId, args: Vec<JavaId>) -> JavaId {
        let mut ch = vec![callee];
        ch.extend(args);
        self.push(NodeData::Call, ch)
    }
    pub fn member(&mut self, obj: JavaId, name: &str) -> JavaId {
        let sym = self.intern_sym(name);
        self.push(NodeData::Member { name: sym }, vec![obj])
    }
    pub fn index(&mut self, arr: JavaId, idx: JavaId) -> JavaId {
        self.push(NodeData::Index, vec![arr, idx])
    }
    pub fn cast(&mut self, ty: JType, e: JavaId) -> JavaId {
        self.push(NodeData::Cast { ty }, vec![e])
    }
    pub fn paren(&mut self, e: JavaId) -> JavaId {
        self.push(NodeData::Paren, vec![e])
    }
    pub fn ternary(&mut self, c: JavaId, a: JavaId, b: JavaId) -> JavaId {
        self.push(NodeData::Ternary, vec![c, a, b])
    }
    pub fn new_(&mut self, ty: JType, args: Vec<JavaId>) -> JavaId {
        self.push(NodeData::New { ty, anon_raw: None }, args)
    }
    pub fn new_anon(&mut self, ty: JType, args: Vec<JavaId>, body_raw: String) -> JavaId {
        self.push(
            NodeData::New {
                ty,
                anon_raw: Some(body_raw),
            },
            args,
        )
    }
    pub fn new_array(
        &mut self,
        ty: JType,
        dims: u16,
        sized: u16,
        sizes: Vec<JavaId>,
        init: Option<JavaId>,
    ) -> JavaId {
        let mut ch = sizes;
        ch.extend(init);
        self.push(NodeData::NewArray { ty, dims, sized }, ch)
    }
    /// 合成分组（无作用域）。
    pub fn group(&mut self, stmts: Vec<JavaId>) -> JavaId {
        self.push(NodeData::Group, stmts)
    }
    pub fn array_lit(&mut self, elems: Vec<JavaId>) -> JavaId {
        self.push(NodeData::ArrayLit, elems)
    }
    pub fn var(&mut self, name: &str) -> JavaId {
        let sym = self.intern_sym(name);
        self.push(NodeData::VarRef { name: sym }, vec![])
    }
    pub fn lit(&mut self, l: Lit) -> JavaId {
        self.push(NodeData::Literal(l), vec![])
    }
    fn build_int_impl(&mut self, v: i64, long: bool) -> JavaId {
        if long {
            self.lit(Lit::Long(v))
        } else {
            self.lit(Lit::Int(v))
        }
    }
    pub fn this(&mut self) -> JavaId {
        self.push(NodeData::This, vec![])
    }
    pub fn super_(&mut self) -> JavaId {
        self.push(NodeData::Super, vec![])
    }
    pub fn instance_of(&mut self, e: JavaId, ty: JType, bind: Option<&str>) -> JavaId {
        self.push(
            NodeData::InstanceOf {
                ty,
                bind: bind.map(Into::into),
            },
            vec![e],
        )
    }
    pub fn lambda(&mut self, params_raw: &str, body: JavaId) -> JavaId {
        self.push(
            NodeData::Lambda {
                params_raw: params_raw.into(),
            },
            vec![body],
        )
    }
    pub fn method_ref(&mut self, receiver: JavaId, name: &str) -> JavaId {
        let sym = self.intern_sym(name);
        self.push(NodeData::MethodRef { name: sym }, vec![receiver])
    }

    // ---- 便捷 ----

    /// `obj.name(args…)` 调用。
    pub fn method_call(&mut self, obj: JavaId, name: &str, args: Vec<JavaId>) -> JavaId {
        let m = self.member(obj, name);
        self.call(m, args)
    }
    /// 裸方法调用 `name(args…)`（隐式 this / static）。
    pub fn plain_call(&mut self, name: &str, args: Vec<JavaId>) -> JavaId {
        let f = self.var(name);
        self.call(f, args)
    }

    /// 作用域解析后的 VarRef 类型（prepare 之后有效）。
    pub fn var_type(&self, id: JavaId) -> Option<&JType> {
        self.var_types.get(id.0 as usize).and_then(|t| t.as_ref())
    }

    /// root 子树内是否存在「两侧都可能为 String」的 ==/!=（引用比较）？
    /// 折叠守卫用：new String(lit)/常量链等产出池化字面量会改变 == 语义。
    /// 结果按 (root, prepare 代次) 缓存——prepare 每次改树后清空，
    /// 首次查询全扫一次，同轮内所有守卫调用复用。
    pub fn has_string_identity_compare(&self, root: JavaId) -> bool {
        if let Some((r, v)) = self.string_identity.get() {
            if r == root {
                return v;
            }
        }
        let v = Self::scan_string_identity(self, root);
        self.string_identity.set(Some((root, v)));
        v
    }

    fn scan_string_identity(&self, root: JavaId) -> bool {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            if let NodeData::Binary { op: BinOp::Eq | BinOp::Ne } = self.data(id) {
                let ch = self.children(id);
                if ch.len() == 2
                    && self.expr_maybe_string(ch[0])
                    && self.expr_maybe_string(ch[1])
                {
                    return true;
                }
            }
            for &c in self.children(id) {
                stack.push(c);
            }
        }
        false
    }

    /// `==`/`!=` 的这个操作数可能持有 String 引用吗？（保守近似）
    fn expr_maybe_string(&self, id: JavaId) -> bool {
        match self.data(id) {
            NodeData::Literal(Lit::Str(_)) => true,
            // null/this/数组/instanceof/数值布尔字面量：折叠不可能改变其 == 结果
            NodeData::Literal(_) | NodeData::This | NodeData::Super
            | NodeData::NewArray { .. } | NodeData::ArrayLit | NodeData::InstanceOf { .. } => false,
            NodeData::VarRef { .. } => self.var_type(id).map_or(true, Self::type_maybe_string),
            NodeData::Cast { ty } => Self::type_maybe_string(ty),
            NodeData::Paren => self
                .children(id)
                .first()
                .map_or(true, |&c| self.expr_maybe_string(c)),
            // 拼接：任一操作数 String ⇒ 结果 String；其余二元运算结果必为原始类型
            NodeData::Binary { op } if *op == BinOp::Add => {
                self.children(id).iter().any(|&c| self.expr_maybe_string(c))
            }
            NodeData::Binary { .. } => false,
            NodeData::Ternary => self
                .children(id)
                .get(1..)
                .map_or(true, |cs| cs.iter().any(|&c| self.expr_maybe_string(c))),
            // Call/Member/Index/MethodRef/Raw/Unary/Lambda…：类型未知或引用 → 保守视为可能
            _ => true,
        }
    }

    /// 类型可能持有 String 引用吗？
    fn type_maybe_string(t: &JType) -> bool {
        match t {
            JType::Bool | JType::Byte | JType::Short | JType::Int | JType::Long
            | JType::Char | JType::Float | JType::Double | JType::Void
            | JType::Array(_) => false, // 数组引用不是 String 本体（元素访问走 Index 节点）
            JType::Var => true,         // var / 推断 / 未知
            JType::Ref(n) => {
                let base = n.split('<').next().unwrap_or(n).trim();
                let last = base.rsplit('.').next().unwrap_or(base);
                matches!(
                    last,
                    "String" | "Object" | "CharSequence" | "Comparable" | "Serializable"
                ) || n.contains('<') // 泛型容器：剥壳后无法判定 → 保守
                // 单字母大写：类型参数 T/E/R（无界，运行期可为 String）
                || (last.len() <= 2 && last.chars().next().is_some_and(|c| c.is_uppercase()))
            }
        }
    }


    /// 设置当前方法的参数类型表（作为根作用域参与解析）。
    pub fn set_param_scope(&mut self, params: &[(String, JType)]) {
        self.param_scope = params.to_vec();
    }

    /// 名字是否为当前方法参数（虚拟执行的字段写守卫——参数写不得
    /// 误记为字段写）。
    pub fn is_param_name(&self, name: &str) -> bool {
        self.param_scope.iter().any(|(n, _)| n == name)
    }

    /// 名字键 → 名字反查（虚拟执行的字段写材料化）。
    pub fn name_of_key(&self, k: u32) -> Option<String> {
        Some(self.names.name(k).to_string())
    }
    pub fn clear_param_scope(&mut self) {
        self.param_scope.clear();
    }
}

// ---------------------------------------------------------------------------
// Lang 实现
// ---------------------------------------------------------------------------


// ---------------------------------------------------------------------------
// JLS 整数折叠辅助（从 cure-engine 迁入——数值语义归属语言侧）
// ---------------------------------------------------------------------------

/// i64 算术（long 语义）：除/余除数为 0 不折（ArithmeticException）。
/// 移位量按 JLS 掩码（long: & 63）。
fn fold_i64(op: BinOp, a: i64, b: i64) -> Option<i64> {
    use BinOp::*;
    Some(match op {
        Add => a.wrapping_add(b),
        Sub => a.wrapping_sub(b),
        Mul => a.wrapping_mul(b),
        Div => {
            if b == 0 {
                return None;
            }
            a.wrapping_div(b)
        }
        Rem => {
            if b == 0 {
                return None;
            }
            a.wrapping_rem(b)
        }
        Shl => a.wrapping_shl((b as u64 & 63) as u32),
        Shr => a.wrapping_shr((b as u64 & 63) as u32),
        UShr => ((a as u64).wrapping_shr((b as u64 & 63) as u32)) as i64,
        BitAnd => a & b,
        BitXor => a ^ b,
        BitOr => a | b,
        _ => return None,
    })
}

/// i32 算术（int 语义）：操作数按 i32 回绕；移位量按 JLS 掩码（& 31）。
fn fold_i32(op: BinOp, a: i64, b: i64) -> Option<i64> {
    use BinOp::*;
    let (x, y) = (a as i32, b as i32);
    Some(match op {
        Add => x.wrapping_add(y) as i64,
        Sub => x.wrapping_sub(y) as i64,
        Mul => x.wrapping_mul(y) as i64,
        Div => {
            if y == 0 {
                return None;
            }
            x.wrapping_div(y) as i64
        }
        Rem => {
            if y == 0 {
                return None;
            }
            x.wrapping_rem(y) as i64
        }
        Shl => x.wrapping_shl((y as u32) & 31) as i64,
        Shr => x.wrapping_shr((y as u32) & 31) as i64,
        UShr => (x as u32).wrapping_shr((y as u32) & 31) as i64,
        BitAnd => (x & y) as i64,
        BitXor => (x ^ y) as i64,
        BitOr => (x | y) as i64,
        _ => return None,
    })
}

/// 数值比较折叠（整数/字符提升为 int 后比较）。
fn cmp_i64(op: BinOp, x: i64, y: i64) -> Option<bool> {
    use BinOp::*;
    Some(match op {
        Lt => x < y,
        Le => x <= y,
        Gt => x > y,
        Ge => x >= y,
        Eq => x == y,
        Ne => x != y,
        _ => return None,
    })
}

impl Lang for JavaAst {
    type Id = JavaId;
    type NameKey = u32;

    fn kind(&self, id: JavaId) -> NodeKind {
        JavaAst::kind(self, id)
    }

    fn children(&self, id: JavaId) -> &[JavaId] {
        JavaAst::children(self, id)
    }

    fn set_child(&mut self, parent: JavaId, index: usize, new: JavaId) {
        self.nodes[parent.0 as usize].children.set(index, new);
    }
    fn remove_child(&mut self, parent: JavaId, index: usize) {
        self.nodes[parent.0 as usize].children.remove(index);
    }
    fn splice(&mut self, node: JavaId, index: usize, remove: usize, insert: Vec<JavaId>) {
        self.nodes[node.0 as usize].children.splice(index, remove, insert);
    }

    fn build_return(&mut self, value: Option<JavaId>) -> JavaId {
        self.ret(value)
    }
    fn build_block(&mut self, stmts: Vec<JavaId>) -> JavaId {
        self.block(stmts)
    }
    fn build_unary(&mut self, op: UnOp, operand: JavaId) -> JavaId {
        self.un(op, operand)
    }
    fn build_bool(&mut self, v: bool) -> JavaId {
        self.lit(Lit::Bool(v))
    }
    fn build_int(&mut self, v: i64, long: bool) -> JavaId {
        self.build_int_impl(v, long)
    }
    fn build_str(&mut self, s: &str) -> JavaId {
        self.lit(Lit::Str(s.to_string()))
    }
    fn build_char(&mut self, c: char) -> JavaId {
        self.lit(Lit::Char(c))
    }
    fn build_bin(&mut self, op: BinOp, l: JavaId, r: JavaId) -> JavaId {
        self.bin(op, l, r)
    }
    fn build_ternary(&mut self, c: JavaId, a: JavaId, b: JavaId) -> JavaId {
        self.ternary(c, a, b)
    }
    fn build_assign(&mut self, target: JavaId, value: JavaId) -> JavaId {
        self.assign(target, value)
    }

    // ---- 字面量折叠语义（JLS；引擎侧只做形态匹配）----

    fn fold_lit_bin(&mut self, op: BinOp, l: JavaId, r: JavaId) -> Option<JavaId> {
        let ll = self.literal(l)?;
        let rr = self.literal(r)?;
        // 字符串拼接（Str 与 Str/Char/Int/Long/Bool 字面量——拼接的隐式
        // valueOf 对这些基元是确定性的；浮点除外：Double.toString 算法与
        // Rust Display 不保证逐位一致）
        if op == BinOp::Add {
            let str_of = |x: LitRef<'_>| -> Option<String> {
                match x {
                    LitRef::Str(s) => Some(s.to_string()),
                    LitRef::Char(c) => Some(c.to_string()),
                    LitRef::Int(v) => Some(v.to_string()),
                    LitRef::Long(v) => Some(v.to_string()),
                    LitRef::Bool(b) => Some(b.to_string()),
                    _ => None,
                }
            };
            if matches!(ll, LitRef::Str(_)) || matches!(rr, LitRef::Str(_)) {
                if let (Some(a), Some(b)) = (str_of(ll), str_of(rr)) {
                    return Some(self.build_str(&format!("{a}{b}")));
                }
            }
        }
        // 整数折叠（JLS 环绕）：1+2→3、2*3→6、1<<3→8、& | ^ 等。
        // 除/余除数为 0 不折叠（ArithmeticException 语义）。
        let int_of = |x: LitRef<'_>| -> Option<i64> {
            match x {
                // Java 中 char 参与算术时提升为 int
                LitRef::Int(v) | LitRef::Long(v) => Some(v),
                LitRef::Char(c) => Some(c as u32 as i64),
                _ => None,
            }
        };
        let (a, b) = (int_of(ll)?, int_of(rr)?);
        let wide = matches!(ll, LitRef::Long(_)) || matches!(rr, LitRef::Long(_));
        let v = if wide { fold_i64(op, a, b) } else { fold_i32(op, a, b) }?;
        Some(self.build_int(v, wide))
    }

    fn fold_lit_cmp(&mut self, op: BinOp, l: JavaId, r: JavaId) -> Option<bool> {
        let ll = self.literal(l)?;
        let rr = self.literal(r)?;
        // 整数/字符按数值比较（char 与 int 比较同样提升为 int）
        let int_of = |x: LitRef<'_>| -> Option<i64> {
            match x {
                LitRef::Int(v) | LitRef::Long(v) => Some(v),
                LitRef::Char(c) => Some(c as u32 as i64),
                _ => None,
            }
        };
        if let (Some(x), Some(y)) = (int_of(ll), int_of(rr)) {
            return cmp_i64(op, x, y);
        }
        // 布尔相等（装箱引用 == 语义上不可折，但布尔字面量场景下
        // true/false 是规范实例，值比较与引用比较一致）
        if let (LitRef::Bool(x), LitRef::Bool(y)) = (ll, rr) {
            if matches!(op, BinOp::Eq | BinOp::Ne) {
                return Some(if op == BinOp::Eq { x == y } else { x != y });
            }
        }
        // 字符串（引用比较）与浮点（NaN）不折
        None
    }

    fn fold_lit_neg(&mut self, v: i64, wide: bool) -> Option<i64> {
        // `-(-2147483648)`：i32 域内无 +2147483648，拒绝折叠
        let n = -v;
        if !wide && n > i32::MAX as i64 {
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
        wide: bool,
    ) -> Option<ReassocOutcome<JavaId>> {
        use BinOp::*;
        let raw: i64 = match (op1, op2) {
            (BitXor, BitXor) => k1 ^ k2,
            (a, b) if matches!(a, Add | Sub) && matches!(b, Add | Sub) => {
                let sign = |o: BinOp| if o == Sub { -1i64 } else { 1i64 };
                sign(a) * k1 + sign(b) * k2
            }
            _ => return None,
        };
        // JLS 宽度回绕：i32 合并结果溢出必须回绕——不回绕会产出
        // 非法字面量（`x + 2147483648` 无法作为 int 编译）
        let d = if wide { raw } else { raw as i32 as i64 };
        if d == 0 {
            return Some(ReassocOutcome::Neutral);
        }
        if op1 == BitXor && op2 == BitXor {
            return Some(ReassocOutcome::Lit(self.build_int(d, wide), BitXor));
        }
        if d > 0 {
            Some(ReassocOutcome::Lit(self.build_int(d, wide), Add))
        } else {
            // 负 delta 用 `x - |d|`；|d| 回绕后仍为负（仅 ±2^31 边界）
            // 或宽整数 -2^63 取反溢出时，退回 `x + d`（负字面量）——
            // 二者模 2^宽度 同余，语法均合法
            let nd = if wide { d.checked_neg().unwrap_or(d) } else { (d as i32).wrapping_neg() as i64 };
            if nd > 0 {
                Some(ReassocOutcome::Lit(self.build_int(nd, wide), Sub))
            } else {
                Some(ReassocOutcome::Lit(self.build_int(d, wide), Add))
            }
        }
    }

    fn copy_subtree(&mut self, id: JavaId) -> JavaId {
        let old_children: Vec<JavaId> = self.nodes[id.0 as usize].children.as_slice().to_vec();
        let mut children = Vec::with_capacity(old_children.len());
        for c in old_children {
            children.push(self.copy_subtree(c));
        }
        let data = self.nodes[id.0 as usize].data.clone();
        self.push(data, children)
    }

    fn effect(&self, id: JavaId) -> Effect {
        if let Some(e) = self.effect_cache.get(id.0 as usize).and_then(|x| *x) {
            return e;
        }
        // 缓存未命中（结构不变量被破坏时）退化为按需递归
        let mut e = self.own_effect(id);
        for &c in self.nodes[id.0 as usize].children.as_slice() {
            e = e.worst(self.effect(c));
        }
        e
    }

    fn invalidate_effect(&mut self, id: JavaId) {
        if let Some(slot) = self.effect_cache.get_mut(id.0 as usize) {
            *slot = None;
        }
        // 事件索引同步失效：本节点子树若被改，其预计算事件已陈旧
        // （B3 脏标记 + 清空该槽——见 cure-tree EventStore）
        self.events.invalidate(id.0 as usize);
    }

    fn debug_verify_events(&self, stmt: JavaId) {
        if !verify_events_on() {
            return;
        }
        if let Some(indexed) = self.region_events(stmt) {
            let fresh = cure_tree::collect_region_events(self, stmt);
            // 比较（键+种类+节点）三元组序列
            let key = |e: &cure_engine::kind::RegionEvent<JavaId, u32>| {
                (e.key, matches!(e.kind, cure_engine::kind::EventKind::Use), e.node.0)
            };
            let a: Vec<_> = indexed.iter().map(key).collect();
            let b: Vec<_> = fresh.iter().map(key).collect();
            if a != b {
                eprintln!(
                    "[EVENTS-STALE] stmt #{} kind={:?}: indexed {} events, fresh {} events",
                    stmt.0, self.kind(stmt), a.len(), b.len()
                );
            }
        }
    }

    fn region_events(
        &self,
        id: JavaId,
    ) -> Option<&[cure_engine::kind::RegionEvent<JavaId, u32>]> {
        // 空序列 = 未索引（非语句节点或已失效）→ None 走原递归
        let v = self.events.slot(id.0 as usize);
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    }

    fn var_key(&self, id: JavaId) -> Option<u32> {
        self.names.var_key(id.0 as usize)
    }

    fn node_index(&self, id: JavaId) -> usize {
        id.0 as usize
    }
    fn id_of_index(&self, idx: usize) -> JavaId {
        JavaId(idx as u32)
    }

    fn is_opaque(&self, id: JavaId) -> bool {
        match self.data(id) {
            // 不可解析原文
            NodeData::Raw { .. } => true,
            // 匿名类体是原文（捕获的外部局部变量不可见）——
            // 对抗波 4：仅被匿名类捕获的变量曾被零用途规则误删
            NodeData::New { anon_raw: Some(_), .. } => true,
            _ => false,
        }
    }

    fn own_effect(&self, id: JavaId) -> Effect {
        match self.data(id) {
            NodeData::Literal(_)
            | NodeData::This
            | NodeData::Super
            | NodeData::Paren
            | NodeData::Block
            | NodeData::Group
            | NodeData::Empty
            | NodeData::VarDecl { .. }
            | NodeData::Break { .. }
            | NodeData::Continue { .. }
            | NodeData::Try
            | NodeData::Catch { .. }
            | NodeData::Switch
            | NodeData::Case { .. }
            | NodeData::Label { .. }
            | NodeData::ExprStmt
            | NodeData::Ternary
            | NodeData::ArrayLit
            | NodeData::Lambda { .. }
            | NodeData::MethodRef { .. }
            | NodeData::InstanceOf { .. } => Effect::Pure,
            NodeData::VarRef { .. } => Effect::MayRead,
            NodeData::Member { .. } => Effect::MayThrow,
            NodeData::Index => Effect::MayThrow,
            NodeData::New { .. } | NodeData::NewArray { .. } => Effect::MayThrow,
            NodeData::Call => Effect::Unknown,
            NodeData::Assign { .. } => Effect::MayWrite,
            NodeData::Unary { op } => {
                if op.is_incdec() {
                    Effect::MayWrite
                } else {
                    Effect::Pure
                }
            }
            NodeData::Binary { op } => {
                if op.may_div_zero() {
                    Effect::MayThrow
                } else {
                    Effect::Pure
                }
            }
            NodeData::Cast { ty } => {
                if ty.is_ref() {
                    // 引用转型可能抛 ClassCastException
                    Effect::MayThrow
                } else {
                    Effect::Pure
                }
            }
            NodeData::For { .. } | NodeData::ForEach { .. } | NodeData::If => Effect::MayRead,
            NodeData::While | NodeData::DoWhile => Effect::MayRead,
            NodeData::Return => Effect::MayThrow,
            NodeData::Throw => Effect::MayThrow,
            NodeData::Assert => Effect::MayThrow,
            NodeData::Synchronized => Effect::MayWrite,
            NodeData::Raw { .. } => Effect::Unknown,
        }
    }

    fn is_bool(&self, id: JavaId) -> bool {
        match self.data(id) {
            NodeData::Literal(l) => matches!(l, Lit::Bool(_)),
            NodeData::Unary { op } => *op == UnOp::Not,
            NodeData::Binary { op } => {
                op.is_comparison() || matches!(op, BinOp::And | BinOp::Or)
            }
            NodeData::Ternary => {
                let ch = self.children(id);
                self.is_bool(ch[1]) && self.is_bool(ch[2])
            }
            NodeData::Paren => self.is_bool(self.children(id)[0]),
            NodeData::InstanceOf { .. } => true,
            NodeData::VarRef { .. } => self
                .var_types
                .get(id.0 as usize)
                .and_then(|t| t.as_ref())
                .is_some_and(|t| t.is_bool()),
            _ => false,
        }
    }

    fn is_exact_int(&self, id: JavaId) -> bool {
        match self.data(id) {
            NodeData::Literal(l) => matches!(
                l,
                Lit::Int(_) | Lit::Long(_) | Lit::Char(_) | Lit::NumRaw { val: NumVal::Int(_) | NumVal::Long(_), .. }
            ),
            NodeData::VarRef { .. } => {
                self.var_types.get(id.0 as usize).and_then(|t| t.as_ref()).is_some_and(|t| t.is_integral())
            }
            NodeData::Binary { op } => {
                let ch = self.children(id);
                (op.is_arith() || op.is_comparison())
                    && self.is_exact_int(ch[0])
                    && self.is_exact_int(ch[1])
            }
            NodeData::Cast { ty } => ty.is_integral(),
            NodeData::Paren => self.is_exact_int(self.children(id)[0]),
            NodeData::Ternary => {
                let ch = self.children(id);
                self.is_exact_int(ch[1]) && self.is_exact_int(ch[2])
            }
            _ => false,
        }
    }

    fn is_local_var(&self, id: JavaId) -> bool {
        // VarRef 必须解析到局部变量/参数（var_types 作用域解析）才可
        // 被传播/删除规则触碰——字段引用（this.x / 静态字段裸名）有
        // 跨方法可见性。未解析（Raw 区域/坏代码）同样返回 false（保守）。
        // 差分审查抓获：dead_store 曾据此把 `first = cl;`（静态字段写）
        // 当局部死存储消除，双向链表表头更新静默丢失。
        matches!(self.data(id), NodeData::VarRef { .. }) && self.var_type(id).is_some()
    }

    fn is_inferred_decl(&self, decl: JavaId) -> bool {
        matches!(self.data(decl), NodeData::VarDecl { ty: JType::Var, .. })
    }

    fn is_char_decl(&self, decl: JavaId) -> bool {
        matches!(self.data(decl), NodeData::VarDecl { ty: JType::Char, .. })
    }

    fn is_wide_decl(&self, decl: JavaId) -> bool {
        // 窄类型（byte/short/char）复合赋值含隐式收窄：delta 折回声明
        // 会产出超域非法常量或丢静态类型（char 99 → println 打数字）
        !matches!(
            self.data(decl),
            NodeData::VarDecl { ty: JType::Byte | JType::Short | JType::Char, .. }
        )
    }

    fn bin_op(&self, id: JavaId) -> Option<BinOp> {
        match self.data(id) {
            NodeData::Binary { op } => Some(*op),
            _ => None,
        }
    }
    fn un_op(&self, id: JavaId) -> Option<UnOp> {
        match self.data(id) {
            NodeData::Unary { op } => Some(*op),
            _ => None,
        }
    }
    fn literal(&self, id: JavaId) -> Option<LitRef<'_>> {
        match self.data(id) {
            NodeData::Literal(l) => Some(match l {
                Lit::Bool(b) => LitRef::Bool(*b),
                Lit::Int(v) => LitRef::Int(*v),
                Lit::Long(v) => LitRef::Long(*v),
                Lit::Float(v) => LitRef::Float(*v),
                Lit::Double(v) => LitRef::Double(*v),
                Lit::Char(c) => LitRef::Char(*c),
                Lit::Str(s) | Lit::TextBlock(s) => LitRef::Str(s),
                Lit::NumRaw { val, .. } => match val {
                    NumVal::Int(v) => LitRef::Int(*v),
                    NumVal::Long(v) => LitRef::Long(*v),
                    NumVal::Float(v) => LitRef::Float(*v),
                    NumVal::Double(v) => LitRef::Double(*v),
                },
                Lit::Null => LitRef::Null,
            }),
            _ => None,
        }
    }
    fn var_name(&self, id: JavaId) -> Option<&str> {
        match self.data(id) {
            NodeData::VarRef { name } => Some(self.sn(*name)),
            NodeData::VarDecl { name, .. } => Some(name),
            NodeData::ForEach { name, .. } | NodeData::Catch { name, .. } => Some(name),
            _ => None,
        }
    }
    fn member_name(&self, id: JavaId) -> Option<&str> {
        match self.data(id) {
            NodeData::Member { name } => Some(self.sn(*name)),
            NodeData::MethodRef { name } => Some(self.sn(*name)),
            _ => None,
        }
    }
    fn assign_op(&self, id: JavaId) -> Option<BinOp> {
        match self.data(id) {
            NodeData::Assign { op } => *op,
            _ => None,
        }
    }

    /// 重建缓存：
    /// 1) 效果表——arena 按升序扫描（child index < parent index 不变量）；
    /// 2) 变量类型表——从 root 做作用域栈遍历。
    fn prepare(&mut self, root: JavaId) {

        if cnt_prepare_on() {
            PREPARE_CALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            PREPARE_NODES.fetch_add(self.nodes.len(), std::sync::atomic::Ordering::Relaxed);
        }
        // 名字键增量扩展（cure-tree NameTable）：新节点 intern（既有节点
        // 键不变——名字与位置无关）。名字先收集（&self 借用安全形态），
        // 再批量扩展（&mut self.names）。
        let n_nodes = self.nodes.len();
        let key_slots = self.names.key_slots();
        if key_slots < n_nodes {
            let names: Vec<Option<String>> = (key_slots..n_nodes)
                .map(|i| self.var_name(JavaId(i as u32)).map(|n| n.to_string()))
                .collect();
            self.names.extend_owned(key_slots, &names);
        }
        // 守卫缓存失效：树已变
        self.string_identity.set(None);
        // 区域事件索引【惰性重建】（B3，cure-tree EventStore）：只重建
        // **空条目**——被 invalidate_effect 失效的语句、或新语句。未失效
        // 条目的事件是子树局部的（与位置无关）。关键不变量：**一切**改树
        // 路径都必须失效受影响语句的条目：
        //   - 立即 Replace：目标祖先链 + with 子树中既有节点的**旧容器**
        //     祖先链（搬移——pass.rs 立即路径两处都做）
        //   - 队列编辑应用：同上（pass.rs deferred 应用后统一失效）
        // 违反不变量的后果：陈旧事件驱动错误决策（B2 教训：cff_diamond
        // 差分当场抓获）。性能背景：Types.java 上 prepare 每 pass 全量重建
        // = 726 倍冗余（37 万文件语料 prepare 占 57% 线程时间）。
        self.events.resize(n_nodes);
        if self.events.is_clean() {
            if self.prepared_root == Some(root) {
                return;
            }
            // 树全局缓存（事件/效果表）仍有效；根变了 → 只重建
            // root 相关的作用域分析段
            self.rebuild_var_types(root);
            self.prepared_root = Some(root);
            return;
        }
        self.prepared_root = Some(root);
        self.events.mark_clean();
        for sr in cure_tree::stmt_roots(self, root) {
            let idx = sr.0 as usize;
            if self.events.slot(idx).is_empty() {
                let events = cure_tree::collect_region_events(self, sr);
                self.events.put(idx, events);
            }
        }
        // ---- 效果表（cure-tree rebuild_effects：不动点 sweep——
        // children-先于-parent 索引序 + 注入节点违例多轮收敛）----
        let effects = cure_tree::rebuild_effects(self, n_nodes);
        self.effect_cache = effects;
        self.rebuild_var_types(root);
    }
}

impl JavaAst {
    /// root 相关的作用域分析（从 prepare 提取——根变化时单独重建）。
    pub(crate) fn rebuild_var_types(&mut self, root: JavaId) {
        // ---- 变量类型（作用域栈）----
        self.var_types.clear();
        self.var_types.resize(self.nodes.len(), None);
        #[derive(Clone)]
        enum Frame {
            Enter(JavaId),
            Exit,
            PopScope,
            Bind(String, JType),
        }
        let mut base: HashMap<String, JType> = HashMap::new();
        for (n, t) in &self.param_scope {
            base.insert(n.clone(), t.clone());
        }
        let mut scopes: Vec<HashMap<String, JType>> = vec![base];
        let mut stack = vec![Frame::Enter(root)];
        while let Some(f) = stack.pop() {
            match f {
                Frame::Enter(id) => {
                    let node = &self.nodes[id.0 as usize];
                    enum Action {
                        None,
                        Scope,
                        Bind(String, JType),
                    }
                    let action = match &node.data {
                        NodeData::Block | NodeData::For { .. } | NodeData::Try => Action::Scope,
                        NodeData::VarDecl { name, ty } => {
                            scopes.last_mut().unwrap().insert(name.clone(), ty.clone());
                            Action::None
                        }
                        NodeData::VarRef { name } => {
                            if let Some(t) = scopes
                                .iter()
                                .rev()
                                .find_map(|s| s.get(self.sn(*name)))
                                .cloned()
                            {
                                self.var_types[id.0 as usize] = Some(t);
                            }
                            Action::None
                        }
                        NodeData::Catch { name, .. } => {
                            Action::Bind(name.clone(), JType::Var)
                        }
                        NodeData::ForEach { name, ty } => {
                            Action::Bind(name.clone(), ty.clone())
                        }
                        _ => Action::None,
                    };
                    // 期望弹出顺序：[Bind?] children… [PopScope?] Exit
                    // ⇒ 压栈顺序取反：Exit、[PopScope]、children(逆序)、[Bind]
                    stack.push(Frame::Exit);
                    if matches!(action, Action::Scope | Action::Bind(_, _)) {
                        scopes.push(HashMap::new());
                        stack.push(Frame::PopScope);
                    }
                    for &c in node.children.as_slice().iter().rev() {
                        stack.push(Frame::Enter(c));
                    }
                    if let Action::Bind(n, t) = action {
                        stack.push(Frame::Bind(n, t));
                    }
                }
                Frame::Bind(name, ty) => {
                    scopes.last_mut().unwrap().insert(name, ty);
                }
                Frame::PopScope => {
                    scopes.pop();
                }
                Frame::Exit => {}
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_smoke() {
        let mut a = JavaAst::new();
        let one = a.lit(Lit::Int(1));
        let x = a.var_decl("x", JType::Int, Some(one));
        let xv = a.var("x");
        let r = a.ret(Some(xv));
        let body = a.block(vec![x, r]);
        assert_eq!(a.kind(body), NodeKind::Block);
        assert_eq!(a.children(body).len(), 2);
    }

    #[test]
    fn effect_aggregation() {
        let mut a = JavaAst::new();
        let f = a.plain_call("foo", vec![]);
        let r = a.ret(Some(f));
        let body = a.block(vec![r]);
        a.prepare(body);
        assert_eq!(a.effect(f), Effect::Unknown);
        assert_eq!(a.effect(r), Effect::Unknown);
        assert_eq!(a.effect(body), Effect::Unknown);

        let mut b = JavaAst::new();
        let two = b.lit(Lit::Int(2));
        let three = b.lit(Lit::Int(3));
        let add = b.bin(BinOp::Add, two, three);
        let r2 = b.ret(Some(add));
        let body2 = b.block(vec![r2]);
        b.prepare(body2);
        assert_eq!(b.effect(add), Effect::Pure);
        // return 语句本身按控制流计为 MayThrow，Block 聚合随之
        assert_eq!(b.effect(r2), Effect::MayThrow);
        assert_eq!(b.effect(body2), Effect::MayThrow);
    }

    #[test]
    fn var_type_resolution_and_shadowing() {
        let mut a = JavaAst::new();
        // { boolean b = true; { int b2 = 1; b2; } b; }
        let one = a.lit(Lit::Int(1));
        let b2_decl = a.var_decl("b2", JType::Int, Some(one));
        let inner_b2 = a.var("b2");
        let inner_stmt = a.expr_stmt(inner_b2);
        let inner = a.block(vec![b2_decl, inner_stmt]);
        let tv = a.lit(Lit::Bool(true));
        let decl_b = a.var_decl("b", JType::Bool, Some(tv));
        let outer_b = a.var("b");
        let outer_stmt = a.expr_stmt(outer_b);
        let body = a.block(vec![decl_b, inner, outer_stmt]);
        a.prepare(body);
        assert!(a.is_bool(outer_b));
        // b2 的使用在 inner 块内，类型可解析
        assert!(a.var_type(inner_b2).is_some());
    }

    #[test]
    fn raw_and_foreach_semantics() {
        let mut a = JavaAst::new();
        // { int x = 1; RAW; for (String s : list) { s.len(); } }
        let one = a.lit(Lit::Int(1));
        let dx = a.var_decl("x", JType::Int, Some(one));
        let r = a.raw("broken ~!@ code");
        let sv = a.var("s");
        let call = a.method_call(sv, "len", vec![]);
        let es = a.expr_stmt(call);
        let body = a.block(vec![es]);
        let lst = a.var("list");
        let fe = a.for_each("s", JType::Ref("String".into()), lst, body);
        let all = a.block(vec![dx, r, fe]);
        a.prepare(all);
        assert_eq!(a.kind(r), NodeKind::Raw);
        assert_eq!(a.effect(r), Effect::Unknown);
        assert!(a.var_type(sv).is_some()); // 循环变量类型可解析
    }

    #[test]
    fn num_raw_literal() {
        let mut a = JavaAst::new();
        let l = a.lit(Lit::NumRaw {
            text: "0x1F".into(),
            val: NumVal::Int(31),
        });
        a.prepare(l);
        assert!(a.is_exact_int(l));
        assert!(matches!(a.literal(l), Some(LitRef::Int(31))));
    }
}

// ---------------------------------------------------------------------------
// 区域事件索引（使用索引）：规则区域扫描的预计算。
// 遍历序必须与引擎 scan_region 原递归严格一致：
//   Assign      → [Write(目标)] + value 子树（目标本身不再当 Use 扫）
//   自增自减    → [Write(目标)]，不扫子
//   VarDecl     → [Write, Shadow] + 全部孩子
//   ForEach     → [Write, Shadow] + 全部孩子；Catch → [Shadow] + 孩子
//   VarRef      → [Use]
//   其余        → 依序拼接孩子
// ---------------------------------------------------------------------------

impl JavaAst {
    /// 名字 → 实化键（持久表，只增；同名字恒同键）。
    /// intern 名字 → Sym（复用 name_intern 表 + sym_names 向量）。
    pub fn intern_sym(&mut self, name: &str) -> Sym {
        Sym(self.names.intern(name))
    }

    /// Sym → 名字原文。
    pub fn sn(&self, sym: Sym) -> &str {
        self.names.name(sym.0)
    }




}


// ---------------------------------------------------------------------------
// 结构化指标：编译单元级聚合（供 --stats 等评估简化效果——节点数 /
// 判定点 / 嵌套深度不受格式化影响，只被语义简化改变）
// ---------------------------------------------------------------------------

/// 编译单元全部可执行体（方法/构造器/初始化块，含嵌套类型）的
/// 结构指标总和。
pub fn unit_metrics(ast: &JavaAst, unit: &CompilationUnit) -> TreeMetrics {
    let mut total = TreeMetrics::default();
    for ty in &unit.types {
        metrics_of_type(ast, ty, &mut total);
    }
    total
}

fn metrics_of_type(ast: &JavaAst, ty: &TypeDecl, total: &mut TreeMetrics) {
    for m in &ty.members {
        match m {
            Member::Method { body: Some(b), .. } | Member::Constructor { body: Some(b), .. } => {
                total.add(subtree_metrics(ast, *b));
            }
            Member::Initializer { body, .. } => {
                total.add(subtree_metrics(ast, *body));
            }
            Member::Type(t) => metrics_of_type(ast, t, total),
            _ => {}
        }
    }
}

static ENV_CACHED: std::sync::OnceLock<(bool, bool)> = std::sync::OnceLock::new();
fn env_flags() -> (bool, bool) {
    *ENV_CACHED.get_or_init(|| {
        (
            std::env::var("CURE_CNT_PREPARE").is_ok(),
            std::env::var("CURE_VERIFY_EVENTS").is_ok(),
        )
    })
}
fn cnt_prepare_on() -> bool { env_flags().0 }
fn verify_events_on() -> bool { env_flags().1 }

pub static PREPARE_CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static PREPARE_NODES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub static COLLECT_CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
