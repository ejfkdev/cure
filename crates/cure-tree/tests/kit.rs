//! cure-tree 独立组装验证：用一个 ~100 行的玩具语言证明「新语言前端」
//! 只需 NodeData + Lang 钩子即可复用全部树基建（ChildList / NameTable /
//! EventStore / 事件收集器 / 效果表重建）。

use cure_engine::kind::{BinOp, LitRef, NodeKind, RegionEvent, UnOp};
use cure_engine::{Effect, Lang};
use cure_tree::{rebuild_effects, ChildList, EventStore, NameTable};

type ToyId = u32;

#[derive(Clone, Debug)]
struct ToyNode {
    kind: ToyKind,
    name: Option<String>,
    children: ChildList<ToyId>,
}

/// 玩具语法：足够覆盖事件收集器的全部分支。
#[derive(Clone, Copy, Debug, PartialEq)]
enum ToyKind {
    Block,
    VarDecl,
    VarRef,
    Assign,
    Inc,
    ForEach,
    Catch,
    Raw, // 不可解析原文（不透明）
    Lit,
}

/// 符合 cure-tree 约定的最小 Lang：稠密 arena + u32 intern 名字键。
struct Toy {
    nodes: Vec<ToyNode>,
    names: NameTable,
    events: EventStore<ToyId, u32>,
    effects: Vec<Option<Effect>>,
}

impl Toy {
    fn new() -> Self {
        Toy {
            nodes: Vec::new(),
            names: NameTable::new(),
            events: EventStore::new(),
            effects: Vec::new(),
        }
    }
    fn push(&mut self, kind: ToyKind, name: Option<&str>, children: Vec<ToyId>) -> ToyId {
        let id = self.nodes.len() as ToyId;
        self.nodes.push(ToyNode {
            kind,
            name: name.map(|s| s.to_string()),
            children: ChildList::from_vec(children),
        });
        id
    }
    fn k(&self, id: ToyId) -> ToyKind {
        self.nodes[id as usize].kind
    }
}

impl Lang for Toy {
    type Id = ToyId;
    type NameKey = u32;

