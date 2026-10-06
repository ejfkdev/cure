//! 属性测试（引擎语义验证）：随机生成 toy 程序 → 简化 → 用解释器对拍
//! **简化前后的返回值 + 调用副作用序列** 必须完全一致。
//!
//! 这是对"语义保持"承诺的直接检验：求值顺序、短路、副作用、传播、
//! 死赋值删除……任何破坏可观察语义的 rewrite 都会被这里抓住。

use std::collections::HashMap;

use cure_engine::kind::{BinOp, LitRef, NodeKind, UnOp};
use BinOp::{Add, And, Div, Eq, Ge, Gt, Le, Lt, Mul, Ne, Or, Rem, Sub};
use BinOp as B;
use UnOp::Not as UnNot;
use UnOp::Neg as UnNeg;
use cure_engine::{Config, Lang};

// ---------------------------------------------------------------------------
// Toy AST（与 tests/engine.rs 相同的结构，独立副本以便自包含）
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Lit {
    Bool(bool),
    Int(i64),
    Str(String),
    Char(char),
}

#[derive(Clone, Debug)]
struct ToyNode {
    kind: NodeKind,
    children: Vec<Id>,
    name: Option<String>,
    bin: Option<BinOp>,
    un: Option<UnOp>,
    lit: Option<Lit>,
}

#[derive(Clone, Debug, Default)]
struct Toy {
    toy_names: std::cell::RefCell<Vec<String>>,
    nodes: Vec<ToyNode>,
}

type Id = u32;

impl Toy {
    fn push(&mut self, kind: NodeKind, children: Vec<Id>, name: Option<String>) -> Id {
        let id = self.nodes.len() as Id;
        self.nodes.push(ToyNode {
            kind,
            children,
            name,
            bin: None,
            un: None,
            lit: None,
        });
        id
    }
}

impl Lang for Toy {
    type Id = Id;
    /// 名字键：&'static str 不可（运行时字符串）——用 u32 索引到 names 表。
    type NameKey = u32;

    fn kind(&self, id: Id) -> NodeKind {
        self.nodes[id as usize].kind
    }
    fn children(&self, id: Id) -> &[Id] {
        &self.nodes[id as usize].children
    }
    fn set_child(&mut self, parent: Id, index: usize, new: Id) {
        self.nodes[parent as usize].children[index] = new;
    }
    fn remove_child(&mut self, parent: Id, index: usize) {
        self.nodes[parent as usize].children.remove(index);
    }
    fn splice(&mut self, node: Id, index: usize, remove: usize, insert: Vec<Id>) {
        let ch = &mut self.nodes[node as usize].children;
        let end = (index + remove).min(ch.len());
        ch.splice(index..end, insert);
    }
    fn build_return(&mut self, value: Option<Id>) -> Id {
        self.push(NodeKind::Return, value.into_iter().collect(), None)
    }
    fn build_block(&mut self, stmts: Vec<Id>) -> Id {
        self.push(NodeKind::Block, stmts, None)
    }
    fn build_unary(&mut self, op: UnOp, operand: Id) -> Id {
        let id = self.push(NodeKind::Unary, vec![operand], None);
        self.nodes[id as usize].un = Some(op);
        id
    }
    fn build_bool(&mut self, v: bool) -> Id {
        let id = self.push(NodeKind::Literal, vec![], None);
        self.nodes[id as usize].lit = Some(Lit::Bool(v));
        id
    }
    fn build_int(&mut self, v: i64, long: bool) -> Id {
        let _ = long; // toy 不区分 long
        let id = self.push(NodeKind::Literal, vec![], None);
        self.nodes[id as usize].lit = Some(Lit::Int(v));
        id
    }
    fn build_str(&mut self, s: &str) -> Id {
        let id = self.push(NodeKind::Literal, vec![], None);
        self.nodes[id as usize].lit = Some(Lit::Str(s.to_string()));
        id
    }
    fn build_char(&mut self, c: char) -> Id {
        let id = self.push(NodeKind::Literal, vec![], None);
        self.nodes[id as usize].lit = Some(Lit::Char(c));
        id
    }
    fn build_bin(&mut self, op: BinOp, l: Id, r: Id) -> Id {
        let id = self.push(NodeKind::Binary, vec![l, r], None);
        self.nodes[id as usize].bin = Some(op);
        id
    }
    fn build_ternary(&mut self, c: Id, a: Id, b: Id) -> Id {
        self.push(NodeKind::Ternary, vec![c, a, b], None)
    }
    fn build_assign(&mut self, target: Id, value: Id) -> Id {
        self.push(NodeKind::Assign, vec![target, value], None)
    }
    fn copy_subtree(&mut self, id: Id) -> Id {
        let old = self.nodes[id as usize].children.clone();
        let mut children = Vec::with_capacity(old.len());
        for c in old {
            children.push(self.copy_subtree(c));
        }
        let n = self.nodes[id as usize].clone();
        let nid = self.nodes.len() as Id;
        self.nodes.push(n);
        let _ = children;
        // 重新挂 children（上面 clone 已含原 children，重设为新拷贝）
        let cid = nid;
        self.nodes[cid as usize].children = children;
        cid
    }

