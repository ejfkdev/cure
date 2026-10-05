//! 规范节点种类表：引擎共享规则所依赖的最小结构词汇表。
//!
//! 语言 crate 把自己的 AST 映射到这些 kind 上。这里**不是** Universal AST：
//! 它只覆盖主流语句/表达式形态，语言特有节点用 [`NodeKind::Custom`] 逃逸，
//! 由语言 crate 自己的规则处理，引擎通用规则不会触碰。
//!
//! 每种 kind 的 children 布局是共享规则的契约（位置即求值/书写顺序）：

use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum NodeKind {
    // ---- 语句 ----
    /// `[stmt…]`
    Block,
    /// 空语句 `;`，无 children。
    Empty,
    /// `[expr]`
    ExprStmt,
    /// 局部变量声明，children: `[(init)?]`，名字由 `Lang::var_name` 提供。
    /// 目标变量必须可写（普通局部变量），声明的类型语义由语言侧保证。
    VarDecl,
    /// `[target, value]`；target 必须是 VarRef（复合赋值通过 `Lang::assign_op` 区分）。
    Assign,
    /// `[cond, then]` 或 `[cond, then, else]`
    If,
    /// `[cond, body]`
    While,
    /// `[body, cond]`
    DoWhile,
    /// `[(init)?, (cond)?, (step)?, body]`
    For,
    /// 增强 for：`[iterable, body]`，变量名/类型为语言侧 attr。
    ForEach,
    /// `[(value)?]`
    Return,
    /// `[(label)?]`（label 以字符串 attr 提供，不在 children）
    Break,
    Continue,
    /// `[expr]`
    Throw,
    /// `[(resource)?, try_block, (catch)?, (finally)?]`；资源是语句节点，catch 子节点。
    Try,
    /// `[body]`；参数为语言侧 attr（`var_name` 返回绑定名）。
    Catch,
    /// `[lock, block]`
    Synchronized,
    /// `[subject, case…]`
    Switch,
    /// `[label…, stmts…]`（label 数与是否 default 为语言侧 attr）
    Case,
    /// `[stmt]`，标签名为 attr。
    Label,
    /// `[(cond, (msg)?)]`
    Assert,

    // ---- 表达式 ----
    /// `[lhs, rhs]`，运算符由 `Lang::bin_op` 提供。
    Binary,
    /// `[operand]`，运算符由 `Lang::un_op` 提供。
    Unary,
    /// `[callee, arg…]`；callee 不能为空。
    Call,
    /// 字面量，值由 `Lang::literal` 提供，无 children。
    Literal,
    /// 变量读，名字由 `Lang::var_name` 提供。
    VarRef,
    /// `[cond, then, else]`
    Ternary,
    /// `[expr]`，目标类型为语言侧 attr。
    Cast,
    /// `[expr]` 纯括号分组，可被 ParenRemoval 删除。
    Paren,
    /// `[object]`，成员名为语言侧 attr（`null` 安全调用等也是语言侧语义）。
    Member,
    /// `[array, index]`
    Index,
    /// `[arg…]`，构造类型为语言侧 attr。
    New,
    /// `[size…, (init)?]`，元素类型与已给尺寸数为语言侧 attr。
    NewArray,
    /// `[elem…]`
    ArrayLit,
    /// 无 children（或语言侧自定）。
    This,
    /// `[expr]`，类型/模式绑定为语言侧 attr。`x instanceof T b`
    InstanceOf,
    /// `[body]`（body 为 Block 或表达式），参数原文为语言侧 attr。
    Lambda,
    /// `[receiver]`，成员名为语言侧 attr。`recv::name`
    MethodRef,
    /// 不可解析的原文区域：语言侧原样保留，通用规则不触碰。
    Raw,
    /// 语言特有节点逃逸口；引擎通用规则不匹配。
    Custom(u16),
}

