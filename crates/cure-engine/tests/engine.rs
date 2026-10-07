//! 引擎集成测试：内置一个微型 toy 语言（S-expression 源码）作为 `Lang` 的参考实现。
//!
//! 这也是未来接入新语言（JS/Python/…）时的样板：实现 Lang → 注册规则 → 跑 golden。

use std::collections::HashSet;

use cure_engine::kind::{BinOp, LitRef, NodeKind, UnOp};
use cure_engine::{Config, Effect, Edit, Lang, RewriteCtx, Rule};

// ---------------------------------------------------------------------------
// Toy AST：arena + 极简负载
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Lit {
    Bool(bool),
    Int(i64),
    Long(i64),
    Float(f64),
    Double(f64),
    Str(String),
    Char(char),
    Null,
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

#[derive(Clone, Debug)]
struct Toy {
    toy_names: std::cell::RefCell<Vec<String>>,
    nodes: Vec<ToyNode>,
    root: Id,
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
    fn with_bin(self_id: Id, nodes: &mut Vec<ToyNode>, op: BinOp) -> Id {
        nodes[self_id as usize].bin = Some(op);
        self_id
    }
}

// ---------------------------------------------------------------------------
// Lang 实现
// ---------------------------------------------------------------------------

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
        let id = self.push(NodeKind::Literal, vec![], None);
        self.nodes[id as usize].lit = Some(if long { Lit::Long(v) } else { Lit::Int(v) });
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
        let old_children = self.nodes[id as usize].children.clone();
        let mut new_children = Vec::with_capacity(old_children.len());
        for c in old_children {
            new_children.push(self.copy_subtree(c));
        }
        let mut n = self.nodes[id as usize].clone();
        n.children = new_children;
        let nid = self.nodes.len() as Id;
        self.nodes.push(n);
        nid
    }

    fn effect(&self, id: Id) -> Effect {
        let mut e = self.own_effect(id);
        for &c in &self.nodes[id as usize].children {
            e = e.worst(self.effect(c));
        }
        e
    }
    // own_effect 用引擎默认实现即可

    fn is_bool(&self, id: Id) -> bool {
        let n = &self.nodes[id as usize];
        match n.kind {
            NodeKind::Literal => matches!(n.lit, Some(Lit::Bool(_))),
            NodeKind::Unary => n.un == Some(UnOp::Not),
            NodeKind::Binary => n
                .bin
                .is_some_and(|o| o.is_comparison() || matches!(o, BinOp::And | BinOp::Or)),
            // toy 变量无类型信息，测试里视为 bool（仅用于让规则得以触发）
            NodeKind::VarRef => true,
            NodeKind::Paren => self.is_bool(n.children[0]),
            NodeKind::Ternary => self.is_bool(n.children[1]) && self.is_bool(n.children[2]),
            _ => false,
        }
    }
    fn is_exact_int(&self, id: Id) -> bool {
        let n = &self.nodes[id as usize];
        match n.kind {
            NodeKind::Literal => matches!(n.lit, Some(Lit::Int(_) | Lit::Long(_))),
            NodeKind::VarRef => true, // toy 变量视为 int
            NodeKind::Binary => n
                .bin
                .is_some_and(|o| o.is_arith() || o.is_comparison())
                && self.is_exact_int(n.children[0])
                && self.is_exact_int(n.children[1]),
            NodeKind::Paren => self.is_exact_int(n.children[0]),
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
            Some(Lit::Long(v)) => Some(LitRef::Long(*v)),
            Some(Lit::Float(v)) => Some(LitRef::Float(*v)),
            Some(Lit::Double(v)) => Some(LitRef::Double(*v)),
            Some(Lit::Str(s)) => Some(LitRef::Str(s)),
            Some(Lit::Char(c)) => Some(LitRef::Char(*c)),
            Some(Lit::Null) => Some(LitRef::Null),
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
    fn id_of_index(&self, idx: usize) -> Id {
        idx as u32
    }
}

// ---------------------------------------------------------------------------
// S-expression 解析器
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum V {
    List(Vec<V>),
    Sym(String),
    Str(String),
}

fn tokenize(src: &str) -> Vec<String> {
    let mut toks = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            '(' | ')' => {
                toks.push(c.to_string());
                chars.next();
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                for c2 in chars.by_ref() {
                    if c2 == '"' {
                        break;
                    }
                    s.push(c2);
                }
                toks.push(format!("\u{1}{}", s)); // \u{1} 前缀标记字符串字面量
            }
            c if c.is_whitespace() => {
                chars.next();
            }
            _ => {
                let mut s = String::new();
                loop {
                    match chars.peek() {
                        None => break,
                        Some(&c2) if c2.is_whitespace() || c2 == '(' || c2 == ')' => break,
                        Some(&c2) => {
                            s.push(c2);
                            chars.next();
                        }
                    }
                }
                toks.push(s);
            }
        }
    }
    toks
}