    fn effect(&self, id: Id) -> cure_engine::Effect {
        let mut e = self.own_effect(id);
        for &c in &self.nodes[id as usize].children {
            e = e.worst(self.effect(c));
        }
        e
    }
    fn is_bool(&self, id: Id) -> bool {
        let n = &self.nodes[id as usize];
        match n.kind {
            NodeKind::Literal => matches!(n.lit, Some(Lit::Bool(_))),
            NodeKind::Unary => n.un == Some(UnOp::Not),
            NodeKind::Binary => n
                .bin
                .is_some_and(|o| o.is_comparison() || o.is_short_circuit()),
            NodeKind::VarRef => true, // 生成器保证类型正确
            NodeKind::Paren => self.is_bool(n.children[0]),
            NodeKind::Ternary => self.is_bool(n.children[1]) && self.is_bool(n.children[2]),
            _ => false,
        }
    }
    fn is_exact_int(&self, id: Id) -> bool {
        let n = &self.nodes[id as usize];
        match n.kind {
            NodeKind::Literal => matches!(n.lit, Some(Lit::Int(_))),
            NodeKind::VarRef => true,
            NodeKind::Binary => n
                .bin
                .is_some_and(|o| o.is_arith() || o.is_comparison())
                && self.is_exact_int(n.children[0])
                && self.is_exact_int(n.children[1]),
            NodeKind::Paren => self.is_exact_int(n.children[0]),
            NodeKind::Unary => self.is_exact_int(n.children[0]),
            _ => false,
        }
    }
    fn bin_op(&self, id: Id) -> Option<BinOp> {
        self.nodes[id as usize].bin
    }
    fn un_op(&self, id: Id) -> Option<UnOp> {
        self.nodes[id as usize].un
    }
    fn literal(&self, id: Id) -> Option<LitRef<'_>> {
        match &self.nodes[id as usize].lit {
            Some(Lit::Bool(b)) => Some(LitRef::Bool(*b)),
            Some(Lit::Int(v)) => Some(LitRef::Int(*v)),
            Some(Lit::Str(s)) => Some(LitRef::Str(s)),
            Some(Lit::Char(c)) => Some(LitRef::Char(*c)),
            None => None,
        }
    }
    fn var_name(&self, id: Id) -> Option<&str> {
        self.nodes[id as usize].name.as_deref()
    }
    fn var_key(&self, id: Id) -> Option<u32> {
        let name = self.nodes[id as usize].name.as_deref()?;
        let mut names = self.toy_names.borrow_mut();
        Some(match names.iter().position(|n| n == name) {
            Some(i) => i as u32,
            None => {
                names.push(name.to_string());
                (names.len() - 1) as u32
            }
        })
    }
    fn node_index(&self, id: Id) -> usize {
        id as usize
    }
}

// ---------------------------------------------------------------------------
// 解释器：值 + 副作用调用日志
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Debug)]
enum V {
    Int(i64),
    Bool(bool),
    Str(String),
}

