//! 词法作用域/绑定解析（prepare 阶段）。
//!
//! 目标：把“名字字符串 + 一堆作用域启发式”换成每个引用的**绑定身份**。
//! 产出的 `Binding` 事实供局部性/捕获/写杀/var 提升陷阱使用，也为后续
//! 规则（常量传播、内联、重命名）提供稳定入口。
//!
//! 约定：
//! - 顶层（ModuleBlock 根）绑定标记 `module_level`，规则不得当局部变量；
//! - `var` 绑定到最近的函数作用域，`let/const/using` 绑定到当前块；
//! - 函数声明名字同时在外部作用域与自身函数作用域可见（自引用不误判
//!   为捕获）；函数表达式的名字只在自己函数作用域内；
//! - 模式参数/解构声明名（原文）暂不绑定——引用落到“未解析”，规则
//!   沿用保守路径。

use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScopeMode {
    Program,
    Function,
    Child,
}

struct Scope {
    parent: Option<u32>,
    /// 最近的函数/程序作用域（`var` 的归属）。
    func: u32,
    binds: HashMap<String, u32>,
}

struct State {
    scopes: Vec<Scope>,
    bindings: Vec<Binding>,
    func_of_binding: Vec<u32>,
    node_scope: Vec<u32>,
    node_func: Vec<u32>,
    parents: Vec<(TsId, usize)>,
    visited: Vec<TsId>,
    program: u32,
    is_module: bool,
    /// 声明节点 → BindingId（pass 2 合并进 TsAst::binding_of）。
    decl_binding: Vec<Option<u32>>,
}

enum Frame {
    Enter(TsId),
    Pop,
}

impl State {
    fn new(n: usize, is_module: bool) -> Self {
        State {
            scopes: Vec::new(),
            bindings: Vec::new(),
            func_of_binding: Vec::new(),
            node_scope: vec![0; n],
            node_func: vec![0; n],
            parents: vec![(TsId(0), usize::MAX); n],
            visited: Vec::new(),
            program: 0,
            is_module,
            decl_binding: vec![None; n],
        }
    }

    fn new_scope(&mut self, parent: Option<u32>, mode: ScopeMode) -> u32 {
        let id = self.scopes.len() as u32;
        let func = match mode {
            ScopeMode::Program | ScopeMode::Function => id,
            ScopeMode::Child => parent.map_or(id, |p| self.scopes[p as usize].func),
        };
        self.scopes.push(Scope { parent, func, binds: HashMap::new() });
        id
    }

    fn bind(
        &mut self,
        scope: u32,
        name: &str,
        kind: BindingKind,
        decl: Option<TsId>,
    ) -> u32 {
        if let Some(&bid) = self.scopes[scope as usize].binds.get(name) {
            if self.bindings[bid as usize].decl.is_none() {
                self.bindings[bid as usize].decl = decl;
            }
            return bid;
        }
        let bid = self.bindings.len() as u32;
        let module_level = self.is_module && scope == self.program;
        self.bindings.push(Binding {
            name: name.to_string(),
            kind,
            decl,
            ref_count: 0,
            write_count: 0,
            captured: false,
            module_level,
        });
        self.func_of_binding.push(self.scopes[scope as usize].func);
        self.scopes[scope as usize].binds.insert(name.to_string(), bid);
        bid
    }

    fn lookup(&self, mut scope: u32, name: &str) -> Option<u32> {
        loop {
            if let Some(&bid) = self.scopes[scope as usize].binds.get(name) {
                return Some(bid);
            }
            scope = self.scopes[scope as usize].parent?;
        }
    }
}

