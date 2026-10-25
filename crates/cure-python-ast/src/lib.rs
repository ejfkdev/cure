//! # cure-python-ast
//!
//! Python arena AST（`PyId`）与语义辅助。设计目标：
//! - 值级语法完整建模（规则可改写）；复杂/未建模语法以 `Raw` 原文保真；
//! - 语句 metadata（参数、装饰器、类型注解、except 头）保存原文，打印层
//!   不做类型解析；
//! - 不依赖 Python 运行时，所有字面量保留原文，折叠产物用规范形式重建。

use std::collections::HashMap;

pub const MAX_NEST: usize = 512;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct PyId(pub u32);

impl PyId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PyBin {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    MatMul,
    LShift,
    RShift,
    BitOr,
    BitXor,
    BitAnd,
}

impl PyBin {
    pub fn symbol(self) -> &'static str {
        match self {
            PyBin::Add => "+",
            PyBin::Sub => "-",
            PyBin::Mul => "*",
            PyBin::Div => "/",
            PyBin::FloorDiv => "//",
            PyBin::Mod => "%",
            PyBin::Pow => "**",
            PyBin::MatMul => "@",
            PyBin::LShift => "<<",
            PyBin::RShift => ">>",
            PyBin::BitOr => "|",
            PyBin::BitXor => "^",
            PyBin::BitAnd => "&",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PyUn {
    Plus,
    Minus,
    Invert,
    Not,
}

impl PyUn {
    pub fn symbol(self) -> &'static str {
        match self {
            PyUn::Plus => "+",
            PyUn::Minus => "-",
            PyUn::Invert => "~",
            PyUn::Not => "not ",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PyBoolOp {
    And,
    Or,
}

impl PyBoolOp {
    pub fn symbol(self) -> &'static str {
        match self {
            PyBoolOp::And => "and",
            PyBoolOp::Or => "or",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PyCmp {
    Eq,
    NotEq,
    /// Python 2 `<>`（与 `!=` 同义，打印时保留原写法）
    LtGt,
    Lt,
    LtE,
    Gt,
    GtE,
    Is,
    IsNot,
    In,
    NotIn,
}

impl PyCmp {
    pub fn symbol(self) -> &'static str {
        match self {
            PyCmp::Eq => "==",
            PyCmp::NotEq => "!=",
            PyCmp::LtGt => "<>",
            PyCmp::Lt => "<",
            PyCmp::LtE => "<=",
            PyCmp::Gt => ">",
            PyCmp::GtE => ">=",
            PyCmp::Is => "is",
            PyCmp::IsNot => "is not",
            PyCmp::In => "in",
            PyCmp::NotIn => "not in",
        }
    }

    pub fn scalar(self) -> Option<crate::fold::ScalarCmp> {
        use crate::fold::ScalarCmp;
        match self {
            PyCmp::Eq => Some(ScalarCmp::Eq),
            PyCmp::NotEq | PyCmp::LtGt => Some(ScalarCmp::Ne),
            PyCmp::Lt => Some(ScalarCmp::Lt),
            PyCmp::LtE => Some(ScalarCmp::Le),
            PyCmp::Gt => Some(ScalarCmp::Gt),
            PyCmp::GtE => Some(ScalarCmp::Ge),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LitKind {
    Int,
    Float,
    Complex,
    Str,
    Bytes,
    Bool,
    None,
    Ellipsis,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Literal {
    /// 源原文（打印优先使用；折叠产物为规范形式）。
    pub raw: String,
    pub kind: LitKind,
    /// 可安全折叠的整数（不在 i64 内则为 None）。
    pub int: Option<i64>,
    pub bool_val: Option<bool>,
}

impl Literal {
    pub fn int(v: i64) -> Self {
        Self {
            raw: v.to_string(),
            kind: LitKind::Int,
            int: Some(v),
            bool_val: None,
        }
    }
    /// 源原文 + 可折叠整数（往返打印保留 1_000/0x1f 等写法）。
    pub fn int_raw(raw: impl Into<String>, v: i64) -> Self {
        Self {
            raw: raw.into(),
            kind: LitKind::Int,
            int: Some(v),
            bool_val: None,
        }
    }
    pub fn float(v: f64) -> Self {
        Self {
            raw: format_py_float(v),
            kind: LitKind::Float,
            int: None,
            bool_val: None,
        }
    }
    pub fn boolean(v: bool) -> Self {
        Self {
            raw: if v { "True" } else { "False" }.into(),
            kind: LitKind::Bool,
            int: None,
            bool_val: Some(v),
        }
    }
    pub fn none() -> Self {
        Self {
            raw: "None".into(),
            kind: LitKind::None,
            int: None,
            bool_val: None,
        }
    }
    pub fn ellipsis() -> Self {
        Self {
            raw: "...".into(),
            kind: LitKind::Ellipsis,
            int: None,
            bool_val: None,
        }
    }
    pub fn raw_str(raw: impl Into<String>, kind: LitKind) -> Self {
        Self {
            raw: raw.into(),
            kind,
            int: None,
            bool_val: None,
        }
    }
}

/// Python `repr(float)` 常用形态（`inf`/`nan` 单独处理；指数补零差异
/// 在 AST 等价比较中按 CPython 的 repr 规范化——打印层保持一致）。
pub fn format_py_float(v: f64) -> String {
    if v.is_nan() {
        return "float('nan')".into();
    }
    if v.is_infinite() {
        return if v > 0.0 {
            "float('inf')".into()
        } else {
            "float('-inf')".into()
        };
    }
    let s = format!("{v:?}");
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{s}.0")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArgKind {
    Positional,
    Star,
    /// 关键字参数名。
    Keyword,
    KwStar,
}

#[derive(Clone, PartialEq, Debug)]
pub struct CallArg {
    pub kind: ArgKind,
    pub name: Option<String>,
}

impl CallArg {
    pub fn pos() -> Self {
        Self {
            kind: ArgKind::Positional,
            name: None,
        }
    }
    pub fn star() -> Self {
        Self {
            kind: ArgKind::Star,
            name: None,
        }
    }
    pub fn kw(name: impl Into<String>) -> Self {
        Self {
            kind: ArgKind::Keyword,
            name: Some(name.into()),
        }
    }
    pub fn kwstar() -> Self {
        Self {
            kind: ArgKind::KwStar,
            name: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CompKind {
    List,
    Set,
    Dict,
    Gen,
}

#[derive(Clone, Debug)]
pub enum NodeData {
    Module,
    Block,

    // ---- expressions ----
    Name(String),
    Literal(Literal),
    /// 不可解析表达式原文（f/t-string、复杂类型、未知语法）。
    Raw(String),
    Tuple,
    List,
    Set,
    Dict,
    Binary {
        op: PyBin,
    },
    Unary {
        op: PyUn,
    },
    BoolOp {
        op: PyBoolOp,
    },
    /// children: operands；`ops.len() == children.len() - 1`
    Compare {
        ops: Vec<PyCmp>,
    },
    IfExp,
    Lambda {
        params_raw: String,
    },
    Call {
        args: Vec<CallArg>,
    },
    Attribute {
        name: String,
    },
    /// children: `[base, index]`；index 可能是 Slice/Tuple。
    Subscript,
    /// children 只含存在的部分；bool 标记 lower/upper/step 是否存在。
    Slice {
        lower: bool,
        upper: bool,
        step: bool,
        /// 是否写了第二个冒号（`a[0:2:]` 与 `a[0:2]` 在 Python 2 AST
        /// 里 step 分别是 Name(None) 与 None，必须保留）。
        step_colon: bool,
    },
    Await,
    Yield {
        from: bool,
    },
    Starred,
    DoubleStar,
    /// children: `[elt, comp_for...]`；Dict 推导为 `[key, value, comp_for...]`
    Comp {
        kind: CompKind,
    },
    /// children: `[iter, if...]`
    CompFor {
        target_raw: String,
        is_async: bool,
        if_count: usize,
    },
    /// `x := value`；children: `[value]`
    NamedExpr {
        name: String,
    },
    /// 显式括号（保留用户括号；打印/幂等稳定）。children: `[expr]`
    Paren,
    /// Python 2 反引号 repr：`` `expr` ``。children: `[expr]`
    Repr,

    // ---- statements ----
    ExprStmt,
    /// children: targets... value；`targets = children.len() - 1`
    Assign {
        targets: usize,
    },
    /// children: `[target, value?]`
    AnnAssign {
        annotation_raw: String,
    },
    /// children: `[target, value]`
    AugAssign {
        op: PyBin,
    },
    /// children: `[body]`；decorators/params/return/type params 原文
    FunctionDef {
        name: String,
        params_raw: String,
        ret_raw: String,
        deco_raw: String,
        generics_raw: String,
        is_async: bool,
    },
    /// children: `[body]`
    ClassDef {
        name: String,
        header_raw: String,
        deco_raw: String,
    },
    /// children: `[test, body, orelse?]`
    If,
    /// children: `[test, body, orelse?]`
    While,
    /// children: `[iter, body, orelse?]`
    For {
        target_raw: String,
        is_async: bool,
    },
    /// children: `[body, handler..., orelse?, finally?]`
    Try {
        star: bool,
        handler_count: usize,
        has_else: bool,
        has_finally: bool,
    },
    /// children: `[body]`；`except`/`except*` 头（不含冒号）原文
    ExceptHandler {
        clause_raw: String,
    },
    /// children: `[body]`
    With {
        items_raw: String,
        is_async: bool,
    },
    /// children: `[subject, case...]`
    Match,
    /// children: `[body]` 或 `[guard, body]`
    Case {
        pattern_raw: String,
        has_guard: bool,
    },
    /// children: `[value?]`
    Return,
    /// children: `[exc?, cause?]`；`from` 标记
    Raise {
        from: bool,
    },
    /// children: `[test, msg?]`
    Assert,
    Delete,
    Import(String),
    FromImport(String),
    Global(String),
    Nonlocal(String),
    /// Python 2 print 语句；children: `[dest?, values...]`
    Print {
        dest: bool,
        nl: bool,
    },
    Pass,
    Break,
    Continue,
    /// 语句级原文保真（整行/整块；首行缩进已剥离）。
    RawStmt(String),
}

#[derive(Clone, Debug)]
pub struct Node {
    pub data: NodeData,
    pub children: Vec<PyId>,
}

#[derive(Clone, Debug)]
pub struct PyAst {
    nodes: Vec<Node>,
    /// 解析时记录的源行号（诊断/调试；不影响语义）。
    lines: HashMap<PyId, u32>,
}

impl Default for PyAst {
    fn default() -> Self {
        Self::new()
    }
}

impl PyAst {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            lines: HashMap::new(),
        }
    }

    pub fn push(&mut self, data: NodeData, children: Vec<PyId>) -> PyId {
        let id = PyId(self.nodes.len() as u32);
        self.nodes.push(Node { data, children });
        id
    }

    pub fn with_line(mut self, id: PyId, line: u32) -> Self {
        self.lines.insert(id, line);
        self
    }

    pub fn set_line(&mut self, id: PyId, line: u32) {
        self.lines.insert(id, line);
    }

    pub fn line_of(&self, id: PyId) -> Option<u32> {
        self.lines.get(&id).copied()
    }

    pub fn data(&self, id: PyId) -> &NodeData {
        &self.nodes[id.index()].data
    }

    pub fn data_mut(&mut self, id: PyId) -> &mut NodeData {
        &mut self.nodes[id.index()].data
    }

    pub fn children(&self, id: PyId) -> &[PyId] {
        &self.nodes[id.index()].children
    }

    pub fn set_child(&mut self, parent: PyId, index: usize, new: PyId) {
        let ch = &mut self.nodes[parent.index()].children;
        if index < ch.len() {
            ch[index] = new;
        }
    }

    pub fn set_children(&mut self, parent: PyId, ch: Vec<PyId>) {
        self.nodes[parent.index()].children = ch;
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn module(&mut self, stmts: Vec<PyId>) -> PyId {
        self.push(NodeData::Module, stmts)
    }

    pub fn block(&mut self, stmts: Vec<PyId>) -> PyId {
        self.push(NodeData::Block, stmts)
    }

    pub fn name(&mut self, name: &str) -> PyId {
        self.push(NodeData::Name(name.into()), vec![])
    }

    pub fn literal(&mut self, lit: Literal) -> PyId {
        self.push(NodeData::Literal(lit), vec![])
    }

    pub fn int(&mut self, v: i64) -> PyId {
        self.literal(Literal::int(v))
    }

    pub fn bool(&mut self, v: bool) -> PyId {
        self.literal(Literal::boolean(v))
    }

    pub fn none(&mut self) -> PyId {
        self.literal(Literal::none())
    }

    pub fn raw(&mut self, text: &str) -> PyId {
        self.push(NodeData::Raw(text.into()), vec![])
    }

    pub fn raw_stmt(&mut self, text: &str) -> PyId {
        self.push(NodeData::RawStmt(text.into()), vec![])
    }

    pub fn paren(&mut self, e: PyId) -> PyId {
        self.push(NodeData::Paren, vec![e])
    }

    pub fn binary(&mut self, op: PyBin, l: PyId, r: PyId) -> PyId {
        self.push(NodeData::Binary { op }, vec![l, r])
    }

    pub fn unary(&mut self, op: PyUn, v: PyId) -> PyId {
        self.push(NodeData::Unary { op }, vec![v])
    }

    pub fn bool_op(&mut self, op: PyBoolOp, vals: Vec<PyId>) -> PyId {
        self.push(NodeData::BoolOp { op }, vals)
    }

    pub fn compare(&mut self, operands: Vec<PyId>, ops: Vec<PyCmp>) -> PyId {
        self.push(NodeData::Compare { ops }, operands)
    }

    pub fn if_exp(&mut self, body: PyId, test: PyId, orelse: PyId) -> PyId {
        self.push(NodeData::IfExp, vec![body, test, orelse])
    }

    pub fn call(&mut self, func: PyId, args: Vec<PyId>, kinds: Vec<CallArg>) -> PyId {
        let mut ch = Vec::with_capacity(args.len() + 1);
        ch.push(func);
        ch.extend(args);
        self.push(NodeData::Call { args: kinds }, ch)
    }

    pub fn attribute(&mut self, base: PyId, name: &str) -> PyId {
        self.push(NodeData::Attribute { name: name.into() }, vec![base])
    }

    pub fn subscript(&mut self, base: PyId, index: PyId) -> PyId {
        self.push(NodeData::Subscript, vec![base, index])
    }

    pub fn slice(
        &mut self,
        lower: Option<PyId>,
        upper: Option<PyId>,
        step: Option<PyId>,
        step_colon: bool,
    ) -> PyId {
        let mut ch = Vec::new();
        if let Some(x) = lower {
            ch.push(x);
        }
        if let Some(x) = upper {
            ch.push(x);
        }
        if let Some(x) = step {
            ch.push(x);
        }
        self.push(
            NodeData::Slice {
                lower: lower.is_some(),
                upper: upper.is_some(),
                step: step.is_some(),
                step_colon,
            },
            ch,
        )
    }

    /// Python 2 反引号 repr 表达式。
    pub fn repr_(&mut self, e: PyId) -> PyId {
        self.push(NodeData::Repr, vec![e])
    }

    /// Python 2 print 语句（`nl=false` 表示末尾逗号抑制换行）。
    pub fn print_stmt(&mut self, dest: Option<PyId>, values: Vec<PyId>, nl: bool) -> PyId {
        let mut ch = Vec::with_capacity(values.len() + 1);
        if let Some(d) = dest {
            ch.push(d);
        }
        ch.extend(values);
        self.push(
            NodeData::Print {
                dest: dest.is_some(),
                nl,
            },
            ch,
        )
    }

    pub fn expr_stmt(&mut self, e: PyId) -> PyId {
        self.push(NodeData::ExprStmt, vec![e])
    }

    pub fn assign(&mut self, targets: Vec<PyId>, value: PyId) -> PyId {
        let n = targets.len();
        let mut ch = targets;
        ch.push(value);
        self.push(NodeData::Assign { targets: n }, ch)
    }

    pub fn aug_assign(&mut self, op: PyBin, target: PyId, value: PyId) -> PyId {
        self.push(NodeData::AugAssign { op }, vec![target, value])
    }

    pub fn ann_assign(&mut self, target: PyId, annotation_raw: &str, value: Option<PyId>) -> PyId {
        let mut ch = vec![target];
        if let Some(v) = value {
            ch.push(v);
        }
        self.push(
            NodeData::AnnAssign {
                annotation_raw: annotation_raw.into(),
            },
            ch,
        )
    }

    pub fn function(
        &mut self,
        name: &str,
        params_raw: &str,
        ret_raw: &str,
        deco_raw: &str,
        generics_raw: &str,
        is_async: bool,
        body: PyId,
    ) -> PyId {
        self.push(
            NodeData::FunctionDef {
                name: name.into(),
                params_raw: params_raw.into(),
                ret_raw: ret_raw.into(),
                deco_raw: deco_raw.into(),
                generics_raw: generics_raw.into(),
                is_async,
            },
            vec![body],
        )
    }

    pub fn class(&mut self, name: &str, header_raw: &str, deco_raw: &str, body: PyId) -> PyId {
        self.push(
            NodeData::ClassDef {
                name: name.into(),
                header_raw: header_raw.into(),
                deco_raw: deco_raw.into(),
            },
            vec![body],
        )
    }

    pub fn if_(&mut self, test: PyId, body: PyId, orelse: Option<PyId>) -> PyId {
        let mut ch = vec![test, body];
        if let Some(e) = orelse {
            ch.push(e);
        }
        self.push(NodeData::If, ch)
    }

    pub fn while_(&mut self, test: PyId, body: PyId, orelse: Option<PyId>) -> PyId {
        let mut ch = vec![test, body];
        if let Some(e) = orelse {
            ch.push(e);
        }
        self.push(NodeData::While, ch)
    }

    pub fn for_(
        &mut self,
        target_raw: &str,
        iter: PyId,
        body: PyId,
        orelse: Option<PyId>,
        is_async: bool,
    ) -> PyId {
        let mut ch = vec![iter, body];
        if let Some(e) = orelse {
            ch.push(e);
        }
        self.push(
            NodeData::For {
                target_raw: target_raw.into(),
                is_async,
            },
            ch,
        )
    }

    pub fn try_(
        &mut self,
        star: bool,
        body: PyId,
        handlers: Vec<PyId>,
        orelse: Option<PyId>,
        finally: Option<PyId>,
    ) -> PyId {
        let handler_count = handlers.len();
        let mut ch = vec![body];
        ch.extend(handlers);
        if let Some(e) = orelse {
            ch.push(e);
        }
        if let Some(f) = finally {
            ch.push(f);
        }
        self.push(
            NodeData::Try {
                star,
                handler_count,
                has_else: orelse.is_some(),
                has_finally: finally.is_some(),
            },
            ch,
        )
    }

    pub fn except_handler(&mut self, clause_raw: &str, body: PyId) -> PyId {
        self.push(
            NodeData::ExceptHandler {
                clause_raw: clause_raw.into(),
            },
            vec![body],
        )
    }

    pub fn with_(&mut self, items_raw: &str, is_async: bool, body: PyId) -> PyId {
        self.push(
            NodeData::With {
                items_raw: items_raw.into(),
                is_async,
            },
            vec![body],
        )
    }

    pub fn match_(&mut self, subject: PyId, cases: Vec<PyId>) -> PyId {
        let mut ch = vec![subject];
        ch.extend(cases);
        self.push(NodeData::Match, ch)
    }

    pub fn case(&mut self, pattern_raw: &str, guard: Option<PyId>, body: PyId) -> PyId {
        match guard {
            Some(g) => self.push(
                NodeData::Case {
                    pattern_raw: pattern_raw.into(),
                    has_guard: true,
                },
                vec![g, body],
            ),
            None => self.push(
                NodeData::Case {
                    pattern_raw: pattern_raw.into(),
                    has_guard: false,
                },
                vec![body],
            ),
        }
    }

    pub fn return_(&mut self, value: Option<PyId>) -> PyId {
        self.push(NodeData::Return, value.into_iter().collect())
    }

    pub fn raise_(&mut self, exc: Option<PyId>, cause: Option<PyId>) -> PyId {
        let mut ch = Vec::new();
        if let Some(e) = exc {
            ch.push(e);
        }
        if let Some(c) = cause {
            ch.push(c);
        }
        self.push(
            NodeData::Raise {
                from: cause.is_some(),
            },
            ch,
        )
    }

    pub fn assert_(&mut self, test: PyId, msg: Option<PyId>) -> PyId {
        let mut ch = vec![test];
        if let Some(m) = msg {
            ch.push(m);
        }
        self.push(NodeData::Assert, ch)
    }

    pub fn simple(&mut self, data: NodeData) -> PyId {
        self.push(data, vec![])
    }

    /// 结构等价（打印往返测试用；Python 的括号不进入 AST）。
    pub fn equiv(&self, a: PyId, b: PyId) -> bool {
        let da = self.data(a);
        let db = self.data(b);
        if std::mem::discriminant(da) != std::mem::discriminant(db) {
            return false;
        }
        match (da, db) {
            (NodeData::Name(x), NodeData::Name(y)) => x == y,
            (NodeData::Literal(x), NodeData::Literal(y)) => {
                x.kind == y.kind && x.raw == y.raw && x.int == y.int && x.bool_val == y.bool_val
            }
            (NodeData::Raw(x), NodeData::Raw(y)) => norm_text(x) == norm_text(y),
            (NodeData::Binary { op: x }, NodeData::Binary { op: y }) => x == y,
            (NodeData::Unary { op: x }, NodeData::Unary { op: y }) => x == y,
            (NodeData::BoolOp { op: x }, NodeData::BoolOp { op: y }) => x == y,
            (NodeData::Compare { ops: x }, NodeData::Compare { ops: y }) => x == y,
            (NodeData::Lambda { params_raw: x }, NodeData::Lambda { params_raw: y }) => {
                norm_text(x) == norm_text(y)
            }
            (NodeData::Call { args: x }, NodeData::Call { args: y }) => x == y,
            (NodeData::Attribute { name: x }, NodeData::Attribute { name: y }) => x == y,
            (
                NodeData::Slice {
                    lower: l1,
                    upper: u1,
                    step: s1,
                    step_colon: c1,
                },
                NodeData::Slice {
                    lower: l2,
                    upper: u2,
                    step: s2,
                    step_colon: c2,
                },
            ) => l1 == l2 && u1 == u2 && s1 == s2 && c1 == c2,
            (NodeData::Yield { from: x }, NodeData::Yield { from: y }) => x == y,
            (NodeData::Comp { kind: x }, NodeData::Comp { kind: y }) => x == y,
            (
                NodeData::CompFor {
                    target_raw: t1,
                    is_async: a1,
                    if_count: i1,
                },
                NodeData::CompFor {
                    target_raw: t2,
                    is_async: a2,
                    if_count: i2,
                },
            ) => norm_text(t1) == norm_text(t2) && a1 == a2 && i1 == i2,
            (NodeData::NamedExpr { name: x }, NodeData::NamedExpr { name: y }) => x == y,
            (NodeData::Assign { targets: x }, NodeData::Assign { targets: y }) => x == y,
            (
                NodeData::AnnAssign { annotation_raw: x },
                NodeData::AnnAssign { annotation_raw: y },
            ) => norm_text(x) == norm_text(y),
            (NodeData::AugAssign { op: x }, NodeData::AugAssign { op: y }) => x == y,
            (
                NodeData::FunctionDef {
                    name: n1,
                    params_raw: p1,
                    ret_raw: r1,
                    deco_raw: d1,
                    generics_raw: g1,
                    is_async: a1,
                },
                NodeData::FunctionDef {
                    name: n2,
                    params_raw: p2,
                    ret_raw: r2,
                    deco_raw: d2,
                    generics_raw: g2,
                    is_async: a2,
                },
            ) => {
                n1 == n2
                    && norm_text(p1) == norm_text(p2)
                    && norm_text(r1) == norm_text(r2)
                    && norm_text(d1) == norm_text(d2)
                    && norm_text(g1) == norm_text(g2)
                    && a1 == a2
            }
            (
                NodeData::ClassDef {
                    name: n1,
                    header_raw: h1,
                    deco_raw: d1,
                },
                NodeData::ClassDef {
                    name: n2,
                    header_raw: h2,
                    deco_raw: d2,
                },
            ) => n1 == n2 && norm_text(h1) == norm_text(h2) && norm_text(d1) == norm_text(d2),
            (
                NodeData::For {
                    target_raw: t1,
                    is_async: a1,
                },
                NodeData::For {
                    target_raw: t2,
                    is_async: a2,
                },
            ) => norm_text(t1) == norm_text(t2) && a1 == a2,
            (
                NodeData::Try {
                    star: s1,
                    handler_count: h1,
                    has_else: e1,
                    has_finally: f1,
                },
                NodeData::Try {
                    star: s2,
                    handler_count: h2,
                    has_else: e2,
                    has_finally: f2,
                },
            ) => s1 == s2 && h1 == h2 && e1 == e2 && f1 == f2,
            (
                NodeData::ExceptHandler { clause_raw: x },
                NodeData::ExceptHandler { clause_raw: y },
            ) => norm_text(x) == norm_text(y),
            (
                NodeData::With {
                    items_raw: x,
                    is_async: a1,
                },
                NodeData::With {
                    items_raw: y,
                    is_async: a2,
                },
            ) => norm_text(x) == norm_text(y) && a1 == a2,
            (
                NodeData::Case {
                    pattern_raw: x,
                    has_guard: g1,
                },
                NodeData::Case {
                    pattern_raw: y,
                    has_guard: g2,
                },
            ) => norm_text(x) == norm_text(y) && g1 == g2,
            (NodeData::Raise { from: x }, NodeData::Raise { from: y }) => x == y,
            (
                NodeData::Print { dest: d1, nl: n1 },
                NodeData::Print { dest: d2, nl: n2 },
            ) => d1 == d2 && n1 == n2,
            (NodeData::Import(x), NodeData::Import(y))
            | (NodeData::FromImport(x), NodeData::FromImport(y))
            | (NodeData::Global(x), NodeData::Global(y))
            | (NodeData::Nonlocal(x), NodeData::Nonlocal(y))
            | (NodeData::RawStmt(x), NodeData::RawStmt(y)) => norm_text(x) == norm_text(y),
            _ => true,
        }
    }

    /// 子树结构等价。
    pub fn subtree_equiv(&self, a: PyId, b: PyId) -> bool {
        if !self.equiv(a, b) {
            return false;
        }
        let ca = self.children(a);
        let cb = self.children(b);
        ca.len() == cb.len() && ca.iter().zip(cb).all(|(&x, &y)| self.subtree_equiv(x, y))
    }
}

fn norm_text(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 统计指标（CLI --stats 用）：节点数、判定点（if/while/for/try/match/布尔）、
/// 最大嵌套（函数/类/分支块）。
pub fn unit_metrics(ast: &PyAst, module: PyId) -> (u64, u64, u32) {
    let mut nodes = 0u64;
    let mut decisions = 0u64;
    let mut max_depth = 0u32;
    fn walk(
        ast: &PyAst,
        id: PyId,
        depth: u32,
        nodes: &mut u64,
        decisions: &mut u64,
        max_depth: &mut u32,
    ) {
        *nodes += 1;
        let d = match ast.data(id) {
            NodeData::If
            | NodeData::While
            | NodeData::For { .. }
            | NodeData::Try { .. }
            | NodeData::Match
            | NodeData::BoolOp { .. }
            | NodeData::Compare { .. }
            | NodeData::IfExp => {
                *decisions += 1;
                depth + 1
            }
            NodeData::FunctionDef { .. } | NodeData::ClassDef { .. } => depth + 1,
            _ => depth,
        };
        *max_depth = (*max_depth).max(d);
        for &c in ast.children(id) {
            walk(ast, c, d, nodes, decisions, max_depth);
        }
    }
    walk(ast, module, 0, &mut nodes, &mut decisions, &mut max_depth);
    (nodes, decisions, max_depth)
}

/// 字面量折叠的标量比较（与 Python 语义一致，供 simplify 使用）。
pub mod fold {
    use crate::Literal;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum ScalarCmp {
        Eq,
        Ne,
        Lt,
        Le,
        Gt,
        Ge,
    }

    pub fn truthy(lit: &Literal) -> Option<bool> {
        match lit.kind {
            crate::LitKind::Bool => lit.bool_val,
            crate::LitKind::None => Some(false),
            crate::LitKind::Int => lit.int.map(|v| v != 0),
            crate::LitKind::Float
            | crate::LitKind::Complex
            | crate::LitKind::Str
            | crate::LitKind::Bytes => None,
            _ => None,
        }
    }
}