struct Interp {
    scopes: Vec<HashMap<String, V>>,
    log: Vec<String>,
}

enum Step {
    Ret(Option<V>),
    Normal,
}

/// (call name) 的确定返回值：与名字稳定相关，让程序对调用值敏感。
fn call_value(name: &str) -> V {
    let h = name.bytes().fold(0x811c9dc5u32, |a, b| {
        a.wrapping_mul(0x01000193).wrapping_add(b as u32)
    });
    if name.starts_with('g') {
        V::Bool(h % 2 == 0)
    } else if name.starts_with('s') {
        V::Str(format!("s{}", h % 7))
    } else {
        V::Int(1 + (h % 97) as i64)
    }
}

impl Interp {
    fn new() -> Self {
        Interp {
            scopes: vec![HashMap::new()],
            log: Vec::new(),
        }
    }

    fn lookup(&self, n: &str) -> V {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.get(n))
            .cloned()
            .unwrap_or_else(|| panic!("unbound var {n}"))
    }

    fn stmt(&mut self, t: &Toy, id: Id) -> Step {
        match t.kind(id) {
            NodeKind::Block => {
                self.scopes.push(HashMap::new());
                for &s in t.children(id) {
                    match self.stmt(t, s) {
                        Step::Ret(v) => {
                            self.scopes.pop();
                            return Step::Ret(v);
                        }
                        Step::Normal => {}
                    }
                }
                self.scopes.pop();
                Step::Normal
            }
            NodeKind::Empty => Step::Normal,
            NodeKind::ExprStmt => {
                if let Some(&e) = t.children(id).first() {
                    self.expr(t, e);
                }
                Step::Normal
            }
            NodeKind::VarDecl => {
                let name = t.var_name(id).unwrap().to_string();
                let v = t
                    .children(id)
                    .first()
                    .map(|&e| self.expr(t, e))
                    .unwrap_or(V::Int(0));
                self.scopes.last_mut().unwrap().insert(name, v);
                Step::Normal
            }
            NodeKind::Assign => {
                let ch = t.children(id);
                let name = t.var_name(ch[0]).unwrap().to_string();
                let v = self.expr(t, ch[1]);
                for s in self.scopes.iter_mut().rev() {
                    if s.contains_key(&name) {
                        s.insert(name, v);
                        return Step::Normal;
                    }
                }
                self.scopes[0].insert(name, v);
                Step::Normal
            }
            NodeKind::If => {
                let ch = t.children(id);
                let c = self.expr(t, ch[0]);
                if c == V::Bool(true) {
                    self.stmt(t, ch[1])
                } else if let Some(&e) = ch.get(2) {
                    self.stmt(t, e)
                } else {
                    Step::Normal
                }
            }
            NodeKind::Return => match t.children(id).first() {
                Some(&e) => Step::Ret(Some(self.expr(t, e))),
                None => Step::Ret(None),
            },
            other => panic!("unexpected stmt {other:?}"),
        }
    }

    fn expr(&mut self, t: &Toy, id: Id) -> V {
        match t.kind(id) {
            NodeKind::Literal => match t.literal(id).unwrap() {
                LitRef::Bool(b) => V::Bool(b),
                LitRef::Int(v) => V::Int(v),
                LitRef::Str(s) => V::Str(s.to_string()),
                _ => V::Int(0),
            },
            NodeKind::VarRef => self.lookup(t.var_name(id).unwrap()),
            NodeKind::Paren => self.expr(t, t.children(id)[0]),
            NodeKind::Call => {
                // callee 形如 (var f2)
                let callee = t.children(id)[0];
                let name = t.var_name(callee).unwrap_or("f0").to_string();
                self.log.push(name.clone());
                call_value(&name)
            }
            NodeKind::Unary => match t.un_op(id).unwrap() {
                UnOp::Not => {
                    let v = self.expr(t, t.children(id)[0]);
                    match v {
                        V::Bool(b) => V::Bool(!b),
                        other => panic!("! on {other:?}"),
                    }
                }
                UnOp::Neg => match self.expr(t, t.children(id)[0]) {
                    V::Int(i) => V::Int(-i),
                    other => panic!("- on {other:?}"),
                },
                op => panic!("unexpected unop {op:?}"),
            },
            NodeKind::Binary => {
                let op = t.bin_op(id).unwrap();
                let ch = t.children(id);
                if op == BinOp::And {
                    let l = self.expr(t, ch[0]);
                    return match l {
                        V::Bool(false) => V::Bool(false), // 短路：右侧不求值
                        V::Bool(true) => self.expr(t, ch[1]),
                        o => panic!("&& on {o:?}"),
                    };
                }
                if op == BinOp::Or {
                    let l = self.expr(t, ch[0]);
                    return match l {
                        V::Bool(true) => V::Bool(true),
                        V::Bool(false) => self.expr(t, ch[1]),
                        o => panic!("|| on {o:?}"),
                    };
                }
                let l = self.expr(t, ch[0]);
                let r = self.expr(t, ch[1]);
                self.binop(op, l, r)
            }
            NodeKind::Ternary => {
                let ch = t.children(id);
                let c = self.expr(t, ch[0]);
                if c == V::Bool(true) {
                    self.expr(t, ch[1])
                } else {
                    self.expr(t, ch[2])
                }
            }
            other => panic!("unexpected expr {other:?}"),
        }
    }

    fn binop(&self, op: BinOp, l: V, r: V) -> V {
        use BinOp::{Add, And, Div, Eq, Ge, Gt, Le, Lt, Mul, Ne, Or, Rem, Sub};
use BinOp as B;
        match (l, r) {
            (V::Int(a), V::Int(b)) => {
                if op.is_comparison() {
                    V::Bool(match op {
                        B::Lt => a < b,
                        B::Le => a <= b,
                        B::Gt => a > b,
                        B::Ge => a >= b,
                        B::Eq => a == b,
                        B::Ne => a != b,
                        _ => unreachable!(),
                    })
                } else {
                    V::Int(match op {
                        B::Add => a.wrapping_add(b),
                        B::Sub => a.wrapping_sub(b),
                        B::Mul => a.wrapping_mul(b),
                        B::Div => {
                            if b == 0 {
                                0 // 生成器不产生除零；防御
                            } else {
                                a.wrapping_div(b)
                            }
                        }
                        B::Rem => {
                            if b == 0 {
                                0
                            } else {
                                a.wrapping_rem(b)
                            }
                        }
                        B::BitAnd => a & b,
                        B::BitXor => a ^ b,
                        B::BitOr => a | b,
                        _ => panic!("bad int op {op:?}"),
                    })
                }
            }
            (V::Str(a), V::Str(b)) => match op {
                B::Add => V::Str(format!("{a}{b}")),
                _ => panic!("str op {op:?}"),
            },
            (V::Bool(a), V::Bool(b)) => V::Bool(match op {
                B::Eq => a == b,
                B::Ne => a != b,
                _ => panic!("bool op {op:?}"),
            }),
            (l, r) => panic!("bad operands {l:?} {r:?} for {op:?}"),
        }
    }
}