fn parse_tokens(toks: &[String], pos: &mut usize) -> V {
    let t = &toks[*pos];
    *pos += 1;
    if t == "(" {
        let mut items = Vec::new();
        while toks[*pos] != ")" {
            items.push(parse_tokens(toks, pos));
        }
        *pos += 1;
        V::List(items)
    } else if let Some(s) = t.strip_prefix('\u{1}') {
        V::Str(s.to_string())
    } else {
        V::Sym(t.clone())
    }
}

fn parse(src: &str) -> Toy {
    let toks = tokenize(src);
    let mut pos = 0;
    let v = parse_tokens(&toks, &mut pos);
    let mut toy = Toy {
        toy_names: std::cell::RefCell::new(Vec::new()),
        nodes: Vec::new(),
        root: 0,
    };
    toy.root = build(&mut toy, &v);
    toy
}

const BIN_OPS: &[(&str, BinOp)] = &[
    ("+", BinOp::Add),
    ("-", BinOp::Sub),
    ("*", BinOp::Mul),
    ("/", BinOp::Div),
    ("%", BinOp::Rem),
    ("<<", BinOp::Shl),
    (">>", BinOp::Shr),
    (">>>", BinOp::UShr),
    ("<", BinOp::Lt),
    ("<=", BinOp::Le),
    (">", BinOp::Gt),
    (">=", BinOp::Ge),
    ("==", BinOp::Eq),
    ("!=", BinOp::Ne),
    ("&", BinOp::BitAnd),
    ("^", BinOp::BitXor),
    ("|", BinOp::BitOr),
    ("&&", BinOp::And),
    ("||", BinOp::Or),
];

const UN_OPS: &[(&str, UnOp)] = &[
    ("not", UnOp::Not),
    ("neg", UnOp::Neg),
    ("bnot", UnOp::BitNot),
    ("preinc", UnOp::PreInc),
    ("predec", UnOp::PreDec),
    ("postinc", UnOp::PostInc),
    ("postdec", UnOp::PostDec),
];