impl NodeKind {
    /// 数组分派用的小整数键：字段变体 Custom 折叠到统一槽位。
    /// 穷尽 match —— 枚举加变体时编译器强制补分支（不会静默漏桶）。
    #[inline]
    pub fn slot(self) -> usize {
        match self {
            NodeKind::Block => 0,
            NodeKind::Empty => 1,
            NodeKind::ExprStmt => 2,
            NodeKind::VarDecl => 3,
            NodeKind::Assign => 4,
            NodeKind::If => 5,
            NodeKind::While => 6,
            NodeKind::DoWhile => 7,
            NodeKind::For => 8,
            NodeKind::ForEach => 9,
            NodeKind::Return => 10,
            NodeKind::Break => 11,
            NodeKind::Continue => 12,
            NodeKind::Throw => 13,
            NodeKind::Try => 14,
            NodeKind::Catch => 15,
            NodeKind::Synchronized => 16,
            NodeKind::Switch => 17,
            NodeKind::Case => 18,
            NodeKind::Label => 19,
            NodeKind::Assert => 20,
            NodeKind::Binary => 21,
            NodeKind::Unary => 22,
            NodeKind::Call => 23,
            NodeKind::Literal => 24,
            NodeKind::VarRef => 25,
            NodeKind::Ternary => 26,
            NodeKind::Cast => 27,
            NodeKind::Paren => 28,
            NodeKind::Member => 29,
            NodeKind::Index => 30,
            NodeKind::New => 31,
            NodeKind::NewArray => 32,
            NodeKind::ArrayLit => 33,
            NodeKind::This => 34,
            NodeKind::InstanceOf => 35,
            NodeKind::Lambda => 36,
            NodeKind::MethodRef => 37,
            NodeKind::Raw => 38,
            NodeKind::Custom(_) => 39,
        }
    }

    /// slot 的上界（分派表定容用）。
    pub const SLOT_COUNT: usize = 41;
}

impl NodeKind {
    pub fn is_stmt(self) -> bool {
        use NodeKind::*;
        matches!(
            self,
            Block | Empty | ExprStmt | VarDecl | Assign | If | While | DoWhile | For | ForEach
                | Return | Break | Continue | Throw | Switch | Case | Try | Catch
                | Synchronized | Label | Assert | Raw
        )
    }
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// 二元运算符（规范词汇表，语言负责映射）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shl,
    Shr,
    UShr,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    BitAnd,
    BitXor,
    BitOr,
    And,
    Or,
}

impl BinOp {
    pub fn is_comparison(self) -> bool {
        matches!(self, BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::Eq | BinOp::Ne)
    }
    pub fn is_short_circuit(self) -> bool {
        matches!(self, BinOp::And | BinOp::Or)
    }
    /// 可能因除零抛异常的运算。
    pub fn may_div_zero(self) -> bool {
        matches!(self, BinOp::Div | BinOp::Rem)
    }
    /// 整数算术/位运算类（用于精确整数恒等式守卫）。
    pub fn is_arith(self) -> bool {
        matches!(
            self,
            BinOp::Add
                | BinOp::Sub
                | BinOp::Mul
                | BinOp::Div
                | BinOp::Rem
                | BinOp::Shl
                | BinOp::Shr
                | BinOp::UShr
                | BinOp::BitAnd
                | BinOp::BitXor
                | BinOp::BitOr
        )
    }
}

/// 一元运算符。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UnOp {
    Not,
    Neg,
    BitNot,
    /// `++x`（读+写）
    PreInc,
    PreDec,
    /// `x++`（读+写）
    PostInc,
    PostDec,
}

impl UnOp {
    pub fn is_incdec(self) -> bool {
        matches!(
            self,
            UnOp::PreInc | UnOp::PreDec | UnOp::PostInc | UnOp::PostDec
        )
    }
}

/// 字面量值的借用视图（语言 crate 从自己的负载映射）。
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LitRef<'a> {
    Bool(bool),
    Int(i64),
    Long(i64),
    Float(f64),
    Double(f64),
    Char(char),
    Str(&'a str),
    Null,
}

impl<'a> LitRef<'a> {
    pub fn is_bool(self) -> bool {
        matches!(self, LitRef::Bool(_))
    }
    /// 整数字面量（Int/Long），返回数值。
    pub fn as_int(self) -> Option<i64> {
        match self {
            LitRef::Int(v) | LitRef::Long(v) => Some(v),
            _ => None,
        }
    }
}