fn interp(t: &Toy, root: Id) -> (Option<V>, Vec<String>) {
    let mut it = Interp::new();
    let out = match it.stmt(t, root) {
        Step::Ret(v) => v,
        Step::Normal => None,
    };
    (out, it.log)
}

// ---------------------------------------------------------------------------
// 随机程序生成（确定性 LCG；类型导向）
// ---------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn range(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Ty {
    Int,
    Bool,
    Str,
}

const INT_VARS: &[&str] = &["i0", "i1", "i2", "i3"];
const BOOL_VARS: &[&str] = &["b0", "b1", "b2"];
const STR_VARS: &[&str] = &["sv0", "sv1"];
const INT_CALLS: &[&str] = &["f0", "f1", "f2", "f3"];
const BOOL_CALLS: &[&str] = &["g0", "g1", "g2"];
const STR_CALLS: &[&str] = &["sc0", "sc1"];

struct Gen<'a> {
    rng: Rng,
    t: &'a mut Toy,
    temp_n: u32,
}

impl<'a> Gen<'a> {
    fn str_lit(&mut self) -> Id {
        let n = self.rng.range(3) as usize;
        let text = ["a", "b", "c"][n].to_string();
        let id = self.t.push(NodeKind::Literal, vec![], None);
        self.t.nodes[id as usize].lit = Some(Lit::Str(text));
        id
    }