fn build(toy: &mut Toy, v: &V) -> Id {
    match v {
        V::Str(s) => {
            let id = toy.push(NodeKind::Literal, vec![], None);
            toy.nodes[id as usize].lit = Some(Lit::Str(s.clone()));
            id
        }
        V::Sym(s) => build_atom(toy, s),
        V::List(items) => {
            let head = match &items[0] {
                V::Sym(s) => s.as_str(),
                _ => panic!("list head must be a symbol: {v:?}"),
            };
            let rest = &items[1..];
            let child_ids: Vec<Id> = rest.iter().map(|c| build(toy, c)).collect();
            match head {
                "block" => toy.push(NodeKind::Block, child_ids, None),
                "empty" => toy.push(NodeKind::Empty, vec![], None),
                "if" => toy.push(NodeKind::If, child_ids, None),
                "while" => toy.push(NodeKind::While, child_ids, None),
                "ret" => toy.push(NodeKind::Return, child_ids, None),
                "expr" => toy.push(NodeKind::ExprStmt, child_ids, None),
                "throw" => toy.push(NodeKind::Throw, child_ids, None),
                "ternary" => toy.push(NodeKind::Ternary, child_ids, None),
                "paren" => toy.push(NodeKind::Paren, child_ids, None),
                "this" => toy.push(NodeKind::This, vec![], None),
                "call" => toy.push(NodeKind::Call, child_ids, None),
                "member" => {
                    let id = toy.push(NodeKind::Member, child_ids, None);
                    toy.nodes[id as usize].name = sym_of(&rest[1]).map(|s| s.to_string());
                    id
                }
                "index" => toy.push(NodeKind::Index, child_ids, None),
                "assign" => toy.push(NodeKind::Assign, child_ids, None),
                "decl" => {
                    let name = sym_of(&rest[0]).unwrap().to_string();
                    let id = toy.push(NodeKind::VarDecl, child_ids[1..].to_vec(), Some(name));
                    id
                }
                "new" => toy.push(NodeKind::New, child_ids, None),
                "arr" => toy.push(NodeKind::ArrayLit, child_ids, None),
                _ => {
                    if let Some((_, op)) = BIN_OPS.iter().find(|(s, _)| *s == head) {
                        let id = toy.push(NodeKind::Binary, child_ids, None);
                        toy.nodes[id as usize].bin = Some(*op);
                        id
                    } else if let Some((_, op)) = UN_OPS.iter().find(|(s, _)| *s == head) {
                        let id = toy.push(NodeKind::Unary, child_ids, None);
                        toy.nodes[id as usize].un = Some(*op);
                        id
                    } else {
                        panic!("unknown toy form: {head}")
                    }
                }
            }
        }
    }
}

fn sym_of(v: &V) -> Option<&str> {
    match v {
        V::Sym(s) => Some(s),
        _ => None,
    }
}

fn build_atom(toy: &mut Toy, s: &str) -> Id {
    let set_lit = |toy: &mut Toy, id: Id, lit: Lit| {
        toy.nodes[id as usize].lit = Some(lit);
        id
    };
    match s {
        "true" => {
            let id = toy.push(NodeKind::Literal, vec![], None);
            set_lit(toy, id, Lit::Bool(true))
        }
        "false" => {
            let id = toy.push(NodeKind::Literal, vec![], None);
            set_lit(toy, id, Lit::Bool(false))
        }
        "null" => {
            let id = toy.push(NodeKind::Literal, vec![], None);
            set_lit(toy, id, Lit::Null)
        }
        _ => {
            if let Some(num) = s.strip_suffix('L') {
                if let Ok(v) = num.parse::<i64>() {
                    let id = toy.push(NodeKind::Literal, vec![], None);
                    return set_lit(toy, id, Lit::Long(v));
                }
            }
            if let Ok(v) = s.parse::<i64>() {
                let id = toy.push(NodeKind::Literal, vec![], None);
                return set_lit(toy, id, Lit::Int(v));
            }
            if let Some(num) = s.strip_suffix('f') {
                if let Ok(v) = num.parse::<f64>() {
                    let id = toy.push(NodeKind::Literal, vec![], None);
                    return set_lit(toy, id, Lit::Float(v));
                }
            }
            if let Ok(v) = s.parse::<f64>() {
                let id = toy.push(NodeKind::Literal, vec![], None);
                return set_lit(toy, id, Lit::Double(v));
            }
            // 变量引用
            toy.push(NodeKind::VarRef, vec![], Some(s.to_string()))
        }
    }
}

// ---------------------------------------------------------------------------
// Toy 打印器（golden 输出格式）
// ---------------------------------------------------------------------------