    fn node_index(&self, id: ToyId) -> usize {
        id as usize
    }
    fn id_of_index(&self, idx: usize) -> ToyId {
        idx as u32
    }
    fn kind(&self, id: ToyId) -> NodeKind {
        match self.k(id) {
            ToyKind::Block => NodeKind::Block,
            ToyKind::VarDecl => NodeKind::VarDecl,
            ToyKind::VarRef => NodeKind::VarRef,
            ToyKind::Assign => NodeKind::Assign,
            ToyKind::Inc => NodeKind::Unary,
            ToyKind::ForEach => NodeKind::ForEach,
            ToyKind::Catch => NodeKind::Catch,
            ToyKind::Raw => NodeKind::Raw,
            ToyKind::Lit => NodeKind::Literal,
        }
    }
    fn children(&self, id: ToyId) -> &[ToyId] {
        self.nodes[id as usize].children.as_slice()
    }
    fn set_child(&mut self, parent: ToyId, index: usize, new: ToyId) {
        self.nodes[parent as usize].children.set(index, new);
    }
    fn remove_child(&mut self, parent: ToyId, index: usize) {
        self.nodes[parent as usize].children.remove(index);
    }
    fn splice(&mut self, node: ToyId, index: usize, remove: usize, insert: Vec<ToyId>) {
        self.nodes[node as usize].children.splice(index, remove, insert);
    }
    fn build_return(&mut self, _v: Option<ToyId>) -> ToyId {
        unreachable!()
    }
    fn build_block(&mut self, stmts: Vec<ToyId>) -> ToyId {
        self.push(ToyKind::Block, None, stmts)
    }
    fn build_unary(&mut self, _op: UnOp, _operand: ToyId) -> ToyId {
        unreachable!()
    }
    fn build_bool(&mut self, _v: bool) -> ToyId {
        self.push(ToyKind::Lit, None, vec![])
    }
    fn build_int(&mut self, _v: i64, _wide: bool) -> ToyId {
        self.push(ToyKind::Lit, None, vec![])
    }
    fn build_bin(&mut self, _op: BinOp, _l: ToyId, _r: ToyId) -> ToyId {
        unreachable!()
    }
    fn build_ternary(&mut self, _c: ToyId, _a: ToyId, _b: ToyId) -> ToyId {
        unreachable!()
    }
    fn build_assign(&mut self, target: ToyId, value: ToyId) -> ToyId {
        self.push(ToyKind::Assign, None, vec![target, value])
    }
    fn build_str(&mut self, _s: &str) -> ToyId {
        self.push(ToyKind::Lit, None, vec![])
    }
    fn build_char(&mut self, _c: char) -> ToyId {
        self.push(ToyKind::Lit, None, vec![])
    }
    fn copy_subtree(&mut self, _id: ToyId) -> ToyId {
        unreachable!()
    }
    fn effect(&self, id: ToyId) -> Effect {
        self.effects[id as usize].unwrap_or(Effect::Unknown)
    }
    fn invalidate_effect(&mut self, id: ToyId) {
        if let Some(slot) = self.effects.get_mut(id as usize) {
            *slot = None;
        }
        self.events.invalidate(id as usize);
    }
    fn region_events(&self, id: ToyId) -> Option<&[RegionEvent<ToyId, u32>]> {
        let v = self.events.slot(id as usize);
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    }
    fn var_key(&self, id: ToyId) -> Option<u32> {
        self.names.var_key(id as usize)
    }
    fn is_opaque(&self, id: ToyId) -> bool {
        self.k(id) == ToyKind::Raw
    }
    fn own_effect(&self, id: ToyId) -> Effect {
        match self.k(id) {
            ToyKind::Lit | ToyKind::Block | ToyKind::VarDecl => Effect::Pure,
            ToyKind::VarRef => Effect::MayRead,
            ToyKind::Assign | ToyKind::Inc => Effect::MayWrite,
            ToyKind::Raw => Effect::Unknown,
            _ => Effect::Pure,
        }
    }
    fn is_bool(&self, _id: ToyId) -> bool {
        false
    }
    fn is_exact_int(&self, _id: ToyId) -> bool {
        false
    }
    fn bin_op(&self, _id: ToyId) -> Option<BinOp> {
        None
    }
    fn un_op(&self, id: ToyId) -> Option<UnOp> {
        // Inc 节点伪装成自增（事件收集器要求）
        if self.k(id) == ToyKind::Inc {
            Some(UnOp::PreInc)
        } else {
            None
        }
    }
    fn literal(&self, _id: ToyId) -> Option<LitRef<'_>> {
        None
    }
    fn var_name(&self, id: ToyId) -> Option<&str> {
        self.nodes[id as usize].name.as_deref()
    }

    fn prepare(&mut self, root: ToyId) {
        // 标准骨架（与 crate 文档一致）
        let n = self.nodes.len();
        let slots = self.names.key_slots();
        if slots < n {
            let names: Vec<Option<String>> = (slots..n)
                .map(|i| self.var_name(i as u32).map(|s| s.to_string()))
                .collect();
            self.names.extend_owned(slots, &names);
        }
        self.events.resize(n);
        if !self.events.is_clean() {
            self.events.mark_clean();
            for sr in cure_tree::stmt_roots(self, root) {
                let idx = sr as usize;
                if self.events.slot(idx).is_empty() {
                    let ev = cure_tree::collect_region_events(self, sr);
                    self.events.put(idx, ev);
                }
            }
        }
        let effects = rebuild_effects(self, n);
        self.effects = effects;
    }
}

// 构造：{ int x = 1; x = foo(); y++; for (T s : list) use(s); raw(); }
fn sample() -> (Toy, ToyId) {
    let mut t = Toy::new();
    let x_decl = t.push(ToyKind::VarDecl, Some("x"), vec![]); // 声明 x
    let x_ref = t.push(ToyKind::VarRef, Some("x"), vec![]);
    let foo = t.push(ToyKind::Lit, None, vec![]);
    let assign = t.push(ToyKind::Assign, None, vec![x_ref, foo]); // x = foo()
    let y_ref = t.push(ToyKind::VarRef, Some("y"), vec![]);
    let inc = t.push(ToyKind::Inc, None, vec![y_ref]); // y++
    let s_decl = t.push(ToyKind::ForEach, Some("s"), vec![]); // for (s : …)
    let s_ref = t.push(ToyKind::VarRef, Some("s"), vec![]);
    let use_stmt = t.push(ToyKind::Assign, None, vec![s_ref, foo]); // use(s) 的替身
    let _ = use_stmt;
    let raw = t.push(ToyKind::Raw, None, vec![]);
    let block = t.push(
        ToyKind::Block,
        None,
        vec![x_decl, assign, inc, s_decl, raw],
    );
    (t, block)
}