    fn int_lit(&mut self) -> Id {
        // 偏置：30% 取 0/1（触发恒等式/零元素规则）
        let v = match self.rng.range(10) {
            0..=1 => self.rng.range(2) as i64,
            2 => -1, // 驱动 x & -1 / x | -1
            _ => self.rng.range(100) as i64,
        };
        let id = self.t.push(NodeKind::Literal, vec![], None);
        self.t.nodes[id as usize].lit = Some(Lit::Int(v));
        id
    }

    fn var(&mut self, name: &str) -> Id {
        self.t.push(NodeKind::VarRef, vec![], Some(name.to_string()))
    }

    fn call(&mut self, name: &str) -> Id {
        let callee = self.var(name);
        self.t.push(NodeKind::Call, vec![callee], None)
    }

    fn bin(&mut self, op: BinOp, l: Id, r: Id) -> Id {
        let id = self.t.push(NodeKind::Binary, vec![l, r], None);
        self.t.nodes[id as usize].bin = Some(op);
        id
    }

    fn un(&mut self, op: UnOp, e: Id) -> Id {
        let id = self.t.push(NodeKind::Unary, vec![e], None);
        self.t.nodes[id as usize].un = Some(op);
        id
    }

    fn expr(&mut self, ty: Ty, depth: u32) -> Id {
        use BinOp::{Add, And, Div, Eq, Ge, Gt, Le, Lt, Mul, Ne, Or, Rem, Sub};
use BinOp as B;
        if depth == 0 {
            return match ty {
                Ty::Int => {
                    let k = self.rng.range(6);
                    if k <= 2 {
                        self.int_lit()
                    } else if k <= 4 {
                        let i = self.rng.range(4) as usize;
                        self.var(INT_VARS[i])
                    } else {
                        let i = self.rng.range(4) as usize;
                        self.call(INT_CALLS[i])
                    }
                }
                Ty::Str => {
                    let k = self.rng.range(4);
                    if k == 0 {
                        self.str_lit()
                    } else if k <= 2 {
                        let i = self.rng.range(2) as usize;
                        self.var(STR_VARS[i])
                    } else {
                        let i = self.rng.range(2) as usize;
                        self.call(STR_CALLS[i])
                    }
                }
                Ty::Bool => {
                    let k = self.rng.range(6);
                    if k == 0 || k == 1 {
                        let id = self.t.push(NodeKind::Literal, vec![], None);
                        self.t.nodes[id as usize].lit = Some(Lit::Bool(k == 0));
                        id
                    } else if k <= 3 {
                        let i = self.rng.range(3) as usize;
                        self.var(BOOL_VARS[i])
                    } else {
                        let i = self.rng.range(3) as usize;
                        self.call(BOOL_CALLS[i])
                    }
                }
            };
        }
        let d = depth - 1;
        match ty {
            Ty::Str => {
                let k = self.rng.range(6);
                if k <= 1 {
                    self.str_lit()
                } else if k == 2 {
                    let i = self.rng.range(2) as usize;
                    self.var(STR_VARS[i])
                } else if k == 3 {
                    let i = self.rng.range(2) as usize;
                    self.call(STR_CALLS[i])
                } else if k == 4 {
                    // 拼接（驱动 reassoc/常量折叠）
                    let a = self.expr(Ty::Str, d);
                    let b = self.expr(Ty::Str, d);
                    self.bin(B::Add, a, b)
                } else {
                    let e = self.expr(Ty::Str, d);
                    self.t.push(NodeKind::Paren, vec![e], None)
                }
            }
            Ty::Int => match self.rng.range(8) {
                0 => {
                    let a = self.expr(Ty::Int, d);
                    let b = self.expr(Ty::Int, d);
                    self.bin(Add, a, b)
                }
                1 => {
                    let a = self.expr(Ty::Int, d);
                    let b = self.expr(Ty::Int, d);
                    self.bin(Sub, a, b)
                }
                2 => {
                    let a = self.expr(Ty::Int, d);
                    let b = self.expr(Ty::Int, d);
                    self.bin(Mul, a, b)
                }
                3 => {
                    let e = self.expr(Ty::Int, d);
                    self.un(UnNeg, e)
                }
                4 => {
                    let e = self.expr(Ty::Int, d);
                    self.t.push(NodeKind::Paren, vec![e], None)
                }
                5 => {
                    let c = self.expr(Ty::Bool, d);
                    let a = self.expr(Ty::Int, d);
                    let b = self.expr(Ty::Int, d);
                    self.t.push(NodeKind::Ternary, vec![c, a, b], None)
                }
                6 => self.int_lit(),
                7 => {
                    // 位运算（驱动 bit_identity / arith_reassoc / const_fold）
                    let op = [B::BitXor, B::BitAnd, B::BitOr][self.rng.range(3) as usize];
                    let a = self.expr(Ty::Int, d);
                    let b = self.expr(Ty::Int, d);
                    self.bin(op, a, b)
                }
                _ => {
                    let i = self.rng.range(4) as usize;
                    self.var(INT_VARS[i])
                }
            },
            Ty::Bool => {
                let k = self.rng.range(10);
                let cmp = [Lt, Le, Gt, Ge, Eq, Ne];
                match k {
                    0..=2 => {
                        let op = cmp[self.rng.range(6) as usize];
                        let a = self.expr(Ty::Int, d);
                        let b = self.expr(Ty::Int, d);
                        self.bin(op, a, b)
                    }
                    3..=4 => {
                        let op = if self.rng.range(2) == 0 { And } else { Or };
                        let a = self.expr(Ty::Bool, d);
                        let b = self.expr(Ty::Bool, d);
                        self.bin(op, a, b)
                    }
                    5 => {
                    let e = self.expr(Ty::Bool, d);
                    self.un(UnNot, e)
                }
                    6 => {
                        // ==(bool, bool)（触发 bool_compare 需要 true/false 字面量一侧）
                        let op = if self.rng.range(2) == 0 { Eq } else { Ne };
                        let a = self.expr(Ty::Bool, d);
                        let b = self.expr(Ty::Bool, d);
                        self.bin(op, a, b)
                    }
                    7 => {
                        let e = self.expr(Ty::Bool, d);
                        self.t.push(NodeKind::Paren, vec![e], None)
                    }
                    8 => {
                        let c = self.expr(Ty::Bool, d);
                        let a = self.expr(Ty::Bool, d);
                        let b = self.expr(Ty::Bool, d);
                        self.t.push(NodeKind::Ternary, vec![c, a, b], None)
                    }
                    _ => {
                        let i = self.rng.range(3) as usize;
                        self.call(BOOL_CALLS[i])
                    }
                }
            }
        }
    }

