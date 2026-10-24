//! # cure-ts-simplify
//!
//! TS/JS 简化门面：收集模块全部可执行体（顶层语句/函数体/类方法/
//! 静态块/字段初始化器），对每个根跑 [`cure_engine`] 的不动点规则
//! 引擎（默认规则集 + TS 专属规则）。
//!
//! JS 语义安全由 [`cure_ts_ast`] 的 Lang 实现承担：
//! - 数值折叠走 f64/ToInt32 域（±2^53 精确性回验；-0/NaN/±Inf 拒折）
//! - `var` 提升陷阱（块外引用 → is_opaque）
//! - 可选链元素不透明（链短路语义）
//! - 解构模式不透明（绑定名不可见）
//! - is_bool 仅语法级（比较/逻辑/!）——真值不确定性由规则守卫覆盖


use cure_engine::kind::{LitRef, NodeKind, UnOp};
use cure_engine::{Config, Edit, Effect, Lang, Report, RewriteCtx, Rule};
use cure_ts_ast::JsLit;
mod rules;
pub use rules::ported_rules;

use std::collections::HashSet;

use cure_ts_ast::{
    simple_params, ClassDecl, ClassMember, ModuleUnit, NodeData, TopItem, TsAst, TsId,
};

/// 单根简化（引擎默认规则 + TS 专属规则）。
pub fn simplify(ast: &mut TsAst, root: TsId, cfg: &Config) -> Report {
    let rules = default_ts_rules();
    cure_engine::simplify(ast, root, &rules, cfg)
}

/// 整个模块：逐体优化，签名/类型层不动。
pub fn simplify_unit(ast: &mut TsAst, unit: &mut ModuleUnit, cfg: &Config) -> Report {
    let mut total = Report::default();
    for _cycle in 0..6 {
        let mut round = Report::default();
        collect_unit_consts(ast, unit);
        // ---- 模块语句合成单根 ----
        // 顶层普通语句（非 Func/Class/Import/Raw/ExportStmt）拼进
        // ModuleBlock：引擎获得完整父链（if-true 解包、跨语句事件
        // 分析）。重分发按位进行：孩子多出 → 追加 Stmt 项；不足 →
        // 多余项删除（全 Stmt 包装同质——位位移无害）。
        let stmt_positions: Vec<usize> = unit
            .items
            .iter()
            .enumerate()
            .filter(|(_, it)| matches!(it, TopItem::Stmt(_)))
            .map(|(i, _)| i)
            .collect();
        if !stmt_positions.is_empty() {
            // Group 保持单节点：拆开会让「位置 → 声明」重分发错位
            //（`let a, b;` 的第二个声明被插到文件末尾，静态块在 TDZ
            // 中读到 probe2——Test262 class static-init-scope 抓获）。
            // 模块级不透明性由 prepare 的 module_decls（含 Group 孩子）
            // 与 group_decls 保障。
            let mut stmts: Vec<TsId> = Vec::new();
            for &i in &stmt_positions {
                match unit.items[i] {
                    TopItem::Stmt(s) => stmts.push(s),
                    _ => unreachable!(),
                }
            }
            let root = ast.module_block(stmts);
            let r = simplify(ast, root, cfg);
            merge_report(&mut round, r);
            let new_children = ast.children(root).to_vec();
            let m = new_children.len();
            let n = stmt_positions.len();
            for (k, &pos) in stmt_positions.iter().enumerate() {
                if k < m {
                    unit.items[pos] = TopItem::Stmt(new_children[k]);
                }
            }
            if m > n {
                let extra: Vec<TopItem> =
                    new_children[n..].iter().map(|&c| TopItem::Stmt(c)).collect();
                let insert_at = stmt_positions[n - 1] + 1;
                for (j, e) in extra.into_iter().enumerate() {
                    unit.items.insert(insert_at + j, e);
                }
            } else if m < n {
                // 尾部 m..n 位被删除（从后往前 remove 保持下标稳定）
                for &pos in stmt_positions[m..].iter().rev() {
                    unit.items.remove(pos);
                }
            }
        }
        // ---- 其余项（函数/类/导出声明）逐根 ----
        for item in &mut unit.items {
            if matches!(item, TopItem::Stmt(_)) {
                continue; // 已在合成根跑过
            }
            simplify_top_item(ast, item, cfg, &mut round);
        }
        total.edits += round.edits;
        total.iterations += round.iterations;
        total.discarded += round.discarded;
        for (k, v) in round.by_rule {
            *total.by_rule.entry(k).or_insert(0) += v;
        }
        if round.edits == 0 {
            break;
        }
    }
    // 模块级死常量/死内联函数清除（全部使用位已被传播/内联消灭）。
    // 门控：**ESM 模块检测**（import/export 存在——classic script 不可能
    // 含这些语句；脚本顶层是全局属性，外部脚本可读——删除不可证安全）
    // 或 --dead-code 显式开启。
    let module_detected = unit.items.iter().any(|it| {
        matches!(
            it,
            TopItem::Import(_)
                | TopItem::ExportRaw(_)
                | TopItem::ExportStmt(_)
        ) || matches!(it, TopItem::Func { sig_raw, .. } if sig_raw.contains("export"))
            || matches!(it, TopItem::Class(c) if c.head_raw.contains("export"))
    });
    if module_detected || cfg.remove_dead_methods {
        total.edits += remove_dead_module_items(ast, unit);
    }
    total
}