fn to_sexp(toy: &Toy, id: Id) -> String {
    let n = &toy.nodes[id as usize];
    let ch: Vec<String> = n.children.iter().map(|&c| to_sexp(toy, c)).collect();
    let join = |sep: &str| ch.join(&format!("{sep} "));
    match n.kind {
        NodeKind::Block => {
            if ch.is_empty() {
                "(block)".into()
            } else {
                format!("(block {})", ch.join(" "))
            }
        }
        NodeKind::Empty => "(empty)".into(),
        NodeKind::ExprStmt => format!("(expr {})", ch[0]),
        NodeKind::VarDecl => match ch.len() {
            0 => format!("(decl {})", n.name.clone().unwrap()),
            _ => format!("(decl {} {})", n.name.clone().unwrap(), ch[0]),
        },
        NodeKind::Assign => format!("(assign {})", ch.join(" ")),
        NodeKind::If => match ch.len() {
            2 => format!("(if {} {})", ch[0], ch[1]),
            _ => format!("(if {} {} {})", ch[0], ch[1], ch[2]),
        },
        NodeKind::While => format!("(while {} {})", ch[0], ch[1]),
        NodeKind::Return => match ch.len() {
            0 => "(ret)".into(),
            _ => format!("(ret {})", ch[0]),
        },
        NodeKind::Break => "(break)".into(),
        NodeKind::Continue => "(continue)".into(),
        NodeKind::Throw => format!("(throw {})", ch[0]),
        NodeKind::Binary => {
            let op = BIN_OPS.iter().find(|(_, o)| Some(*o) == n.bin).unwrap().0;
            format!("({} {} {})", op, ch[0], ch[1])
        }
        NodeKind::Unary => {
            let op = UN_OPS.iter().find(|(_, o)| Some(*o) == n.un).unwrap().0;
            format!("({} {})", op, ch[0])
        }
        NodeKind::Call => format!("(call {})", join("")),
        NodeKind::Literal => match &n.lit {
            Some(Lit::Bool(true)) => "true".into(),
            Some(Lit::Bool(false)) => "false".into(),
            Some(Lit::Int(v)) => format!("{v}"),
            Some(Lit::Long(v)) => format!("{v}L"),
            Some(Lit::Float(v)) => format!("{}f", ensure_frac(*v)),
            Some(Lit::Double(v)) => ensure_frac(*v),
            Some(Lit::Str(s)) => format!("\"{s}\""),
            Some(Lit::Char(c)) => format!("'{c}'"),
            Some(Lit::Null) => "null".into(),
            None => "?".into(),
        },
        NodeKind::VarRef => n.name.clone().unwrap(),
        NodeKind::Ternary => format!("(ternary {})", ch.join(" ")),
        NodeKind::Paren => format!("(paren {})", ch[0]),
        NodeKind::Member => format!("(member {} {})", ch[0], n.name.clone().unwrap()),
        NodeKind::Index => format!("(index {} {})", ch[0], ch[1]),
        NodeKind::New => format!("(new {})", ch.join(" ")),
        NodeKind::ArrayLit => format!("(arr {})", ch.join(" ")),
        NodeKind::This => "(this)".into(),
        other => format!("(:{:?})", other),
    }
}

// ---------------------------------------------------------------------------
// 测试驱动
// ---------------------------------------------------------------------------

/// f64 的 Display 对 0.0/1.0 输出 "0"/"1"，golden 需要保留小数点。
fn ensure_frac(v: f64) -> String {
    let s = format!("{v}");
    if s.contains('.') || s.contains('e') || s.contains("inf") || s.contains("NaN") {
        s
    } else {
        format!("{s}.0")
    }
}

fn simplify_to(src: &str) -> String {
    let mut toy = parse(src);
    let root = toy.root;
    let rules = cure_engine::rules::default_rules::<Toy>();
    let report = cure_engine::simplify(&mut toy, root, &rules, &Config::default());
    format!("{}\n-- edits={} iters={}", to_sexp(&toy, root), report.edits, report.iterations)
}

fn run(src: &str) -> String {
    simplify_to(src)
        .lines()
        .next()
        .unwrap()
        .to_string()
}

#[test]
fn boolean_return_both_polarities() {
    assert_eq!(
        run("(block (if c (ret true) (ret false)))"),
        "(block (ret c))"
    );
    assert_eq!(
        run("(block (if c (ret false) (ret true)))"),
        "(block (ret (not c)))"
    );
}

#[test]
fn bool_compare() {
    assert_eq!(run("(block (ret (== x true)))"), "(block (ret x))");
    assert_eq!(run("(block (ret (== x false)))"), "(block (ret (not x)))");
    assert_eq!(run("(block (ret (== true x)))"), "(block (ret x))");
    assert_eq!(run("(block (ret (== false x)))"), "(block (ret (not x)))");
    assert_eq!(run("(block (ret (!= x true)))"), "(block (ret (not x)))");
    assert_eq!(run("(block (ret (!= x false)))"), "(block (ret x))");
    assert_eq!(run("(block (ret (!= true x)))"), "(block (ret (not x)))");
    assert_eq!(run("(block (ret (!= false x)))"), "(block (ret x))");
}