    fn stmt(&mut self, depth: u32) -> Id {
        let kind = self.rng.range(8);
        match kind {
            0 => {
                // 声明临时变量
                let ty = if self.rng.range(2) == 0 { Ty::Int } else { Ty::Bool };
                let name = format!("t{}", self.temp_n);
                self.temp_n += 1;
                let init = self.expr(ty, 2);
                let id = self.t.push(NodeKind::VarDecl, vec![init], Some(name));
                id
            }
            1 | 2 => {
                // 赋值既有变量（含字符串变量）
                let k = self.rng.range(4);
                let (name, ty) = if k == 0 {
                    let i = self.rng.range(3) as usize;
                    (BOOL_VARS[i].to_string(), Ty::Bool)
                } else if k == 3 {
                    let i = self.rng.range(2) as usize;
                    (STR_VARS[i].to_string(), Ty::Str)
                } else {
                    let i = self.rng.range(4) as usize;
                    (INT_VARS[i].to_string(), Ty::Int)
                };
                let v = self.expr(ty, 2);
                let target = self.var(&name);
                self.t.push(NodeKind::Assign, vec![target, v], None)
            }
            7 => {
                // 裸声明（后续可能被赋值——驱动 decl_assign_merge）
                let ty = [Ty::Int, Ty::Bool, Ty::Str][self.rng.range(3) as usize];
                let name = format!("t{}", self.temp_n);
                self.temp_n += 1;
                self.t.push(NodeKind::VarDecl, vec![], Some(name))
            }
            3 => {
                // 纯副作用调用语句
                let name = if self.rng.range(2) == 0 {
                    let i = self.rng.range(4) as usize;
                    INT_CALLS[i]
                } else {
                    let i = self.rng.range(3) as usize;
                    BOOL_CALLS[i]
                };
                let c = self.call(name);
                self.t.push(NodeKind::ExprStmt, vec![c], None)
            }
            4 if depth > 0 => {
                // if（带/不带 else）
                let c = self.expr(Ty::Bool, 2);
                let then = self.block(depth - 1);
                if self.rng.range(2) == 0 {
                    let els = self.block(depth - 1);
                    self.t.push(NodeKind::If, vec![c, then, els], None)
                } else {
                    self.t.push(NodeKind::If, vec![c, then], None)
                }
            }
            5 if depth > 0 => {
                // 提前 return（驱动 if→三元 / 布尔返回 归并）
                let ty = if self.rng.range(2) == 0 { Ty::Int } else { Ty::Bool };
                let e = self.expr(ty, 2);
                self.t.push(NodeKind::Return, vec![e], None)
            }
            _ => {
                // 再来一个表达式语句
                let ty = if self.rng.range(2) == 0 { Ty::Int } else { Ty::Bool };
                let e = self.expr(ty, 2);
                self.t.push(NodeKind::ExprStmt, vec![e], None)
            }
        }
    }