/// 模块级死项清除：
/// - **死常量**：名字 ∈ const_fields、全模块 VarRef 计数为零、任何
///   原文区域（Raw/模板 quasi/解构模式/签名）无该词——可安全移除声明
/// - **死内联函数**：名字 ∈ inline_funcs、VarRef 计数零（调用全部被
///   内联消灭）、非导出——移除声明
/// 引用计数必须覆盖**原文区域**：Raw 文本里出现过的名字（JSX/坏区域）
/// 视为被引用——保守保留。
fn remove_dead_module_items(ast: &mut TsAst, unit: &mut ModuleUnit) -> usize {
    // 每项的 (VarRef 引用集, 原文区域) 分别收集——候选项判定时
    // **排除自身**（函数签名里的自身名字、声明自身不算引用——
    // 否则死函数永远「被自己引用」）
    let n_items = unit.items.len();
    let mut item_refs: Vec<HashSet<String>> = Vec::with_capacity(n_items);
    let mut item_raws: Vec<Vec<String>> = Vec::with_capacity(n_items);
    for item in &unit.items {
        let mut refs: HashSet<String> = HashSet::new();
        let mut raws: Vec<String> = Vec::new();
        match item {
            TopItem::Stmt(s) | TopItem::ExportStmt(s) => {
                collect_refs_and_raws(ast, *s, &mut refs, &mut raws);
            }
            TopItem::Func { sig_raw, body, .. } => {
                raws.push(sig_raw.clone());
                if let Some(b) = body {
                    collect_refs_and_raws(ast, *b, &mut refs, &mut raws);
                }
            }
            TopItem::Class(c) => {
                raws.push(c.head_raw.clone());
                for m in &c.members {
                    match m {
                        ClassMember::Method { sig_raw, body } => {
                            raws.push(sig_raw.clone());
                            if let Some(b) = body {
                                collect_refs_and_raws(ast, *b, &mut refs, &mut raws);
                            }
                        }
                        ClassMember::Field { sig_raw, init, .. } => {
                            raws.push(sig_raw.clone());
                            if let Some(i) = init {
                                collect_refs_and_raws(ast, *i, &mut refs, &mut raws);
                            }
                        }
                        ClassMember::StaticBlock { body } => {
                            collect_refs_and_raws(ast, *body, &mut refs, &mut raws);
                        }
                        ClassMember::Raw(raw) => raws.push(raw.clone()),
                    }
                }
            }
            TopItem::Import(raw) | TopItem::ExportRaw(raw) | TopItem::Raw(raw) => {
                raws.push(raw.clone());
            }
        }
        item_refs.push(refs);
        item_raws.push(raws);
    }
    // 全项（含原文词）引用并集
    let all_words = |exclude: Option<usize>| -> HashSet<String> {
        let mut out: HashSet<String> = HashSet::new();
        for (i, refs) in item_refs.iter().enumerate() {
            if Some(i) != exclude {
                out.extend(refs.iter().cloned());
            }
        }
        for (i, raws) in item_raws.iter().enumerate() {
            if Some(i) == exclude {
                continue;
            }
            for raw in raws {
                for w in raw.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$')) {
                    if !w.is_empty() {
                        out.insert(w.to_string());
                    }
                }
            }
        }
        out
    };
    let mut removed = 0usize;
    let mut drop_indices: Vec<usize> = Vec::new();
    // Group 项（`var a=…,b=…`）：逐声明判定，保留存活声明重建 Group。
    // 组内引用必须**排除候选自身**、保留其它兄弟（`var p=…,e=f(p)`：
    // 整项引用并集排除会误判 p 未引用）。
    let mut group_rebuild: Vec<(usize, Option<TsId>)> = Vec::new();
    for (i, item) in unit.items.iter().enumerate() {
        match item {
            TopItem::Stmt(s) => {
                if let NodeData::Group = ast.data(*s) {
                    let kids = ast.children(*s).to_vec();
                    // 组内常量的存活是**图可达**问题：`var wi=1024,Ti=64*wi*wi;`
                    // 中 Ti 可能因传播而死，wi 的唯一引用来自这个将死的
                    // 兄弟——只按“任一兄弟引用”判定会保留 wi，二轮才删
                    // （npm 语料幂等差分抓获）。先算候选，再对“被活兄弟
                    // 引用”做不动点。
                    let cand: Vec<Option<String>> = kids
                        .iter()
                        .map(|&kid| {
                            if let NodeData::VarDecl { name, .. } = ast.data(kid) {
                                if ast.const_fields.contains_key(name)
                                    && !all_words(Some(i)).contains(name)
                                {
                                    return Some(name.clone());
                                }
                            }
                            None
                        })
                        .collect();
                    let mut live: Vec<bool> = cand.iter().map(|c| c.is_none()).collect();
                    loop {
                        let mut changed = false;
                        for k in 0..kids.len() {
                            if live[k] {
                                continue;
                            }
                            let name = cand[k].as_ref().unwrap();
                            for (j, &o) in kids.iter().enumerate() {
                                if j == k || !live[j] {
                                    continue;
                                }
                                let mut refs = HashSet::new();
                                let mut raws = Vec::new();
                                collect_refs_and_raws(ast, o, &mut refs, &mut raws);
                                if refs.contains(name) || raws.contains(name) {
                                    live[k] = true;
                                    changed = true;
                                    break;
                                }
                            }
                        }
                        if !changed {
                            break;
                        }
                    }
                    let keep: Vec<TsId> = kids
                        .iter()
                        .enumerate()
                        .filter(|(k, _)| live[*k])
                        .map(|(_, &kid)| kid)
                        .collect();
                    let changed = keep.len() != kids.len();
                    if changed {
                        let new_group = if keep.is_empty() {
                            None
                        } else if keep.len() == 1 {
                            Some(keep[0])
                        } else {
                            Some(ast.group(keep))
                        };
                        group_rebuild.push((i, new_group));
                    }
                } else if let NodeData::VarDecl { name, .. } = ast.data(*s) {
                    if ast.const_fields.contains_key(name)
                        && !all_words(Some(i)).contains(name)
                    {
                        drop_indices.push(i);
                    }
                }
            }
            TopItem::Func { sig_raw, name, .. } => {
                if !sig_raw.contains("export")
                    && ast.inline_funcs.contains_key(name)
                    && !all_words(Some(i)).contains(name)
                {
                    drop_indices.push(i);
                }
            }
            _ => {}
        }
    }
    removed = 0;
    // 先应用到 Group（不改项数），再删除整项（逆序保下标）
    for (i, new_item) in group_rebuild {
        match new_item {
            Some(g) => {
                unit.items[i] = TopItem::Stmt(g);
                removed += 1;
            }
            None => drop_indices.push(i),
        }
    }
    drop_indices.sort_unstable();
    drop_indices.dedup();
    for &i in drop_indices.iter().rev() {
        unit.items.remove(i);
        removed += 1;
    }
    removed
}

/// 收集子树的 VarRef 名 + Raw/模板 quasi/解构模式原文。
fn collect_refs_and_raws(ast: &TsAst, root: TsId, refs: &mut HashSet<String>, raws: &mut Vec<String>) {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        match ast.data(n) {
            NodeData::VarRef { name } => {
                refs.insert(ast.sn(*name).to_string());
            }
            NodeData::Raw { text } | NodeData::ClassExpr { raw: text } => {
                raws.push(text.clone());
            }
            NodeData::Template { quasis, .. } => {
                for q in quasis {
                    raws.push(q.clone());
                }
            }
            NodeData::Destruct { pattern_raw } => {
                raws.push(pattern_raw.clone());
            }
            NodeData::DestructStmt { pattern, .. } => {
                raws.push(pattern.clone());
            }
            NodeData::ObjLit { props } => raws.extend(props.iter().cloned()),
            NodeData::Lambda { params_raw, .. } | NodeData::FuncStmt { params_raw, .. } => {
                raws.push(params_raw.clone())
            }
            NodeData::Catch { param_raw: Some(p) } => raws.push(p.clone()),
            NodeData::ForEach { name, .. }
                if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$') =>
            {
                raws.push(name.clone())
            }
            _ => {}
        }
        for &c in ast.children(n) {
            stack.push(c);
        }
    }
}