#[test]
fn toy_events_match_semantics() {
    let (mut t, root) = sample();
    t.prepare(root);

    // 语句根 = Block 直系孩子
    let roots = cure_tree::stmt_roots(&t, root);
    assert_eq!(roots.len(), 5);

    // x 声明语句：Write(x) + Shadow(x)
    let x_key = t.names.intern("x");
    let ev = t.region_events(roots[0]).unwrap();
    assert_eq!(ev.len(), 2);
    assert_eq!(ev[0].key, x_key);
    assert!(matches!(ev[0].kind, cure_engine::kind::EventKind::Write));
    assert!(matches!(ev[1].kind, cure_engine::kind::EventKind::Shadow));

    // x = foo()：Write(x)（事件节点 = target VarRef）
    let ev = t.region_events(roots[1]).unwrap();
    assert_eq!(ev.len(), 1);
    assert!(matches!(ev[0].kind, cure_engine::kind::EventKind::Write));

    // y++：Write(y)
    let y_key = t.names.intern("y");
    let ev = t.region_events(roots[2]).unwrap();
    assert_eq!(ev.len(), 1);
    assert_eq!(ev[0].key, y_key);
    assert!(matches!(ev[0].kind, cure_engine::kind::EventKind::Write));

    // for (s : …)：Write(s) + Shadow(s)
    let ev = t.region_events(roots[3]).unwrap();
    assert_eq!(ev.len(), 2);

    // raw()：不透明 → 不索引（空槽）
    assert!(t.region_events(roots[4]).is_none());
}

#[test]
fn toy_effect_rebuild() {
    let (mut t, root) = sample();
    t.prepare(root);
    // Block 聚合 = 子树最坏效果（含 Raw → Unknown）
    assert_eq!(t.effect(root), Effect::Unknown);
    // 纯声明 = Pure
    let roots = cure_tree::stmt_roots(&t, root);
    assert_eq!(t.effect(roots[0]), Effect::Pure);
    // 赋值 = MayWrite
    assert_eq!(t.effect(roots[1]), Effect::MayWrite);
}

#[test]
fn toy_invalidate_and_lazy_rebuild() {
    let (mut t, root) = sample();
    t.prepare(root);
    // 失效一条语句 → B3：只重建空条目，其余保留
    let roots = cure_tree::stmt_roots(&t, root);
    let before = t.region_events(roots[1]).unwrap().len();
    t.invalidate_effect(roots[1]);
    assert!(t.region_events(roots[1]).is_none());
    t.prepare(root);
    let after = t.region_events(roots[1]).unwrap().len();
    assert_eq!(before, after);
}

#[test]
fn child_list_inline_and_spill() {
    let mut cl: ChildList<u32> = ChildList::new();
    assert!(cl.is_empty());
    cl = ChildList::from_vec(vec![1, 2, 3]);
    assert_eq!(cl.as_slice(), &[1, 2, 3]); // 内联
    cl = ChildList::from_vec(vec![1, 2, 3, 4, 5]); // 溢出堆
    assert_eq!(cl.as_slice(), &[1, 2, 3, 4, 5]);
    cl.remove(4); // 缩回 ≤3 → 回内联
    assert_eq!(cl.as_slice(), &[1, 2, 3, 4]);
    cl.remove(3);
    assert_eq!(cl.as_slice(), &[1, 2, 3]);
    cl.splice(1, 1, vec![9, 9]);
    assert_eq!(cl.as_slice(), &[1, 9, 9, 3]);
}

#[test]
fn name_table_intern_and_keys() {
    let mut nt = NameTable::new();
    let a = nt.intern("foo");
    let b = nt.intern("foo");
    let c = nt.intern("bar");
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(nt.name(a), "foo");
    assert_eq!(nt.name(c), "bar");
    nt.extend_owned(0, &[Some("x".into()), None, Some("y".into())]);
    assert_eq!(nt.var_key(0), Some(a.max(c) + 1)); // 新键稠密递增
    assert_eq!(nt.var_key(1), None);
    assert!(nt.var_key(2).is_some());
}
