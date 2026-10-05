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

use std::collections::HashMap;

use cure_engine::Lang;

// 本 crate 自用 + 供下游 crate（如 cure-java-print/parser）免依赖引擎直接使用
pub use cure_engine::kind::{BinOp, LitRef, NodeKind, UnOp};
pub use cure_engine::Effect;

// ---------------------------------------------------------------------------
// 基础类型
// ---------------------------------------------------------------------------

/// arena 句柄。
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct JavaId(pub u32);

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
    Member { name: String },
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
    VarRef { name: String },
    Literal(Lit),
    This,
    /// `super`（super.foo() / super(args) 的 callee）
    Super,
    /// children: `[expr]`；`x instanceof T bind`
    InstanceOf { ty: JType, bind: Option<String> },
    /// children: `[body]`（body 为 Block 或表达式）
    Lambda { params_raw: String },
    /// children: `[receiver]`；`recv::name`（name 含 `new`）
    MethodRef { name: String },
}

#[derive(Clone, PartialEq, Debug)]
pub struct Node {
    pub data: NodeData,
    pub children: Vec<JavaId>,
}

/// Java AST arena。只追加不回收，[`JavaId`] 永不失效。
#[derive(Default, Debug)]
pub struct JavaAst {
    pub(crate) nodes: Vec<Node>,
    /// prepare() 重建：JavaId → 聚合效果。
    effect_cache: HashMap<u32, Effect>,
    /// prepare() 重建：VarRef 节点 → 声明类型（作用域解析）。
    var_types: HashMap<u32, JType>,
    /// 方法参数类型（由 simplify 门面按方法设置，作为根作用域）。
    param_scope: Vec<(String, JType)>,
    /// 类级常量字段（static final 且字面量/字面量数组初始化、无写、无同名局部）
    /// → 初始化节点。由 simplify_unit 填充（跨方法解密的字符串表）。
    pub const_fields: HashMap<String, JavaId>,
    /// 可内联的单 return 方法：名字 → (参数名表, 返回表达式节点)。
    /// 由 simplify_unit 填充（解密 helper：d(0) → 方法体）。
    pub inline_methods: HashMap<String, (Vec<String>, JavaId)>,
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
        self.nodes.push(Node { data, children });
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
            NodeData::Block => NodeKind::Block,
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
    pub fn children(&self, id: JavaId) -> &[JavaId] {
        &self.nodes[id.0 as usize].children
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
        self.push(NodeData::Member { name: name.into() }, vec![obj])
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
    pub fn array_lit(&mut self, elems: Vec<JavaId>) -> JavaId {
        self.push(NodeData::ArrayLit, elems)
    }
    pub fn var(&mut self, name: &str) -> JavaId {
        self.push(NodeData::VarRef { name: name.into() }, vec![])
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
        self.push(
            NodeData::MethodRef { name: name.into() },
            vec![receiver],
        )
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
        self.var_types.get(&id.0)
    }

    /// 设置当前方法的参数类型表（作为根作用域参与解析）。
    pub fn set_param_scope(&mut self, params: &[(String, JType)]) {
        self.param_scope = params.to_vec();
    }
    pub fn clear_param_scope(&mut self) {
        self.param_scope.clear();
    }
}

// ---------------------------------------------------------------------------
// Lang 实现
// ---------------------------------------------------------------------------

impl Lang for JavaAst {
    type Id = JavaId;

    fn kind(&self, id: JavaId) -> NodeKind {
        JavaAst::kind(self, id)
    }

    fn children(&self, id: JavaId) -> &[JavaId] {
        JavaAst::children(self, id)
    }

    fn set_child(&mut self, parent: JavaId, index: usize, new: JavaId) {
        self.nodes[parent.0 as usize].children[index] = new;
    }
    fn remove_child(&mut self, parent: JavaId, index: usize) {
        self.nodes[parent.0 as usize].children.remove(index);
    }
    fn splice(&mut self, node: JavaId, index: usize, remove: usize, insert: Vec<JavaId>) {
        let ch = &mut self.nodes[node.0 as usize].children;
        let end = (index + remove).min(ch.len());
        ch.splice(index..end, insert);
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
    fn copy_subtree(&mut self, id: JavaId) -> JavaId {
        let old_children = self.nodes[id.0 as usize].children.clone();
        let mut children = Vec::with_capacity(old_children.len());
        for c in old_children {
            children.push(self.copy_subtree(c));
        }
        let data = self.nodes[id.0 as usize].data.clone();
        self.push(data, children)
    }

    fn effect(&self, id: JavaId) -> Effect {
        if let Some(&e) = self.effect_cache.get(&id.0) {
            return e;
        }
        // 缓存未命中（结构不变量被破坏时）退化为按需递归
        let mut e = self.own_effect(id);
        for &c in &self.nodes[id.0 as usize].children {
            e = e.worst(self.effect(c));
        }
        e
    }

    fn own_effect(&self, id: JavaId) -> Effect {
        match self.data(id) {
            NodeData::Literal(_)
            | NodeData::This
            | NodeData::Super
            | NodeData::Paren
            | NodeData::Block
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
            NodeData::VarRef { .. } => self.var_types.get(&id.0).is_some_and(|t| t.is_bool()),
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
                self.var_types.get(&id.0).is_some_and(|t| t.is_integral())
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
        matches!(self.data(id), NodeData::VarRef { .. })
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
            NodeData::VarRef { name } | NodeData::VarDecl { name, .. } => Some(name),
            NodeData::ForEach { name, .. } | NodeData::Catch { name, .. } => Some(name),
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
        // ---- 效果表 ----
        // 正常情况一轮升序扫描即可（children index < parent index 的
        // 解析器不变量）。但 Edit::Replace 注入的新节点 append 在 arena
        // 末尾、index 大于其（旧）父节点——单轮 sweep 会让父聚合到 Unknown
        // 并污染祖先链。故迭代到不动点（违例深度有限，2 轮内收敛）。
        self.effect_cache.clear();
        let n = self.nodes.len() as u32;
        for _round in 0..4 {
            let mut changed = false;
            for i in 0..n {
                let id = JavaId(i);
                let mut e = self.own_effect(id);
                for &c in &self.nodes[i as usize].children {
                    e = e.worst(self.effect_cache.get(&c.0).copied().unwrap_or(Effect::Unknown));
                }
                if self.effect_cache.insert(i, e) != Some(e) {
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        // ---- 变量类型（作用域栈）----
        self.var_types.clear();
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
                            if let Some(t) =
                                scopes.iter().rev().find_map(|s| s.get(name)).cloned()
                            {
                                self.var_types.insert(id.0, t);
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
                    for &c in node.children.iter().rev() {
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