    fn block(&mut self, depth: u32) -> Id {
        let n = 1 + self.rng.range(3) as usize;
        let mut stmts = Vec::new();
        for _ in 0..n {
            stmts.push(self.stmt(depth));
        }
        self.t.push(NodeKind::Block, stmts, None)
    }
}

fn gen_program(seed: u64) -> Toy {
    let mut rng = Rng(seed);
    let mut t = Toy::default();
    let mut stmts = Vec::new();
    // 预声明变量
    for name in INT_VARS {
        let id = t.push(NodeKind::Literal, vec![], None);
        t.nodes[id as usize].lit = Some(Lit::Int(rng.range(50) as i64));
        let d = t.push(NodeKind::VarDecl, vec![id], Some(name.to_string()));
        stmts.push(d);
    }
    for name in BOOL_VARS {
        let id = t.push(NodeKind::Literal, vec![], None);
        t.nodes[id as usize].lit = Some(Lit::Bool(rng.range(2) == 0));
        let d = t.push(NodeKind::VarDecl, vec![id], Some(name.to_string()));
        stmts.push(d);
    }
    for name in STR_VARS {
        let n = rng.range(3) as usize;
        let id = t.push(NodeKind::Literal, vec![], None);
        t.nodes[id as usize].lit = Some(Lit::Str(["a", "b", "c"][n].to_string()));
        let d = t.push(NodeKind::VarDecl, vec![id], Some(name.to_string()));
        stmts.push(d);
    }
    let n = 2 + rng.range(4) as usize;
    let mut g = Gen {
        rng,
        t: &mut t,
        temp_n: 0,
    };
    for _ in 0..n {
        stmts.push(g.stmt(1));
    }
    // 末尾 return
    let ret_ty = [Ty::Int, Ty::Bool, Ty::Str][g.rng.range(3) as usize];
    let e = g.expr(ret_ty, 2);
    let r = g.t.push(NodeKind::Return, vec![e], None);
    stmts.push(r);
    let root = g.t.push(NodeKind::Block, stmts, None);
    // Toy 借用结束；root 恰为最后一个节点
    let _ = root;
    t
}

// ---------------------------------------------------------------------------
// S-expr 打印（失败时展示程序）
// ---------------------------------------------------------------------------