#[test]
fn double_not() {
    assert_eq!(run("(block (ret (not (not x))))"), "(block (ret x))");
}

#[test]
fn self_assign_removed() {
    assert_eq!(run("(block (assign x x) (expr (call foo)))"), "(block (expr (call foo)))");
    // 复合赋值 x += x 不删
    // （toy 未建复合赋值节点，见 java 侧测试）
}

#[test]
fn arith_identity() {
    assert_eq!(run("(block (ret (+ x 0)))"), "(block (ret x))");
    assert_eq!(run("(block (ret (+ 0 x)))"), "(block (ret x))");
    assert_eq!(run("(block (ret (- x 0)))"), "(block (ret x))");
    assert_eq!(run("(block (ret (* x 1)))"), "(block (ret x))");
    assert_eq!(run("(block (ret (* 1 x)))"), "(block (ret x))");
    assert_eq!(run("(block (ret (/ x 1)))"), "(block (ret x))");
    // 浮点守卫：x + 0.0 不化简（-0.0 / NaN 语义）
    assert_eq!(run("(block (ret (+ x 0.0)))"), "(block (ret (+ x 0.0)))");
    assert_eq!(run("(block (ret (/ x 1.0)))"), "(block (ret (/ x 1.0)))");
}

#[test]
fn const_condition() {
    assert_eq!(run("(block (if true (expr (call a)) (expr (call b))))"), "(block (expr (call a)))");
    // false → else 分支必然执行，保留之
    assert_eq!(
        run("(block (if false (expr (call a)) (expr (call b))))"),
        "(block (expr (call b)))"
    );
}

#[test]
fn if_else_empty() {
    assert_eq!(
        run("(block (if c (block) (block (expr (call a)))))"),
        "(block (if (not c) (block (expr (call a)))))"
    );
    assert_eq!(
        run("(block (if c (block (expr (call a))) (block)))"),
        "(block (if c (block (expr (call a)))))"
    );
    // cond 无副作用时可整体删除
    assert_eq!(run("(block (if x (block) ))"), "(block)");
    // cond 有副作用时保留
    assert_eq!(
        run("(block (if (call f) (block) ))"),
        "(block (if (call f) (block)))"
    );
}

#[test]
fn paren_removal() {
    assert_eq!(run("(block (ret (paren x)))"), "(block (ret x))");
}

#[test]
fn propagation_pure_values() {
    // 字面量传播
    assert_eq!(run("(block (decl x 5) (ret x))"), "(block (ret 5))");
    // 局部读传播
    assert_eq!(run("(block (decl x y) (ret x))"), "(block (ret y))");
    // 区间内无关语句（只读）不阻断（语句本身保留，不在传播范围内）
    assert_eq!(
        run("(block (decl x 5) (expr y) (ret x))"),
        "(block (expr y) (ret 5))"
    );
    // 区间内有对 value 读到的变量的写 → 放弃传播；
    // 但根块内的死写（assign y 2 无后续读）先被零用途 DeadStore 删除，
    // 删除后传播 y 健全（y 保持旧值，ret y ≡ ret x，行为等价）
    assert_eq!(
        run("(block (decl x y) (assign y 2) (ret x))"),
        "(block (ret y))"
    );
    // 多次使用 → 放弃
    assert_eq!(
        run("(block (decl x 5) (expr (+ x 1)) (ret x))"),
        "(block (decl x 5) (expr (+ x 1)) (ret x))"
    );
    // 区间内重声明：旧声明被遮蔽不传播；新声明本身照常传播；
    // 内层传播后旧声明的读归零 → 零用途 DeadStore 删除（值从未流出，健全）
    assert_eq!(
        run("(block (decl x 5) (expr (call f)) (decl x 6) (ret x))"),
        "(block (expr (call f)) (ret 6))"
    );
}