pub(crate) fn rebuild(ast: &mut TsAst, root: TsId) {
    let n = ast.nodes.len();
    if n == 0 {
        return;
    }
    let is_module = matches!(ast.nodes[root.0 as usize].data, NodeData::ModuleBlock);
    let mut st = State::new(n, is_module);
    let mode = if is_module { ScopeMode::Program } else { ScopeMode::Function };
    let program = st.new_scope(None, mode);
    st.program = program;
    for p in &ast.param_scope {
        st.bind(program, p, BindingKind::Param, None);
    }

    // ---- pass 1：建作用域、收集声明 ----
    let mut cur = program;
    let mut stack = vec![Frame::Enter(root)];
    while let Some(f) = stack.pop() {
        match f {
            Frame::Pop => {
                if let Some(p) = st.scopes[cur as usize].parent {
                    cur = p;
                }
            }
            Frame::Enter(id) => {
                let outer = cur;
                let mut node_scope = outer;
                let mut opened = false;
                match &ast.nodes[id.0 as usize].data {
                    NodeData::ModuleBlock => {
                        if id == root {
                            node_scope = program;
                        } else {
                            let s = st.new_scope(Some(outer), ScopeMode::Child);
                            cur = s;
                            node_scope = s;
                            opened = true;
                        }
                    }
                    NodeData::Block => {
                        let s = st.new_scope(Some(outer), ScopeMode::Child);
                        cur = s;
                        node_scope = s;
                        opened = true;
                    }
                    NodeData::For { .. } => {
                        let s = st.new_scope(Some(outer), ScopeMode::Child);
                        cur = s;
                        node_scope = s;
                        opened = true;
                    }
                    NodeData::ForEach { name, decl, .. } => {
                        match decl {
                            Some(VarKind::Var) => {
                                let target = st.scopes[outer as usize].func;
                                let bid = st.bind(target, name, BindingKind::Var, Some(id));
                                st.decl_binding[id.0 as usize] = Some(bid);
                            }
                            Some(k) => {
                                let kind = match k {
                                    VarKind::Let => BindingKind::Let,
                                    VarKind::Const => BindingKind::Const,
                                    VarKind::Using => BindingKind::Using,
                                    VarKind::Var => BindingKind::Var,
                                };
                                let s = st.new_scope(Some(outer), ScopeMode::Child);
                                let bid = st.bind(s, name, kind, Some(id));
                                st.decl_binding[id.0 as usize] = Some(bid);
                                cur = s;
                                node_scope = s;
                                opened = true;
                            }
                            None => {}
                        }
                    }
                    NodeData::Catch { param_raw } => {
                        let s = st.new_scope(Some(outer), ScopeMode::Child);
                        if let Some(nm) = param_raw.as_deref().and_then(simple_binding_name) {
                            let bid = st.bind(s, &nm, BindingKind::CatchParam, Some(id));
                            st.decl_binding[id.0 as usize] = Some(bid);
                        }
                        cur = s;
                        node_scope = s;
                        opened = true;
                    }
                    NodeData::Switch => {
                        let s = st.new_scope(Some(outer), ScopeMode::Child);
                        cur = s;
                        node_scope = s;
                        opened = true;
                    }
                    NodeData::VarDecl { name, kind, .. } => {
                        let target = if *kind == VarKind::Var {
                            st.scopes[outer as usize].func
                        } else {
                            outer
                        };
                        let k = match kind {
                            VarKind::Var => BindingKind::Var,
                            VarKind::Let => BindingKind::Let,
                            VarKind::Const => BindingKind::Const,
                            VarKind::Using => BindingKind::Using,
                        };
                        let bid = st.bind(target, name, k, Some(id));
                        st.decl_binding[id.0 as usize] = Some(bid);
                    }
                    NodeData::FuncStmt { params_raw, name, .. } => {
                        if !name.is_empty() {
                            let bid = st.bind(outer, name, BindingKind::Function, Some(id));
                            st.decl_binding[id.0 as usize] = Some(bid);
                        }
                        let fs = st.new_scope(Some(outer), ScopeMode::Function);
                        // 函数体内的自引用绑定（否则递归名会被误判为
                        // 捕获外部绑定）。
                        if !name.is_empty() {
                            st.bind(fs, name, BindingKind::Function, Some(id));
                        }
                        if let Some(ps) = simple_params(params_raw) {
                            for p in ps {
                                st.bind(fs, &p, BindingKind::Param, Some(id));
                            }
                        }
                        cur = fs;
                        opened = true;
                    }
                    NodeData::Lambda { params_raw, name, .. } => {
                        let fs = st.new_scope(Some(outer), ScopeMode::Function);
                        if !name.is_empty() {
                            let bid = st.bind(fs, name, BindingKind::FunctionExprName, Some(id));
                            st.decl_binding[id.0 as usize] = Some(bid);
                        }
                        if let Some(ps) = simple_params(params_raw) {
                            for p in ps {
                                st.bind(fs, &p, BindingKind::Param, Some(id));
                            }
                        }
                        cur = fs;
                        opened = true;
                    }
                    _ => {}
                }
                st.node_scope[id.0 as usize] = node_scope;
                st.node_func[id.0 as usize] = st.scopes[node_scope as usize].func;
                st.visited.push(id);
                if opened {
                    stack.push(Frame::Pop);
                }
                let ch = ast.nodes[id.0 as usize].children.as_slice();
                for (i, &c) in ch.iter().enumerate().rev() {
                    st.parents[c.0 as usize] = (id, i);
                    stack.push(Frame::Enter(c));
                }
            }
        }
    }

    // ---- pass 2：解析引用、统计读写/捕获 ----
    let mut local = vec![false; n];
    let mut captured: HashSet<String> = HashSet::new();
    let mut mutated: HashSet<String> = HashSet::new();
    let mut refs: Vec<Vec<TsId>> = vec![Vec::new(); st.bindings.len()];
    // 声明节点的绑定映射在 pass 1 已写入 decl_binding；引用解析只补
    // VarRef（声明节点保留原映射）。
    let mut binding_of = std::mem::take(&mut st.decl_binding);

    for &id in &st.visited {
        match &ast.nodes[id.0 as usize].data {
            NodeData::VarRef { name } => {
                let nm = ast.sn(*name).to_string();
                let Some(bid) = st.lookup(st.node_scope[id.0 as usize], &nm) else {
                    continue;
                };
                let write = is_write_target(ast, &st.parents, id);
                let captured_here =
                    st.node_func[id.0 as usize] != st.func_of_binding[bid as usize];
                {
                    let b = &mut st.bindings[bid as usize];
                    b.ref_count += 1;
                    if write {
                        b.write_count += 1;
                    }
                    if captured_here {
                        b.captured = true;
                    }
                    if !b.module_level {
                        local[id.0 as usize] = true;
                    }
                }
                if write {
                    mutated.insert(nm.clone());
                }
                if captured_here {
                    captured.insert(nm);
                }
                binding_of[id.0 as usize] = Some(bid);
                refs[bid as usize].push(id);
            }
            NodeData::ForEach { name, decl: None, .. } => {
                if let Some(bid) = st.lookup(st.node_scope[id.0 as usize], name) {
                    let b = &mut st.bindings[bid as usize];
                    b.ref_count += 1;
                    b.write_count += 1;
                    mutated.insert(name.clone());
                    refs[bid as usize].push(id);
                }
            }
            // 解构模式的绑定名不可见：写目标按“可能变异”保守登记
            NodeData::Destruct { pattern_raw } => {
                collect_pattern_words(pattern_raw, &mut mutated);
            }
            NodeData::DestructStmt { pattern, .. } => {
                collect_pattern_words(pattern, &mut mutated);
            }
            _ => {}
        }
    }
    // ---- 源码保真区（preserve_function_source）----
    // 函数/对象字面量将逐字节原样打印；任何内部改写都会被打印层丢弃，
    // 但**基于内部引用的外部改写**（把绑定值传播进字段/删除声明）不会
    // 被丢弃——打印原文后会变成悬挂引用。这里把保真区内的节点及其
    // 引用的绑定声明全部标为不透明，规则既不进入也不据此删除。
    let preserved = if ast.preserve_source() {
        let mut in_zone = vec![false; n];
        // 从根向下传播：命中保真根后其整棵子树都在保真区。
        let mut zone_root = vec![false; n];
        for &id in &st.visited {
            let d = &ast.nodes[id.0 as usize].data;
            if matches!(
                d,
                NodeData::Lambda { .. } | NodeData::ObjLit { .. } | NodeData::FuncStmt { .. }
            ) {
                zone_root[id.0 as usize] = true;
            }
        }
        // 迭代式父->子传播（visited 顺序不保证父先于子）
        let mut order: Vec<TsId> = Vec::new();
        let mut seen = vec![false; n];
        let mut q = vec![root];
        while let Some(x) = q.pop() {
            if seen[x.0 as usize] {
                continue;
            }
            seen[x.0 as usize] = true;
            order.push(x);
            for c in ast.nodes[x.0 as usize].children.as_slice() {
                q.push(*c);
            }
        }
        for &x in &order {
            if zone_root[x.0 as usize] {
                in_zone[x.0 as usize] = true;
            }
            if in_zone[x.0 as usize] {
                for c in ast.nodes[x.0 as usize].children.as_slice() {
                    in_zone[c.0 as usize] = true;
                }
            }
        }
        in_zone
    } else {
        Vec::new()
    };
    let preserved_zone = !preserved.is_empty();

    // ---- var 提升陷阱（按绑定引用，而不是全名出现）----
    let mut var_trap = vec![false; n];
    for &id in &st.visited {
        let NodeData::VarDecl { kind, .. } = &ast.nodes[id.0 as usize].data else {
            continue;
        };
        let Some(bid) = binding_of[id.0 as usize] else {
            var_trap[id.0 as usize] = true;
            continue;
        };
        // 绑定在保真区内被引用 → 声明不得被传播/删除（打印原文会
        // 重新引入该引用）。
        if preserved_zone
            && refs[bid as usize]
                .iter()
                .any(|&r| preserved.get(r.0 as usize).copied().unwrap_or(false))
        {
            var_trap[id.0 as usize] = true;
            continue;
        }
        // 顶层声明（含 let/const）在模块合成根里看不见别的顶层项
        //（class/函数体/另一个根）里的读写——声明级规则必须拒绝触碰。
        // 旧实现用 module root var_trap 覆盖；resolver 统一按绑定判定。
        if st.bindings[bid as usize].module_level {
            var_trap[id.0 as usize] = true;
            continue;
        }
        if *kind != VarKind::Var {
            continue;
        }
        let Some((block, dpos)) = ancestor_block_pos(&ast.nodes, &st.parents, id) else {
            var_trap[id.0 as usize] = true;
            continue;
        };
        let mut trap = false;
        for &r in &refs[bid as usize] {
            if !inside_later_sibling(&st.parents, r, block, dpos) {
                trap = true;
                break;
            }
        }
        var_trap[id.0 as usize] = trap;
    }

    if preserved_zone {
        // 保真区内部节点也不接受规则改写（打印层反正会丢弃内部改写，
        // 但避免基于改写结果的跨节点提案）。
        for (i, &z) in preserved.iter().enumerate() {
            if z {
                var_trap[i] = true;
            }
        }
    }
    ast.bindings = st.bindings;
    ast.binding_of = binding_of;
    ast.local_marks = local;
    ast.captured_names = captured;
    ast.mutated_names = mutated;
    ast.var_trap = var_trap;
}

fn collect_pattern_words(pattern: &str, out: &mut HashSet<String>) {
    for w in pattern.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$')) {
        if !w.is_empty() && !w.starts_with(|c: char| c.is_ascii_digit()) {
            out.insert(w.to_string());
        }
    }
}

fn is_write_target(ast: &TsAst, parents: &[(TsId, usize)], id: TsId) -> bool {
    let mut n = id;
    loop {
        let (p, i) = parents[n.0 as usize];
        if i == usize::MAX {
            return false;
        }
        match &ast.nodes[p.0 as usize].data {
            NodeData::Paren if i == 0 => n = p,
            NodeData::Assign { .. } if i == 0 => return true,
            NodeData::Unary { op } if i == 0 && op.is_incdec() => return true,
            _ => return false,
        }
    }
}