fn to_sexp(t: &Toy, id: Id) -> String {
    let n = &t.nodes[id as usize];
    let ch: Vec<String> = n.children.iter().map(|&c| to_sexp(t, c)).collect();
    match n.kind {
        NodeKind::Block => format!("(block {})", ch.join(" ")),
        NodeKind::Empty => "(empty)".into(),
        NodeKind::ExprStmt => format!("(expr {})", ch.first().cloned().unwrap_or_default()),
        NodeKind::VarDecl => format!(
            "(decl {} {})",
            n.name.clone().unwrap_or_default(),
            ch.first().cloned().unwrap_or_default()
        ),
        NodeKind::Assign => format!("(assign {} {})", ch[0], ch[1]),
        NodeKind::If => format!("(if {})", ch.join(" ")),
        NodeKind::Return => format!("(ret {})", ch.first().cloned().unwrap_or_default()),
        NodeKind::Binary => format!(
            "({:?} {} {})",
            n.bin.unwrap(),
            ch[0],
            ch[1]
        ),
        NodeKind::Unary => format!("({:?} {})", n.un.unwrap(), ch[0]),
        NodeKind::Call => format!("(call {})", ch[0]),
        NodeKind::Ternary => format!("(ternary {} {} {})", ch[0], ch[1], ch[2]),
        NodeKind::Paren => format!("(paren {})", ch[0]),
        NodeKind::VarRef => n.name.clone().unwrap_or_default(),
        NodeKind::Literal => match &n.lit {
            Some(Lit::Bool(b)) => format!("{b}"),
            Some(Lit::Int(v)) => format!("{v}"),
            Some(Lit::Str(s)) => format!("\"{s}\""),
            Some(Lit::Char(c)) => format!("'{c}'"),
            None => "?".into(),
        },
        k => format!("(:{k:?})"),
    }
}

// ---------------------------------------------------------------------------
// 属性测试本体
// ---------------------------------------------------------------------------

#[test]
fn random_programs_preserve_semantics() {
    let rules = cure_engine::rules::default_rules::<Toy>();
    let cfg = Config::default();
    let mut checked = 0u32;
    let mut total_edits = 0usize;

    // 多个确定性种子批次
    for seed in 1..=80u64 {
        let toy = gen_program(seed);
        let root = (toy.nodes.len() - 1) as Id;
        let before = interp(&toy, root);

        // 深拷贝一份做简化（简化是原位修改）
        let mut simplified = clone_toy(&toy);
        let report = cure_engine::simplify(&mut simplified, root, &rules, &cfg);
        total_edits += report.edits;
        let after = interp(&simplified, root);

        if before != after {
            panic!(
                "语义改变！seed={seed}\n== 原程序 ==\n{}\n== 简化后 ==\n{}\n== before ==\n{before:?}\n== after ==\n{after:?}",
                to_sexp(&toy, root),
                to_sexp(&simplified, root),
            );
        }
        checked += 1;
    }
    // 确保这批程序确实触发了简化（否则属性测试形同虚设）
    assert!(total_edits > 200, "batch too weak: {total_edits} edits");
    eprintln!("property: {checked} programs, {total_edits} edits, all equivalent");
}

fn clone_toy(t: &Toy) -> Toy {
    Toy {
        toy_names: std::cell::RefCell::new(Vec::new()),
        nodes: t.nodes.clone(),
    }
}

#[test]
fn idempotent_second_run_makes_no_edits() {
    let rules = cure_engine::rules::default_rules::<Toy>();
    let cfg = Config::default();
    for seed in 1..=40u64 {
        let toy = gen_program(seed);
        let root = (toy.nodes.len() - 1) as Id;
        let mut t1 = clone_toy(&toy);
        let r1 = cure_engine::simplify(&mut t1, root, &rules, &cfg);
        // 第二轮：fixed point 必须已达成
        let r2 = cure_engine::simplify(&mut t1, root, &rules, &cfg);
        assert_eq!(
            r2.edits, 0,
            "seed {seed}: second run still edits (oscillation?)\n{}",
            to_sexp(&t1, root)
        );
        let _ = r1;
    }
}