#[test]
fn propagation_impure_adjacent() {
    // 相邻 + 直线 return：int a = foo(); return a; → return foo();
    assert_eq!(
        run("(block (decl x (call foo)) (ret x))"),
        "(block (ret (call foo)))"
    );
    // 相邻 + 赋值语句：int a = foo(); y = a; → y = foo();
    assert_eq!(
        run("(block (decl x (call foo)) (assign y x))"),
        "(block (assign y (call foo)))"
    );
    // 相邻但语句内有先求值的非只读子表达式：return bar() + a; → 不动
    assert_eq!(
        run("(block (decl x (call foo)) (ret (+ (call bar) x)))"),
        "(block (decl x (call foo)) (ret (+ (call bar) x)))"
    );
    // 相邻但先求值的是只读的 y：return y + a; → 可行
    assert_eq!(
        run("(block (decl x (call foo)) (ret (+ y x)))"),
        "(block (ret (+ y (call foo))))"
    );
    // 相邻但不是直线语句（if 条件中用）→ 声明不内联；
    // if 本身被 IfToTernary 归并成三元（副作用声明仍然保留——传播被正确阻断）
    assert_eq!(
        run("(block (decl x (call foo)) (if x (ret 1) (ret 2)))"),
        "(block (decl x (call foo)) (ret (ternary x 1 2)))"
    );
    // 不相邻的有副作用值 → 不动
    assert_eq!(
        run("(block (decl x (call foo)) (expr (call bar)) (ret x))"),
        "(block (decl x (call foo)) (expr (call bar)) (ret x))"
    );
}

#[test]
fn doc_example_chain() {
    // 设计文档示例 1：
    // int a = foo(); int b = a; return b;  →  return foo();
    assert_eq!(
        run("(block (decl a (call foo)) (decl b a) (ret b))"),
        "(block (ret (call foo)))"
    );
}

#[test]
fn doc_example_boolean_chain() {
    // 设计文档示例 2：
    // boolean b = x > 10; if (b) return true; else return false;  →  return x > 10;
    assert_eq!(
        run("(block (decl b (> x 10)) (if b (ret true) (ret false)))"),
        "(block (ret (> x 10)))"
    );
    // 设计文档示例 3：
    // if (c) return false; else return true;  →  return !c;
    assert_eq!(
        run("(block (if c (ret false) (ret true)))"),
        "(block (ret (not c)))"
    );
}

#[test]
fn report_and_disable() {
    let mut toy = parse("(block (decl x 5) (ret x))");
    let root = toy.root;
    let rules = cure_engine::rules::default_rules::<Toy>();
    let mut cfg = Config::default();
    cfg.disabled_rules.insert("local_propagation".into());
    let report = cure_engine::simplify(&mut toy, root, &rules, &cfg);
    assert_eq!(report.edits, 0);
    assert_eq!(to_sexp(&toy, root), "(block (decl x 5) (ret x))");
}

#[test]
fn fixed_point_multirule() {
    // boolean_return → local_propagation 的链式触发
    let out = run("(block (decl b (> x 10)) (if b (ret true) (ret false)))");
    assert_eq!(out, "(block (ret (> x 10)))");
    let full = simplify_to("(block (decl b (> x 10)) (if b (ret true) (ret false)))");
    assert!(full.contains("edits=2"), "got: {full}");
}

// 直接用 Edit/Rule API 的白盒小测试，保证 trait 对外可用
#[test]
fn rule_trait_is_public_and_usable() {
    struct Noop;
    impl Rule<Toy> for Noop {
        fn name(&self) -> &'static str {
            "noop"
        }
        fn check(&self, _ctx: RewriteCtx<'_, Toy>, _id: Id) -> Option<Edit<Toy>> {
            None
        }
    }
    let mut toy = parse("(block)");
    let root = toy.root;
    let rules: Vec<Box<dyn Rule<Toy>>> = vec![Box::new(Noop)];
    let r = cure_engine::simplify(&mut toy, root, &rules, &Config::default());
    assert_eq!(r.edits, 0);
    let _unused: HashSet<&str> = HashSet::new();
}