/// 模块级常量/内联函数收集（每轮重算——Java 版同构）：
///
/// **常量接纳条件**（混淆器拷贝链的关键）：
/// - 名字全模块唯一声明（同名二见 → 放弃）
/// - **零写**：任何赋值目标/++--（含 `arr[i] =` 的数组基座）出现即除名
/// - **零遮蔽**：任何函数体/类成员/嵌套作用域内的同名声明/参数/绑定
/// - init 纯净：字面量、字面量数组、或**全部自由变量已在先前接纳集**
///   的纯表达式（`var a = 0x1; var b = a + 0x1;` 的链式拷贝——var
///   的非字面量 init 仅在引用先序常量时时间无关地确定）
///
/// **内联函数**：模块级 `function f(简单参数) { return 纯表达式; }`
/// ——字符串数组访问器 `_0x5d3e(i) { return arr[i - off]; }` 的调用点
/// 展开材料。
pub fn collect_unit_consts(ast: &mut TsAst, unit: &ModuleUnit) {
    ast.const_fields.clear();
    ast.const_scalars.clear();
    ast.inline_funcs.clear();

    // ---- 第一遍：模块级声明计数（唯一性）+ 内部遮蔽集 + 全单元写计数 ----
    // 关键区分：模块语句**根**的 VarDecl/解构是「被遮蔽者候选」——只计数
    // 不入 shadowed；其**内部**（初始化表达式里的 lambda/嵌套块）与一切
    // 函数/类体内的声明才是「遮蔽者」。曾因不区分而把每个模块声明都
    // 计入 shadowed ——常量收集全军覆没（obf1 样本抓获）。
    let mut shadowed: HashSet<String> = HashSet::new();
    let mut write_counts: std::collections::HashMap<String, u32> = Default::default();
    let mut decl_counts: std::collections::HashMap<String, u32> = Default::default();

    fn walk_interior(
        ast: &TsAst,
        from: TsId,
        skip_root_decl: bool,
        shadowed: &mut HashSet<String>,
        write_counts: &mut std::collections::HashMap<String, u32>,
    ) {
        // skip_root_decl：from 本身是模块语句根（其 VarDecl 只计数——
        // 由调用方处理），从孩子开始走
        let mut stack: Vec<TsId> = Vec::new();
        let roots: Vec<TsId> = if skip_root_decl {
            ast.children(from).to_vec()
        } else {
            vec![from]
        };
        for r in roots.into_iter().rev() {
            stack.push(r);
        }
        while let Some(n) = stack.pop() {
            match ast.data(n) {
                NodeData::VarDecl { name, .. } | NodeData::ForEach { name, .. } => {
                    shadowed.insert(name.clone());
                }
                NodeData::Catch { param_raw: Some(p) } => {
                    let p = p.trim();
                    if !p.is_empty()
                        && p.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$')
                        && !p.starts_with(|c: char| c.is_ascii_digit())
                    {
                        shadowed.insert(p.to_string());
                    }
                }
                NodeData::Destruct { pattern_raw } | NodeData::DestructStmt { pattern: pattern_raw, .. } => {
                    for w in pattern_raw
                        .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'))
                    {
                        if !w.is_empty() && !w.starts_with(|c: char| c.is_ascii_digit()) {
                            shadowed.insert(w.to_string());
                        }
                    }
                }
                NodeData::FuncStmt { name, params_raw, .. } => {
                    shadowed.insert(name.clone());
                    if let Some(ps) = simple_params(params_raw) {
                        for p in ps {
                            shadowed.insert(p);
                        }
                    }
                }
                NodeData::Lambda { params_raw, .. } => {
                    if let Some(ps) = simple_params(params_raw) {
                        for p in ps {
                            shadowed.insert(p);
                        }
                    }
                }
                NodeData::Assign { .. } => {
                    let t = ast.children(n)[0];
                    let mut ts = vec![t];
                    while let Some(x) = ts.pop() {
                        if ast.kind(x) == NodeKind::VarRef {
                            if let Some(nm) = ast.var_name(x) {
                                *write_counts.entry(nm.to_string()).or_insert(0) += 1;
                            }
                        }
                        for &c in ast.children(x) {
                            ts.push(c);
                        }
                    }
                }
                // ++/-- 目标也是写（seq 差分抓获：`var i = 0;
                // (i++, i++)` 的 i 曾未被计数而接纳为常量）
                NodeData::Unary { op }
                    if matches!(
                        op,
                        UnOp::PreInc | UnOp::PreDec | UnOp::PostInc | UnOp::PostDec
                    ) =>
                {
                    // 目标可能被 Paren 包裹（`(y)++`）——逐层剥开
                    let mut t = ast.children(n)[0];
                    while matches!(ast.data(t), NodeData::Paren) {
                        let Some(&c) = ast.children(t).first() else { break };
                        t = c;
                    }
                    if ast.kind(t) == NodeKind::VarRef {
                        if let Some(nm) = ast.var_name(t) {
                            *write_counts.entry(nm.to_string()).or_insert(0) += 1;
                        }
                    }
                }
                _ => {}
            }
            for &c in ast.children(n) {
                stack.push(c);
            }
        }
    }

    for item in &unit.items {
        match item {
            TopItem::Stmt(s) | TopItem::ExportStmt(s) => {
                // 根声明计数（唯一性）；多声明符 Group 的每个声明也要
                // 计数（否则 `var p=…,a=p+…,e=…` 全部无法接纳 → 首轮
                // 不传播、打印拆开后第二轮才传播 = 非幂等）
                let roots: Vec<TsId> = if matches!(ast.data(*s), NodeData::Group) {
                    ast.children(*s).to_vec()
                } else {
                    vec![*s]
                };
                for d in roots {
                    match ast.data(d) {
                        NodeData::VarDecl { name, .. } => {
                            *decl_counts.entry(name.clone()).or_insert(0) += 1;
                        }
                        NodeData::DestructStmt { pattern, .. } => {
                            for w in pattern.split(|c: char| {
                                !(c.is_alphanumeric() || c == '_' || c == '$')
                            }) {
                                if !w.is_empty() {
                                    *decl_counts.entry(w.to_string()).or_insert(0) += 1;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                // 初始化表达式**内部**（lambda 参数等）是遮蔽者；整个
                // 子树的写也要计。skip_root_decl 只对模块声明根生效——
                // `for (let x in …)` 的 ForEach 根自身名 x 必须计入
                // 遮蔽（否则模块 let x 被当常量传播进循环头/体，
                // Test262 for-in/of scope-head 家族抓获）。
                if matches!(ast.data(*s), NodeData::Group) {
                    // Group 的每个孩子都是模块声明根——逐个按
                    // skip_root_decl 行走（否则孩子自身名字进 shadowed，
                    // 多声明符常量永远无法接纳）
                    for &c in ast.children(*s) {
                        walk_interior(ast, c, true, &mut shadowed, &mut write_counts);
                    }
                } else {
                    let skip_root_decl = matches!(ast.data(*s), NodeData::VarDecl { .. });
                    walk_interior(ast, *s, skip_root_decl, &mut shadowed, &mut write_counts);
                }
            }
            TopItem::Func { body: Some(b), .. } => {
                walk_interior(ast, *b, false, &mut shadowed, &mut write_counts);
                // 函数名本身作为全局可调用——体内同名局部声明遮蔽它
            }
            TopItem::Func { name, .. } => {
                let _ = name;
            }
            TopItem::Class(c) => {
                for m in &c.members {
                    match m {
                        ClassMember::Method { body: Some(b), .. } => {
                            walk_interior(ast, *b, false, &mut shadowed, &mut write_counts);
                        }
                        ClassMember::StaticBlock { body } => {
                            walk_interior(ast, *body, false, &mut shadowed, &mut write_counts);
                        }
                        ClassMember::Field { init: Some(i), .. } => {
                            walk_interior(ast, *i, false, &mut shadowed, &mut write_counts);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    // ---- 原文区域词集 ----
    // 原文区（类表达式/Raw/模板 quasi/解构模式/签名）里的名字对
    // AST 扫描不可见——其中的**写**同样不可见（`callCount = callCount
    // + 1` 在 class 表达式原文内曾被当常量传播；Test262 class/dstr
    // 族 ~80 用例抓获）。任何在原文中出现的名字一律拒绝接纳。
    let mut raw_words: HashSet<String> = HashSet::new();
    {
        let mut raws: Vec<String> = Vec::new();
        let mut has_eval = false;
        fn collect_raws(ast: &TsAst, root: TsId, raws: &mut Vec<String>, has_eval: &mut bool) {
            let mut stack = vec![root];
            while let Some(n) = stack.pop() {
                // 直接/间接 eval 都算：`(0, eval)('x=1')` 的 callee 是
                // Paren/Seq——只认裸 callee 会漏（Test262 eval-code
                // indirect 家族抓到常量被穿过 eval 传播）。任何名为 eval
                // 的 VarRef 一律保守触发。
                if let NodeData::VarRef { name } = ast.data(n) {
                    if ast.sn(*name) == "eval" {
                        *has_eval = true;
                    }
                }
                match ast.data(n) {
                    NodeData::Raw { text } | NodeData::ClassExpr { raw: text } => {
                        raws.push(text.clone())
                    }
                    NodeData::Template { quasis, .. } => {
                        for q in quasis {
                            raws.push(q.clone());
                        }
                    }
                    NodeData::Destruct { pattern_raw } => raws.push(pattern_raw.clone()),
                    NodeData::DestructStmt { pattern, .. } => raws.push(pattern.clone()),
                    // 对象字面量：访问器/方法属性整体存在 props 原文里
                    // （`{ get v() { count++; } }`），值表达式才是孩子。
                    NodeData::ObjLit { props } => raws.extend(props.iter().cloned()),
                    // for-of/for-in 的解构头整体存在 name（`{ a = x += 1 }`）。
                    // 简单标识符无原文需求，非标识符一律按原文处理。
                    NodeData::ForEach { name, .. }
                        if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$') =>
                    {
                        raws.push(name.clone())
                    }
                    NodeData::FuncStmt { sig_raw, params_raw, .. } => {
                        raws.push(sig_raw.clone());
                        raws.push(params_raw.clone());
                    }
                    // 函数表达式形参默认值/解构 getter 里的写不可见——
                    // 必须进原文词集（Test262 dstr 家族抓获）
                    NodeData::Lambda { params_raw, .. } => raws.push(params_raw.clone()),
                    // catch 解构参数默认值同样藏写（`catch ([[] = f()])`）
                    NodeData::Catch { param_raw: Some(p) } => raws.push(p.clone()),
                    _ => {}
                }
                for &c in ast.children(n) {
                    stack.push(c);
                }
            }
        }
        for item in &unit.items {
            match item {
                TopItem::Stmt(s) | TopItem::ExportStmt(s) => collect_raws(ast, *s, &mut raws, &mut has_eval),
                TopItem::Func { sig_raw, body, .. } => {
                    raws.push(sig_raw.clone());
                    if let Some(b) = body {
                        collect_raws(ast, *b, &mut raws, &mut has_eval);
                    }
                }
                TopItem::Class(c) => {
                    raws.push(c.head_raw.clone());
                    for m in &c.members {
                        match m {
                            ClassMember::Method { sig_raw, body } => {
                                raws.push(sig_raw.clone());
                                if let Some(b) = body {
                                    collect_raws(ast, *b, &mut raws, &mut has_eval);
                                }
                            }
                            ClassMember::Field { sig_raw, init, .. } => {
                                raws.push(sig_raw.clone());
                                if let Some(i) = init {
                                    collect_raws(ast, *i, &mut raws, &mut has_eval);
                                }
                            }
                            ClassMember::StaticBlock { body } => {
                                collect_raws(ast, *body, &mut raws, &mut has_eval)
                            }
                            ClassMember::Raw(r) => raws.push(r.clone()),
                        }
                    }
                }
                TopItem::Import(r) | TopItem::ExportRaw(r) | TopItem::Raw(r) => {
                    raws.push(r.clone())
                }
            }
        }
        for raw in &raws {
            for w in raw.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$')) {
                if !w.is_empty() {
                    raw_words.insert(w.to_string());
                }
            }
        }
        has_eval |= raws.iter().any(|r| cure_ts_ast::raw_has_direct_eval(r));
        if has_eval {
            // 直接 eval 的字符串内容可写任意外层名字（`var x=1;
            // eval('x=2'); use(x)` 的写对 AST 完全不可见）——全单元放弃
            // 常量传播/内联（语义保真优先；混淆器极少用 eval）。
            return;
        }
    }

    // ---- 第二遍：按声明序接纳模块级常量（链式拷贝） ----
    let mut accepted: HashSet<String> = HashSet::new();
    let mut scalar_set: HashSet<String> = HashSet::new();
    // 声明之前已被引用的名字不可接纳：`assert(typeof f); var f = 1;` 的
    // 早读看到的是 undefined（var 提升）/eval 注入值——常量传播把早读
    // 也换成 1（Test262 annexB eval-code 用例：间接 eval 注入全局 f，
    // 早读是 function）。按模块项顺序累积先序引用集。
    let mut earlier_refs: HashSet<String> = HashSet::new();
    for item in &unit.items {
        let mut refs: HashSet<String> = HashSet::new();
        let mut raws: Vec<String> = Vec::new();
        match item {
            TopItem::Stmt(s) | TopItem::ExportStmt(s) => {
                collect_refs_and_raws(ast, *s, &mut refs, &mut raws);
            }
            TopItem::Func { body: Some(b), .. } => {
                collect_refs_and_raws(ast, *b, &mut refs, &mut raws);
            }
            TopItem::Class(c) => {
                for m in &c.members {
                    match m {
                        ClassMember::Method { body: Some(b), .. } | ClassMember::StaticBlock { body: b } => {
                            collect_refs_and_raws(ast, *b, &mut refs, &mut raws);
                        }
                        ClassMember::Field { init: Some(i), .. } => {
                            collect_refs_and_raws(ast, *i, &mut refs, &mut raws);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        if let TopItem::Stmt(s) | TopItem::ExportStmt(s) = item {
            // 多声明符 Group：按声明序逐个接纳（`var p=…, a=p+…, e=…`）
            let cands: Vec<TsId> = if matches!(ast.data(*s), NodeData::Group) {
                ast.children(*s).to_vec()
            } else {
                vec![*s]
            };
            for d in cands {
            if let NodeData::VarDecl { name, .. } = ast.data(d) {
                let name = name.clone();
                let init = ast.children(d).first().copied();
                let candidate = init.is_some_and(|init| {
                    decl_counts.get(&name).copied().unwrap_or(0) == 1
                        && !shadowed.contains(&name)
                        && !earlier_refs.contains(&name)
                        && write_counts.get(&name).copied().unwrap_or(0) == 0
                        && !raw_words.contains(&name)
                        && const_init_ok(ast, init, &accepted, &scalar_set)
                });
                if candidate {
                    let init = init.unwrap();
                    // 标量不只“已折叠的字面量”：Binary/Paren 常量在
                    // const_init_ok 下已保证操作数全为标量、效果≤MayRead，
                    // 结果必为原始值（string/number/bool）——可在**首轮**
                    // 就参与传播。否则长常量链每轮只前进一层，第六轮后
                    // 仍未收敛（真实 Test262 RegExp 语料二轮漂移抓获）。
                    let scalar_ok = is_scalar_const(ast, init);
                    ast.const_fields.insert(name.clone(), init);
                    if scalar_ok {
                        ast.const_scalars.insert(name.clone());
                        scalar_set.insert(name.clone());
                    }
                    accepted.insert(name);
                }
            }
            }
        }
        earlier_refs.extend(refs);
    }

    // ---- 第三遍：数组逃逸扫描 ----
    // 数组候选（const_fields 中 init 为 ArrayLit 的名字）仅当**每个**
    // 使用都在只读上下文（Index 读基座 / 纯方法接收者 / for-of 迭代 /
    // spread / 模板插值）才保留下标折叠资格。任何其他上下文
    //（别名赋值/调用实参/return/未知方法接收者——`copy = arr;
    // copy.sort()` / `log.push(x)` 的突变能力对写扫描不可见）→ 除名。
    {
        let array_names: HashSet<String> = ast
            .const_fields
            .iter()
            .filter(|(_, &init)| ast.kind(init) == NodeKind::ArrayLit)
            .map(|(k, _)| k.clone())
            .collect();
        if !array_names.is_empty() {
            // 父/祖父表（walk 需要两级上下文判定赋值目标位的 Index）
            let mut parents: std::collections::HashMap<TsId, (TsId, usize)> = Default::default();
            let mut escaped: HashSet<String> = HashSet::new();
            let mut roots: Vec<TsId> = Vec::new();
            for_each_item_body(unit, &mut |r| roots.push(r));
            for root in roots {
                let mut stack = vec![(root, TsId(u32::MAX), 0usize)];
                while let Some((n, p, i)) = stack.pop() {
                    if p != TsId(u32::MAX) {
                        parents.insert(n, (p, i));
                    }
                    if let NodeData::VarRef { name } = ast.data(n) {
                        let nm = ast.sn(*name);
                        if array_names.contains(nm) {
                            if !array_read_context(ast, n, &parents) {
                                escaped.insert(nm.to_string());
                            }
                        }
                    }
                    for (ci, &c) in ast.children(n).iter().enumerate() {
                        stack.push((c, n, ci));
                    }
                }
            }
            for e in escaped {
                ast.const_fields.remove(&e);
            }
        }
    }

    // ---- 内联函数：模块级单 return（体不含 this/arguments/自引用） ----
    for item in &unit.items {
        if let TopItem::Func { sig_raw, body: Some(b), name, .. } = item {
            let _ = name;
            if sig_raw.contains("export default") {
                continue;
            }
            let stmts = ast.children(*b).to_vec();
            if stmts.len() != 1 {
                continue;
            }
            if !matches!(ast.data(stmts[0]), NodeData::Return) {
                continue;
            }
            let Some(&expr) = ast.children(stmts[0]).first() else { continue };
            // 参数提取（sig 首对圆括号）
            let open = match sig_raw.find('(') {
                Some(o) => o,
                None => continue,
            };
            let bytes = sig_raw.as_bytes();
            let mut depth = 0i32;
            let mut close = None;
            for (i, &bb) in bytes.iter().enumerate().skip(open) {
                match bb {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            close = Some(i);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let Some(close) = close else { continue };
            let Some(params) = simple_params(&sig_raw[open + 1..close]) else {
                continue;
            };
            let fname = sig_name(sig_raw);
            if fname.is_empty() || shadowed.contains(&fname) {
                continue;
            }
            // 体守卫：this / arguments / 自引用（递归内联会无限展开；
            // this 的调用点绑定与表达式位置不同——语义改变）
            if body_has_inline_hazard(ast, expr, &fname) {
                continue;
            }
            ast.inline_funcs.insert(fname, (params, expr));
        }
    }
}

/// 内联危险：this / arguments / 递归自引用。
fn body_has_inline_hazard(ast: &TsAst, expr: TsId, fname: &str) -> bool {
    let mut stack = vec![expr];
    while let Some(n) = stack.pop() {
        match ast.data(n) {
            NodeData::This => return true,
            NodeData::VarRef { name } => {
                if ast.sn(*name) == fname || ast.sn(*name) == "arguments" {
                    return true;
                }
            }
            // 写参数/任意赋值：内联把实参字面量替换进写目标产
            // `++1` / `10 += 20`（Test262 S13_A4_T2/T3 抓获）
            NodeData::Assign { .. } => return true,
            NodeData::Unary { op } if op.is_incdec() => return true,
            NodeData::Raw { .. } | NodeData::ClassExpr { .. } => return true,
            _ => {}
        }
        for &c in ast.children(n) {
            stack.push(c);
        }
    }
    false
}

/// 常量初始化式是否产生**原始值**（可安全多副本传播）。
/// `const_init_ok` 已保证 Binary/Paren 的操作数全为标量、效果 ≤ MayRead；
/// Paren 只看内部——数组/对象即使被括号包住也不是标量（同一性）。
fn is_scalar_const(ast: &TsAst, id: TsId) -> bool {
    match ast.kind(id) {
        NodeKind::Literal => is_scalar_lit(ast, id),
        NodeKind::Binary => true,
        NodeKind::Paren => ast
            .children(id)
            .first()
            .is_some_and(|&c| is_scalar_const(ast, c)),
        _ => false,
    }
}

/// 标量字面量（正则除外——对象语义）。
fn is_scalar_lit(ast: &TsAst, n: TsId) -> bool {
    matches!(ast.data(n), NodeData::Literal(l) if !matches!(l, JsLit::Regex(_)))
}

/// 常量 init 判定：字面量 / 字面量数组 / 自由变量全在已接纳集的纯表达式。
fn const_init_ok(
    ast: &TsAst,
    init: TsId,
    accepted: &HashSet<String>,
    scalar_set: &HashSet<String>,
) -> bool {
    match ast.kind(init) {
        // 正则字面量是**对象**：每次求值新实例（lastIndex 状态、同一性、
        // defineProperty 都不同）——绝不能当标量常量传播（Test262
        // RegExp while(/g.exec) 死循环 / Symbol.matchAll getter 抓获）。
        NodeKind::Literal => is_scalar_lit(ast, init),
        NodeKind::ArrayLit => {
            if ast.effect(init) > Effect::MayRead {
                return false;
            }
            // 数组元素：字面量或已接纳常量的引用（混淆器字符串数组的
            // 元素有时本身就是前置常量名——Java 版同构支持）
            let mut ok = true;
            let mut stack = vec![init];
            while let Some(n) = stack.pop() {
                match ast.kind(n) {
                    NodeKind::Literal => {
                        if !is_scalar_lit(ast, n) {
                            ok = false;
                            break;
                        }
                    }
                    NodeKind::ArrayLit => {}
                    NodeKind::VarRef => {
                        let nm = ast.var_name(n).unwrap_or("");
                        if !accepted.contains(nm) {
                            ok = false;
                            break;
                        }
                    }
                    NodeKind::Binary => {}
                    _ => {
                        ok = false;
                        break;
                    }
                }
                for &c in ast.children(n) {
                    stack.push(c);
                }
            }
            ok
        }
        // 括号只是语法层：语义按内部表达式判定（`("a" + B)` 与
        // `"a" + B` 同值）。
        NodeKind::Paren => {
            let Some(&inner) = ast.children(init).first() else { return false };
            const_init_ok(ast, inner, accepted, scalar_set)
        }
        NodeKind::Binary => {
            // 表达式效果 ≤ MayRead 且自由变量全在先序接纳集**且为标量**
            //（数组引用 = 别名——突变能力经别名传导，链式拷贝对数组
            // 恒不安全；标量值语义无此问题）
            if ast.effect(init) > Effect::MayRead {
                return false;
            }
            let mut ok = true;
            let mut stack = vec![init];
            while let Some(n) = stack.pop() {
                match ast.kind(n) {
                    NodeKind::Literal => {
                        if !is_scalar_lit(ast, n) {
                            ok = false;
                            break;
                        }
                    }
                    NodeKind::Binary | NodeKind::Paren => {}
                    NodeKind::VarRef => {
                        let nm = ast.var_name(n).unwrap_or("");
                        if !scalar_set.contains(nm) {
                            ok = false;
                            break;
                        }
                    }
                    _ => {
                        ok = false;
                        break;
                    }
                }
                for &c in ast.children(n) {
                    stack.push(c);
                }
            }
            ok
        }
        _ => false,
    }
}

/// sig 里的函数名（"function name(" / "function* name("）。
fn sig_name(sig_raw: &str) -> String {
    let after = sig_raw.strip_prefix("function").unwrap_or(sig_raw);
    let after = after.strip_prefix('*').unwrap_or(after);
    let name: String = after
        .trim_start()
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
        .collect();
    name
}

/// 全部可执行体根（语句/函数体/类成员/字段 init）。
fn for_each_item_body(unit: &ModuleUnit, f: &mut dyn FnMut(TsId)) {
    for item in &unit.items {
        match item {
            TopItem::Stmt(s) | TopItem::ExportStmt(s) => f(*s),
            TopItem::Func { body: Some(b), .. } => f(*b),
            TopItem::Class(c) => {
                for m in &c.members {
                    match m {
                        ClassMember::Method { body: Some(b), .. } => f(*b),
                        ClassMember::StaticBlock { body } => f(*body),
                        ClassMember::Field { init: Some(i), .. } => f(*i),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

/// 数组候选的 VarRef 是否处于**只读上下文**。
fn array_read_context(
    ast: &TsAst,
    varref: TsId,
    parents: &std::collections::HashMap<TsId, (TsId, usize)>,
) -> bool {
    let Some(&(p, ci)) = parents.get(&varref) else {
        return false; // 根位——保守逃逸
    };
    match ast.kind(p) {
        // arr[i] 读位：Index 基座且 Index 不在写/删除位
        NodeKind::Index => {
            if ci != 0 {
                return false;
            }
            if let Some(&(gp, gci)) = parents.get(&p) {
                // Index 作为 Assign 的 child-0 = 写目标 → 逃逸；
                // `delete arr[1]` / `arr[0]++`（Unary 父）同样突变。
                // reduceRight 用例：arr 曾被当常量、`delete arr[1]`
                // 静态下标折成 `delete 2`。
                if ast.kind(gp) == NodeKind::Assign && gci == 0 {
                    return false;
                }
                if ast.kind(gp) == NodeKind::Unary {
                    return false;
                }
            }
            true
        }
        // arr.length / arr.join()——纯读成员
        NodeKind::Member => {
            if ci != 0 {
                return false;
            }
            if let Some(&(gp, gci)) = parents.get(&p) {
                // arr.length = 0 / delete arr.length
                if ast.kind(gp) == NodeKind::Assign && gci == 0 {
                    return false;
                }
                if ast.kind(gp) == NodeKind::Unary {
                    return false;
                }
            }
            let mname = ast.member_name(p).unwrap_or("");
            if mname == "length" {
                return true;
            }
            ARRAY_PURE_METHODS.contains(&mname)
        }
        // for (const x of arr) ——迭代只读
        NodeKind::ForEach => ci == 0,
        // 原文类自定义节点按种类白名单：只有这些位置对数组是纯读。
        // ObjLit/ClassExpr/Raw 等（存进对象/未知原文）一律逃逸。
        NodeKind::Custom(k) => {
            use cure_ts_ast::custom;
            ci == 0
                && matches!(
                    k,
                    custom::SPREAD
                        | custom::TEMPLATE
                        | custom::TS_AS
                        | custom::TS_SATISFIES
                        | custom::TS_NON_NULL
                        | custom::DESTRUCT
                        | custom::DESTRUCT_STMT
                        | custom::SEQ
                )
        }
        _ => false,
    }
}

/// 数组的纯读方法（未知方法默认逃逸——用户函数可能突变）。
const ARRAY_PURE_METHODS: &[&str] = &[
    "join", "indexOf", "lastIndexOf", "includes", "concat", "slice", "at",
    "entries", "keys", "values", "toString", "toLocaleString", "find",
    "findIndex", "findLast", "findLastIndex", "flat", "flatMap", "map",
    "filter", "reduce", "reduceRight", "forEach", "some", "every",
    "toSorted", "toReversed", "toSpliced", "with",
];

fn simplify_top_item(ast: &mut TsAst, item: &mut TopItem, cfg: &Config, total: &mut Report) {
    match item {
        TopItem::Stmt(s) => {
            simplify_stmt_root(ast, s, cfg, total);
        }
        TopItem::ExportStmt(s) => {
            simplify_stmt_root(ast, s, cfg, total);
        }
        TopItem::Func { sig_raw, body, .. } => {
            // 形参默认值内的直接 eval 会把 var 注入函数作用域——整个
            // 函数跳过简化（静态作用域分析失效）。
            if !cure_ts_ast::raw_has_direct_eval(sig_raw) {
                let params = sig_params(sig_raw);
                ast.set_param_scope(&params);
                if let Some(b) = body {
                    let r = simplify(ast, *b, cfg);
                    merge_report(total, r);
                }
                ast.clear_param_scope();
            }
        }
        TopItem::Class(c) => {
            simplify_class(ast, c, cfg, total);
        }
        _ => {}
    }
}

fn simplify_class(ast: &mut TsAst, c: &mut ClassDecl, cfg: &Config, total: &mut Report) {
    for m in &mut c.members {
        simplify_class_member(ast, m, cfg, total);
    }
}

fn simplify_class_member(ast: &mut TsAst, m: &mut ClassMember, cfg: &Config, total: &mut Report) {
    match m {
        ClassMember::Method { sig_raw, body } => {
            if cure_ts_ast::raw_has_direct_eval(sig_raw) {
                return;
            }
            let params = sig_params(sig_raw);
            ast.set_param_scope(&params);
            if let Some(b) = body {
                let r = simplify(ast, *b, cfg);
                merge_report(total, r);
            }
            ast.clear_param_scope();
        }
        ClassMember::StaticBlock { body } => {
            let r = simplify(ast, *body, cfg);
            merge_report(total, r);
        }
        ClassMember::Field { init, .. } => {
            if let Some(i) = init {
                simplify_stmt_root(ast, i, cfg, total);
            }
        }
        ClassMember::Raw(_) => {}
    }
}

/// 顶层语句根：普通语句直接跑；VarDecl init 根折叠（引擎 Replace 无
/// 父槽的既有边界——由本侧拿新根回写）。
fn simplify_stmt_root(ast: &mut TsAst, s: &mut TsId, cfg: &Config, total: &mut Report) {
    if matches!(ast.data(*s), NodeData::VarDecl { .. }) {
        // 声明根：init 折叠后回写孩子
        if let Some(&init) = ast.children(*s).first() {
            let rules = default_ts_rules();
            let mut cur = init;
            let mut new_edits = 0;
            for _ in 0..8 {
                match cure_engine::fold_root(ast, cur, &rules, cfg) {
                    Some((new_root, name)) => {
                        ast.set_child(*s, 0, new_root);
                        cur = new_root;
                        *total.by_rule.entry(name).or_insert(0) += 1;
                        new_edits += 1;
                    }
                    None => break,
                }
            }
            if new_edits > 0 {
                total.edits += new_edits;
                let r = simplify(ast, *s, cfg);
                merge_report(total, r);
            } else {
                let r = simplify(ast, *s, cfg);
                merge_report(total, r);
            }
        } else {
            let r = simplify(ast, *s, cfg);
            merge_report(total, r);
        }
        return;
    }
    let r = simplify(ast, *s, cfg);
    merge_report(total, r);
}

/// 签名原文的首对圆括号内参数（简单名）。
fn sig_params(sig_raw: &str) -> Vec<String> {
    let open = match sig_raw.find('(') {
        Some(o) => o,
        None => return vec![],
    };
    // 平衡找配对 `)`
    let bytes = sig_raw.as_bytes();
    let mut depth = 0i32;
    let mut close = None;
    for (i, &b) in bytes.iter().enumerate().skip(open) {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = match close {
        Some(c) => c,
        None => return vec![],
    };
    let inner = &sig_raw[open + 1..close];
    simple_params(inner).unwrap_or_default()
}

fn merge_report(total: &mut Report, r: Report) {
    total.edits += r.edits;
    total.iterations += r.iterations;
    total.discarded += r.discarded;
    for (k, v) in r.by_rule {
        *total.by_rule.entry(k).or_insert(0) += v;
    }
}

// ---------------------------------------------------------------------------
// TS 专属规则包（混淆器对抗）
// ---------------------------------------------------------------------------

/// **模块常量传播**：`_0x4b1c`（const_fields）→ init 子树拷贝。
/// 收集期已保证零写/零遮蔽/先序确定（见 collect_unit_consts）。
pub struct TsConstPropagate;

impl Rule<TsAst> for TsConstPropagate {
    fn name(&self) -> &'static str {
        "ts_const_propagate"
    }
    /// 结构性：VarRef → init 子树的成本可能**上升**（链式拷贝的表达式
    /// 比名字贵）——单调成本门会拒绝。净简化在后续折叠完成（Java 的
    /// ConstFieldPropagate 同款声明）。
    fn structural(&self) -> bool {
        true
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::VarRef]
    }
    fn check(&self, ctx: RewriteCtx<'_, TsAst>, id: TsId) -> Option<Edit<TsAst>> {
        if ctx.lang.kind(id) != NodeKind::VarRef {
            return None;
        }
        // 目标位检查先行（ctx 借用结束后再拿 lang 的可变借用）
        if let Some(p) = ctx.parent(id) {
            let k = ctx.lang.kind(p);
            if k == NodeKind::Assign && ctx.lang.children(p).first() == Some(&id) {
                return None;
            }
            if k == NodeKind::Unary {
                if let Some(op) = ctx.lang.un_op(p) {
                    if matches!(
                        op,
                        UnOp::PreInc | UnOp::PreDec | UnOp::PostInc | UnOp::PostDec
                    ) && ctx.lang.children(p).first() == Some(&id)
                    {
                        return None;
                    }
                    // `delete x` 对 var/let 绑定返回 false 且不删除——把 x
                    // 换成字面量后 `delete 1` 返回 true（语义改变；
                    // Test262 delete 家族抓获）。
                    if op == UnOp::Delete {
                        return None;
                    }
                }
            }
        }
        let RewriteCtx { lang, .. } = ctx;
        let name = lang.var_name(id)?;
        // 仅标量（值语义——数组/对象传播会破坏同一性：`copy === arr`
        // 曾被替换成 `copy === [3,1,2]`；数组走 ts_static_array_index
        // 的下标折叠）
        if !lang.const_scalars.contains(name) {
            return None;
        }
        let Some(&init) = lang.const_fields.get(name) else {
            return None;
        };
        // init 效果 ≤ MayRead：自由变量已由收集期的先序接纳约束保证
        // 全是**已验证常量**——常量读取无位置敏感性（写/抛/未知调用
        // 仍然拒绝）
        if lang.effect(init) > Effect::MayRead {
            return None;
        }
        let copy = lang.copy_subtree(init);
        Some(Edit::Replace { target: id, with: copy })
    }
}

/// **常量数组下标折叠**：`_0x3f2a[2]`（const ArrayLit）→ 元素字面量。
/// 数组内容零写（收集期写扫描含 `arr[i] =` 基座）→ 值恒定。
pub struct TsStaticArrayIndex;

impl Rule<TsAst> for TsStaticArrayIndex {
    fn name(&self) -> &'static str {
        "ts_static_array_index"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Index]
    }
    fn check(&self, ctx: RewriteCtx<'_, TsAst>, id: TsId) -> Option<Edit<TsAst>> {
        let RewriteCtx { lang, .. } = ctx;
        if lang.kind(id) != NodeKind::Index {
            return None;
        }
        let ch = lang.children(id).to_vec();
        if ch.len() != 2 {
            return None;
        }
        let (base, idx) = (ch[0], ch[1]);
        // optional index 不碰
        if matches!(lang.data(id), NodeData::Index { optional: true }) {
            return None;
        }
        // 基座：const 名字（VarRef）或**内联后的数组字面量本体**
        //（ts_const_propagate 把整个 ArrayLit 拷到使用位——直配形态
        // 同样可折；数组是语法常量）
        let arr = match lang.data(base) {
            NodeData::VarRef { name } => {
                let name = lang.sn(*name);
                let Some(&a) = lang.const_fields.get(name) else {
                    return None;
                };
                a
            }
            _ if lang.kind(base) == NodeKind::ArrayLit => base,
            _ => return None,
        };
        if lang.kind(arr) != NodeKind::ArrayLit {
            return None;
        }
        // 全字面量元素（非字面量元素的下标折叠会改变求值语义）
        let elems = lang.children(arr).to_vec();
        if !elems.iter().all(|&e| lang.kind(e) == NodeKind::Literal) {
            return None;
        }
        let i = match lang.literal(idx) {
            Some(LitRef::Int(v)) => v,
            _ => return None,
        };
        let i = usize::try_from(i).ok()?;
        let &elem = elems.get(i)?;
        Some(Edit::Replace { target: id, with: elem })
    }
}

/// **真值字面量折叠**：`![]` → false、`!!{}` → true（经 bool_not_fold
/// 链式收敛）——混淆器的 JSFuck 风格布尔噪声。
pub struct TsTruthyFold;

impl Rule<TsAst> for TsTruthyFold {
    fn name(&self) -> &'static str {
        "ts_truthy_fold"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Unary]
    }
    fn check(&self, ctx: RewriteCtx<'_, TsAst>, id: TsId) -> Option<Edit<TsAst>> {
        let RewriteCtx { lang, .. } = ctx;
        if lang.un_op(id) != Some(UnOp::Not) {
            return None;
        }
        let operand = *lang.children(id).first()?;
        let truthy = definitely_truthy(lang, operand)?;
        let v = !truthy; // !truthy → false；!falsy → true
        Some(Edit::Replace {
            target: id,
            with: lang.build_bool(v),
        })
    }
}

/// 操作数是否「确定真值」（JS 语义：[] / {} / new / 非空串 / 非零数 /
/// true 恒真；0 / NaN / '' / null / false 恒假）。None = 不确定。
fn definitely_truthy(lang: &TsAst, e: TsId) -> Option<bool> {
    use JsLit::*;
    match lang.data(e) {
        NodeData::Literal(Bool(b)) => Some(*b),
        // 空数组是 truthy（JS 语义）
        NodeData::ArrayLit | NodeData::ObjLit { .. } | NodeData::New => Some(true),
        NodeData::Literal(Str(s)) => Some(!s.is_empty()),
        NodeData::Literal(Int(v)) => Some(*v != 0),
        NodeData::Literal(Double(v)) => Some(*v != 0.0 && !v.is_nan()),
        NodeData::Literal(Null) => Some(false),
        NodeData::Paren => definitely_truthy(lang, lang.children(e).first().copied()?),
        NodeData::Template { quasis, .. } => {
            // 空模板 `` → '' falsy；有内容/插值 truthy（插值字符串化
            // 恒非空？不—— `${''}` 是空——保守：有插值 → None）
            let has_interp = !lang.children(e).is_empty();
            if has_interp {
                return None;
            }
            Some(quasis.iter().any(|q| !q.is_empty()))
        }
        _ => None,
    }
}

/// **引号成员还原**：`console['log']` → `console.log`（字符串键是
/// 合法标识符且非数字形态——数字串 `obj['0']` 与 `obj[0]` 语义不同）。
pub struct TsMemberUnquote;

impl Rule<TsAst> for TsMemberUnquote {
    fn name(&self) -> &'static str {
        "ts_member_unquote"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Index]
    }
    fn check(&self, ctx: RewriteCtx<'_, TsAst>, id: TsId) -> Option<Edit<TsAst>> {
        let RewriteCtx { lang, .. } = ctx;
        if lang.kind(id) != NodeKind::Member {
            return None;
        }
        if matches!(lang.data(id), NodeData::Index { optional: true }) {
            return None;
        }
        let ch = lang.children(id).to_vec();
        if ch.len() != 2 {
            return None;
        }
        let (base, idx) = (ch[0], ch[1]);
        let key = match lang.literal(idx) {
            Some(LitRef::Str(s)) => s.to_string(),
            _ => return None,
        };
        // 合法标识符且非纯数字（'0' 下标是数字键不是成员名）
        if key.is_empty()
            || !key.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_' || c == '$')
            || !key.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$')
        {
            return None;
        }
        // 基座可为任意表达式（obj[...] → obj.name 语义恒等——
        // undefined/null 基座两者同样抛 TypeError）
        let m = lang.member(base, &key, false);
        Some(Edit::Replace { target: id, with: m })
    }
}

/// **常量函数内联**：`_0x5d3e(0x2)` → 函数体 return 表达式（参数字面量
/// 替换）——字符串数组访问器/解密 helper 的调用点展开。
pub struct TsInlineFunc;

impl Rule<TsAst> for TsInlineFunc {
    fn name(&self) -> &'static str {
        "ts_inline_func"
    }
    /// 结构性（调用 → 展开体的净简化由后续折叠完成）
    fn structural(&self) -> bool {
        true
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Call]
    }
    fn check(&self, ctx: RewriteCtx<'_, TsAst>, id: TsId) -> Option<Edit<TsAst>> {
        let RewriteCtx { lang, .. } = ctx;
        if lang.kind(id) != NodeKind::Call {
            return None;
        }
        if matches!(lang.data(id), NodeData::Call { optional: true }) {
            return None;
        }
        let ch = lang.children(id).to_vec();
        let callee = *ch.first()?;
        let name = match lang.data(callee) {
            NodeData::VarRef { name } => lang.sn(*name),
            _ => return None,
        };
        let Some((params, body_expr)) = lang.inline_funcs.get(name).cloned() else {
            return None;
        };
        let args = &ch[1..];
        if args.len() != params.len() || params.is_empty() {
            return None;
        }
        // 全部实参须为字面量（多引用安全——不可变；求值时机恒定）
        for &a in args {
            if lang.kind(a) != NodeKind::Literal {
                return None;
            }
        }
        // 参数名 → 实参节点映射（字面量节点可共享——不可变）
        let mut map: std::collections::HashMap<String, TsId> = Default::default();
        for (p, &a) in params.iter().zip(args.iter()) {
            map.insert(p.clone(), a);
        }
        // 体表达式逐参出现须恰一次？不——字面量可安全多引用（不可变）
        let expanded = subst_expr(lang, body_expr, &map);
        Some(Edit::Replace { target: id, with: expanded })
    }
}

/// 表达式的参数替换克隆（VarRef 命中参数 → 实参节点；否则按结构克隆）。
fn subst_expr(
    lang: &mut TsAst,
    n: TsId,
    map: &std::collections::HashMap<String, TsId>,
) -> TsId {
    if let NodeData::VarRef { name } = lang.data(n) {
        if let Some(&arg) = map.get(lang.sn(*name)) {
            return arg;
        }
    }
    let children: Vec<TsId> = lang.children(n).to_vec();
    let children: Vec<TsId> = children.into_iter().map(|c| subst_expr(lang, c, map)).collect();
    lang.clone_node(n, children)
}

/// TS 规则集：引擎默认规则（Lang 钩子承担 JS 语义守卫）+ TS 专属
/// 混淆对抗规则（常量传播/数组下标/真值折叠/成员还原/函数内联）。
pub fn default_ts_rules() -> Vec<Box<dyn Rule<TsAst>>> {
    let mut rules: Vec<Box<dyn Rule<TsAst>>> = cure_engine::rules::default_rules();
    rules.push(Box::new(TsConstPropagate));
    rules.push(Box::new(TsStaticArrayIndex));
    rules.push(Box::new(TsTruthyFold));
    rules.push(Box::new(TsMemberUnquote));
    rules.push(Box::new(TsInlineFunc));
    rules.push(Box::new(crate::rules::TsAssignToCompound));
    // 移植规则包（字面量求值 / try 清理 / else 展平 / 尾赋值返回 /
    // 纯调用删除）
    rules.extend(ported_rules());
    rules
}

// ---------------------------------------------------------------------------
// 参数作用域桥（TsAst 的 param_scope 为私有——本 crate 的门面是唯一
// 合法写入方）。SetParamScope trait 桥接。
// ---------------------------------------------------------------------------

/// 参数作用域注入 trait（pub API——parser/simplify 门面共用）。
pub trait SetParamScope {
    fn set_param_scope(&mut self, params: &[String]);
    fn clear_param_scope(&mut self);
}

impl SetParamScope for TsAst {
    fn set_param_scope(&mut self, params: &[String]) {
        self.param_scope_set(params);
    }
    fn clear_param_scope(&mut self) {
        self.param_scope_clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cure_ts_parser::parse;
    use cure_ts_print::print_module;

    fn run(src: &str) -> (String, Report) {
        let mut out = parse(src);
        assert!(out.errors.is_empty(), "{:#?}", out.errors);
        let report = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
        (print_module(&out.ast, &out.unit), report)
    }

    #[test]
    fn const_propagation_and_fold() {
        // 寄存器噪声：var 传播 + 常量折叠 + 死存储
        let (out, r) = run(
            "function f() {\n  var a = 2;\n  var b = 3;\n  var c = a * b;\n  return c + 0;\n}\n",
        );
        assert!(r.edits >= 2, "edits={}", r.edits);
        assert!(out.contains("return 6"), "{out}");
    }

    #[test]
    fn dead_store_removal() {
        // 死存储 + 传播链全折叠：a=1 死 → a=2 → b=a → return b ⇒ return 2
        let (out, r) = run("function f() {\n  var a = 1;\n  a = 2;\n  var b = a;\n  return b;\n}\n");
        assert!(r.edits >= 2, "edits={}", r.edits);
        assert!(out.contains("return 2"), "{out}");
    }

    #[test]
    fn var_hoisting_trap_respected() {
        // var 块外引用：声明不可删（var_trap 不透明）
        let (out, _r) = run(
            "function f(c) {\n  if (c) { var x = 1; }\n  return x;\n}\n",
        );
        assert!(out.contains("var x = 1"), "{out}");
    }

    #[test]
    fn if_to_ternary() {
        // if-else 赋值 → 三元 + 单使用传播 ⇒ return c ? 1 : 2
        let (out, r) = run(
            "function f(c) {\n  var r;\n  if (c) { r = 1; } else { r = 2; }\n  return r;\n}\n",
        );
        assert!(r.edits >= 1, "edits={}", r.edits);
        assert!(out.contains("return c ? 1 : 2"), "{out}");
    }

    #[test]
    fn semantics_js_safe() {
        // 2 ** 53 边界外的整数字面量不折
        let (out, _r) = run("function f() { return 9007199254740993 + 1; }\n");
        assert!(out.contains("9007199254740993 + 1"), "{out}");
    }

    #[test]
    fn semantics_undefined_not_touched() {
        // undefined 是全局名——不得当字面量处理
        let (out, _r) = run("function f(u) { return u ?? undefined; }\n");
        assert!(out.contains("undefined"), "{out}");
    }

    #[test]
    fn loop_body_simplify() {
        // 循环体内的常量折叠与声明合并照常发生
        let (out, r) = run(
            "function f(n) {\n  var i = 0;\n  while (i < n) { var t = 1 + 1; use(t); i = i + 1; }\n  return i;\n}\n",
        );
        assert!(r.edits >= 1, "edits={}", r.edits);
        assert!(out.contains("use(2)"), "{out}");
    }

    #[test]
    fn idempotent() {
        let src = "function f(n) {\n  var a = 2;\n  var b = a * 3;\n  if (b > 5) { return b + 1; }\n  return b - 1;\n}\n";
        let (once, _) = run(src);
        let mut out2 = parse(&once);
        assert!(out2.errors.is_empty(), "{:#?}\n== once ==\n{once}", out2.errors);
        let _ = simplify_unit(&mut out2.ast, &mut out2.unit, &Config::default());
        let twice = print_module(&out2.ast, &out2.unit);
        assert_eq!(once, twice, "\n== once ==\n{once}\n== twice ==\n{twice}");
    }

    #[test]
    fn class_method_bodies() {
        let (out, r) = run(
            "class A {\n  m(n) {\n    var t = n * 2;\n    var u = t + 1;\n    return u + 0;\n  }\n}\n",
        );
        assert!(r.edits >= 1, "edits={}", r.edits);
        assert!(out.contains("return n * 2 + 1"), "{out}");
    }

    #[test]
    fn arrow_bodies() {
        let (out, r) = run("const f = (a, b) => { var c = a + b; return c + 0; };\n");
        assert!(r.edits >= 1, "edits={}", r.edits);
        assert!(out.contains("return a + b"), "{out}");
    }
}
