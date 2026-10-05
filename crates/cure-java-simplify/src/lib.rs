//! # cure-java-simplify
//!
//! Java 简化门面：引擎通用规则 + Java 特有规则，一站式 [`simplify`]。
//!
//! 引擎规则（布尔、自赋值、常量条件、局部传播……）定义在 cure-engine，
//! 通过 `Lang` 抽象复用；本 crate 只放需要 Java 类型/负载信息的规则。

use cure_engine::kind::{BinOp, UnOp};
use cure_engine::{Config, Edit, Lang, LitRef, NodeKind, Report, RewriteCtx, Rule};
use cure_java_ast::{CompilationUnit, JavaAst, JavaId, JType, Lit, Member, NodeData, TypeDecl};

// ---------------------------------------------------------------------------
// Java 特有规则
// ---------------------------------------------------------------------------

/// 冗余 cast：
/// - `(T)(T) x → (T) x`（嵌套同型）
/// - `(T) x → x`（x 的声明类型就是 T，作用域解析后可知）
pub struct CastSimplify;

impl Rule<JavaAst> for CastSimplify {
    fn name(&self) -> &'static str {
        "cast_simplify"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        let NodeData::Cast { ty } = lang.data(id) else {
            return None;
        };
        let ty = ty.clone();
        let inner = lang.children(id)[0];

        // (T)(T) x → (T) x
        if let NodeData::Cast { ty: inner_ty } = lang.data(inner) {
            if *inner_ty == ty {
                return Some(Edit::Replace {
                    target: id,
                    with: inner,
                });
            }
        }

        // (T) x，x 声明类型 == T → 去掉 cast
        if let NodeData::VarRef { .. } = lang.data(inner) {
            if lang.var_type(inner) == Some(&ty) {
                return Some(Edit::Replace {
                    target: id,
                    with: inner,
                });
            }
        }
        None
    }
}

/// 同一整数局部变量的 `x == x → true` / `x != x → false`
/// （浮点有 NaN 语义，引用保守起见不碰）。
pub struct SelfCompare;

impl Rule<JavaAst> for SelfCompare {
    fn name(&self) -> &'static str {
        "self_compare"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        use cure_engine::kind::BinOp::{Eq, Ne};
        let lang = ctx.lang;
        let NodeData::Binary { op } = lang.data(id) else {
            return None;
        };
        if !matches!(op, Eq | Ne) {
            return None;
        }
        let ch = lang.children(id);
        let (NodeData::VarRef { name: n1 }, NodeData::VarRef { name: n2 }) =
            (lang.data(ch[0]), lang.data(ch[1]))
        else {
            return None;
        };
        if n1 != n2 || !lang.is_exact_int(ch[0]) {
            return None;
        }
        let val = lang.build_bool(*op == Eq);
        Some(Edit::Replace {
            target: id,
            with: val,
        })
    }
}


// ---------------------------------------------------------------------------
// StringBuilder 链还原（反编译器高频产物）：
//   new StringBuilder().append(a).append(b).toString()  →  a + b
//   new StringBuilder("s").append(x).toString()          →  "s" + x
// 安全性：两侧每个实参恰好按序求值一次；null/基本类型转换语义一致
// （append(String.valueOf 语义) == 字符串拼接的 String.valueOf 语义）。
// 首参非 String 类型时前置 ""（`1 + 2` 是整数加法，`"" + 1 + 2` 才是拼接）。
// new StringBuilder(int容量) 形态**不折叠**（容量构造，语义不同）。
// ---------------------------------------------------------------------------

pub struct StringBuilderFold;

impl Rule<JavaAst> for StringBuilderFold {
    fn name(&self) -> &'static str {
        "string_builder_fold"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        // 外层：X.toString()
        if lang.kind(id) != NodeKind::Call {
            return None;
        }
        let callee = lang.children(id)[0];
        if let NodeData::Member { name } = lang.data(callee) {
            if name != "toString" || !lang.children(id)[1..].is_empty() {
                return None;
            }
        } else {
            return None;
        }
        let mut node = lang.children(callee)[0]; // 链头：… .append(...)
        let mut parts: Vec<JavaId> = Vec::new();
        // 沿 append 链下溯
        loop {
            match lang.data(node) {
                NodeData::Call => {
                    let c = lang.children(node);
                    if let NodeData::Member { name } = lang.data(c[0]) {
                        if name == "append" && c.len() == 2 {
                            parts.push(c[1]);
                            node = lang.children(c[0])[0];
                            continue;
                        }
                    }
                    return None;
                }
                NodeData::New { ty, .. } => {
                    let is_sb = matches!(ty, JType::Ref(n) if n == "StringBuilder" || n == "java.lang.StringBuilder" || n.ends_with(".StringBuilder"));
                    if !is_sb {
                        return None;
                    }
                    let args = lang.children(node).to_vec();
                    match args.len() {
                        0 => {} // new StringBuilder()
                        1 => {
                            // 仅 String 内容构造可折叠；int 字面量是容量构造
                            let a = args[0];
                            let stringy = matches!(lang.literal(a), Some(LitRef::Str(_)))
                                || lang
                                    .var_type(a)
                                    .is_some_and(|t| matches!(t, JType::Ref(n) if n == "String"));
                            if !stringy {
                                return None;
                            }
                            parts.push(a);
                        }
                        _ => return None,
                    }
                    break;
                }
                _ => return None,
            }
        }
        if parts.is_empty() {
            // new StringBuilder().toString() → ""
            let empty = lang.build_str("");
            return Some(Edit::Replace {
                target: id,
                with: empty,
            });
        }
        // 逆序收集：parts 从链尾到链头，反转得源顺序
        parts.reverse();
        let first = parts[0];
        let stringy_first = matches!(lang.literal(first), Some(LitRef::Str(_)))
            || lang
                .var_type(first)
                .is_some_and(|t| matches!(t, JType::Ref(n) if n == "String"));
        let mut acc = if stringy_first {
            first
        } else {
            let empty = lang.build_str("");
            lang.build_bin(BinOp::Add, empty, first)
        };
        for &p in &parts[1..] {
            acc = lang.build_bin(BinOp::Add, acc, p);
        }
        Some(Edit::Replace {
            target: id,
            with: acc,
        })
    }
}

// ---------------------------------------------------------------------------
// 装箱-拆箱链还原（反编译器产物）：
//   Integer.valueOf(n).intValue() → n（Long/Boolean/Double/Float/Character/Short/Byte 同理）
//   new Integer(n).intValue()     → n
// 类型守卫：实参必须匹配包装类型的原始形态（valueOf(String) 是解析，语义不同）。
// ---------------------------------------------------------------------------

pub struct BoxUnboxChain;

const BOXERS: &[(&str, &str)] = &[
    ("Integer", "intValue"),
    ("java.lang.Integer", "intValue"),
    ("Long", "longValue"),
    ("java.lang.Long", "longValue"),
    ("Boolean", "booleanValue"),
    ("java.lang.Boolean", "booleanValue"),
    ("Double", "doubleValue"),
    ("java.lang.Double", "doubleValue"),
    ("Float", "floatValue"),
    ("java.lang.Float", "floatValue"),
    ("Character", "charValue"),
    ("java.lang.Character", "charValue"),
    ("Short", "shortValue"),
    ("java.lang.Short", "shortValue"),
    ("Byte", "byteValue"),
    ("java.lang.Byte", "byteValue"),
];

impl Rule<JavaAst> for BoxUnboxChain {
    fn name(&self) -> &'static str {
        "box_unbox_chain"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        // 外层：inner.xxxValue()
        if lang.kind(id) != NodeKind::Call {
            return None;
        }
        let callee = lang.children(id)[0];
        let NodeData::Member { name: unbox } = lang.data(callee) else {
            return None;
        };
        if !lang.children(id)[1..].is_empty() {
            return None;
        }
        let inner = lang.children(callee)[0];
        if lang.kind(inner) != NodeKind::Call {
            return None;
        }
        // 内层：Box.valueOf(arg) 或 new Box(arg)
        let ich = lang.children(inner).to_vec();
        let (box_name, arg): (String, JavaId) = if let NodeData::New { ty, .. } = lang.data(inner)
        {
            let n = match ty {
                JType::Ref(r) => r.clone(),
                _ => return None,
            };
            if ich.len() != 1 {
                return None;
            }
            (n, ich[0])
        } else {
            // valueOf 静态调用：callee 是 Member 或 VarRef
            let icallee = ich.first().copied()?;
            if let NodeData::Member { name } = lang.data(icallee) {
                // Integer.valueOf：recv 是 VarRef(Integer)
                let recv = lang.children(icallee)[0];
                if name != "valueOf" {
                    return None;
                }
                match lang.var_name(recv) {
                    Some(n) => (n.to_string(), *ich.get(1)?),
                    None => return None,
                }
            } else if let NodeData::VarRef { name } = lang.data(icallee) {
                // java.lang.Integer.valueOf 形态在解析里是 member 链；
                // 裸 valueOf( 视为同包静态——保守跳过
                let _ = name;
                return None;
            } else {
                return None;
            }
        };
        let unbox_name = unbox.clone();
        let expected = BOXERS
            .iter()
            .find(|(b, _)| *b == box_name.as_str())
            .map(|(_, u)| *u)?;
        if expected != unbox_name {
            return None;
        }
        // 类型守卫：实参必须是该包装类型对应的原始类型表达式
        // （valueOf(String) 是解析语义，原始类型形态天然排除字符串）。
        let var_is = |t: fn(&JType) -> bool| lang.var_type(arg).is_some_and(t);
        let lit_is_num = lang.literal(arg).is_some_and(|l| {
            matches!(
                l,
                LitRef::Int(_) | LitRef::Long(_) | LitRef::Float(_) | LitRef::Double(_)
            )
        });
        let ok = match box_name.trim_start_matches("java.lang.") {
            "Integer" | "Long" | "Short" | "Byte" => lang.is_exact_int(arg),
            "Double" | "Float" => {
                lit_is_num
                    || var_is(|t| matches!(t, JType::Double | JType::Float | JType::Int | JType::Long))
            }
            "Boolean" => lang.is_bool(arg),
            "Character" => {
                matches!(lang.literal(arg), Some(LitRef::Char(_))) || var_is(|t| *t == JType::Char)
            }
            _ => false,
        };
        if !ok {
            return None;
        }
        Some(Edit::Replace {
            target: id,
            with: arg,
        })
    }
}

// ---------------------------------------------------------------------------
// 迭代器模式还原（反编译器经典产物）：
//   for (Iterator<E> it = c.iterator(); it.hasNext(); ) {
//       E e = it.next();
//       …rest…
//   }
//   →  for (E e : c) { …rest… }
// 安全性：与 for-each 的脱糖完全同构（iterator() 各求值一次、hasNext/next 同序）。
// 守卫：`it` 在 body 其余部分不再被引用。
// ---------------------------------------------------------------------------

pub struct IteratorToForEach;

impl Rule<JavaAst> for IteratorToForEach {
    fn name(&self) -> &'static str {
        "iterator_to_for_each"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        let NodeData::For {
            inits,
            has_cond,
            steps,
        } = lang.data(id)
        else {
            return None;
        };
        if *inits != 1 || !*has_cond || *steps != 0 {
            return None;
        }
        let ch = lang.children(id).to_vec();
        // children: [init, cond, body]
        let (init, cond, body) = (ch[0], ch[1], ch[2]);

        // init: Iterator<…> it = <iterable>.iterator();
        let NodeData::VarDecl { name: it_name, .. } = lang.data(init) else {
            return None;
        };
        let it_name = it_name.clone();
        let init_call = lang.children(init).first().copied()?;
        if lang.kind(init_call) != NodeKind::Call {
            return None;
        }
        let ic = lang.children(init_call).to_vec();
        let icallee = *ic.first()?;
        let NodeData::Member { name: m0 } = lang.data(icallee) else {
            return None;
        };
        if m0 != "iterator" || ic.len() != 1 {
            return None;
        }
        let iterable = lang.children(icallee)[0];
        // iterable 不得引用 it（自引用）
        if subtree_has_var(&*lang, iterable, &it_name) {
            return None;
        }

        // cond: it.hasNext()
        if lang.kind(cond) != NodeKind::Call {
            return None;
        }
        let cc = lang.children(cond).to_vec();
        let ccallee = *cc.first()?;
        let NodeData::Member { name: m1 } = lang.data(ccallee) else {
            return None;
        };
        if m1 != "hasNext" || cc.len() != 1 {
            return None;
        }
        if lang.var_name(lang.children(ccallee)[0]) != Some(it_name.as_str()) {
            return None;
        }

        // body: Block[ VarDecl e = it.next();, rest… ]
        if lang.kind(body) != NodeKind::Block {
            return None;
        }
        let bch = lang.children(body).to_vec();
        let first = *bch.first()?;
        let NodeData::VarDecl { name: e_name, ty } = lang.data(first) else {
            return None;
        };
        let (e_name, ty) = (e_name.clone(), ty.clone());
        let next_call = lang.children(first).first().copied()?;
        if lang.kind(next_call) != NodeKind::Call {
            return None;
        }
        let nc = lang.children(next_call).to_vec();
        let ncallee = *nc.first()?;
        let NodeData::Member { name: m2 } = lang.data(ncallee) else {
            return None;
        };
        if m2 != "next" || nc.len() != 1 {
            return None;
        }
        if lang.var_name(lang.children(ncallee)[0]) != Some(it_name.as_str()) {
            return None;
        }
        // rest 不得引用 it
        let rest = bch[1..].to_vec();
        for &r in &rest {
            if subtree_has_var(&*lang, r, &it_name) {
                return None;
            }
        }
        let new_body = lang.build_block(rest);
        let with = lang.for_each(&e_name, ty, iterable, new_body);
        Some(Edit::Replace {
            target: id,
            with,
        })
    }
}

/// 剥掉 next() 调用外围的 Paren / Cast 包装（DAD：`String s = (String) it.next();`）
fn unwrap_paren_cast(lang: &JavaAst, mut n: JavaId) -> JavaId {
    loop {
        match lang.kind(n) {
            NodeKind::Paren | NodeKind::Cast => {
                let ch = lang.children(n);
                if ch.is_empty() {
                    return n;
                }
                n = ch[0];
            }
            _ => return n,
        }
    }
}

/// 从任意泛型 Ref（ArrayList<String> / List<T>）提取元素类型；裸类型 None。
fn generic_elem_ty(ty: &JType) -> Option<JType> {
    let name = match ty {
        JType::Ref(n) => n.as_str(),
        _ => return None,
    };
    let lt = name.find('<')?;
    let gt = name.rfind('>')?;
    if gt <= lt {
        return None;
    }
    let inner = name[lt + 1..gt].trim();
    if inner.is_empty() || inner.contains('<') {
        return None; // 嵌套泛型/通配符 → Object 兜底
    }
    Some(JType::Ref(inner.to_string()))
}

/// 收集子树内 name 的全部 VarRef。
fn collect_var_refs(lang: &JavaAst, root: JavaId, name: &str, out: &mut Vec<JavaId>) {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if lang.kind(n) == NodeKind::VarRef && lang.var_name(n) == Some(name) {
            out.push(n);
        }
        for &c in lang.children(n) {
            stack.push(c);
        }
    }
}

/// 在 root 子树内找 target 的直接父节点。
fn parent_of_recv(lang: &JavaAst, root: JavaId, target: JavaId) -> JavaId {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        for &c in lang.children(n) {
            if c == target {
                return n;
            }
            stack.push(c);
        }
    }
    root
}

fn subtree_has_var(lang: &JavaAst, id: JavaId, name: &str) -> bool {
    let mut stack = vec![id];
    while let Some(n) = stack.pop() {
        if lang.kind(n) == NodeKind::VarRef && lang.var_name(n) == Some(name) {
            return true;
        }
        for &c in lang.children(n) {
            stack.push(c);
        }
    }
    false
}

// ---------------------------------------------------------------------------
// new String 折叠（混淆器/反编译器产物）：new String("lit") → "lit"、new String() → ""
// 仅字面量实参（new String(charArray/bytes) 是拷贝语义，不折）。
// ---------------------------------------------------------------------------

pub struct NewStringFold;

impl Rule<JavaAst> for NewStringFold {
    fn name(&self) -> &'static str {
        "new_string_fold"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        let NodeData::New { ty, .. } = lang.data(id) else {
            return None;
        };
        if !matches!(ty, JType::Ref(n) if n == "String" || n == "java.lang.String" || n.ends_with(".String")) {
            return None;
        }
        let args = lang.children(id).to_vec();
        match args.len() {
            0 => {
                let empty = lang.build_str("");
                Some(Edit::Replace {
                    target: id,
                    with: empty,
                })
            }
            1 => {
                if matches!(lang.literal(args[0]), Some(LitRef::Str(_))) {
                    Some(Edit::Replace {
                        target: id,
                        with: args[0],
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}


// ---------------------------------------------------------------------------
// 循环头断路还原（jadx/反编译器高频形态）：
//   while (true) { if (c) { break; } REST }        →  while (!c) { REST }
//   while (true) { if (c) break; else { REST } … }  →  while (!c) { REST… }
// 安全性：c 在两种形态下都于每次迭代头部求值一次；break 无标签（目标是本循环）。
// ---------------------------------------------------------------------------

pub struct LoopHeadBreak;

impl Rule<JavaAst> for LoopHeadBreak {
    fn name(&self) -> &'static str {
        "loop_head_break"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        let body = match lang.kind(id) {
            NodeKind::While => {
                let ch = lang.children(id).to_vec();
                if !matches!(lang.literal(ch[0]), Some(LitRef::Bool(true))) {
                    return None;
                }
                ch[1]
            }
            NodeKind::DoWhile => {
                let ch = lang.children(id).to_vec();
                if !matches!(lang.literal(ch[1]), Some(LitRef::Bool(true))) {
                    return None;
                }
                ch[0]
            }
            _ => return None,
        };
        if lang.kind(body) != NodeKind::Block {
            return None;
        }
        let bch = lang.children(body).to_vec();
        let is_do = lang.kind(id) == NodeKind::DoWhile;
        // do-while 形态：if 必须是体里唯一语句
        if is_do && bch.len() != 1 {
            return None;
        }
        let first = *bch.first()?;
        if lang.kind(first) != NodeKind::If {
            return None;
        }
        let ich = lang.children(first).to_vec();
        let c = ich[0];

        let bare_break = |n: JavaId| matches!(lang.data(n), NodeData::Break { label: None });
        let is_break = |n: JavaId| match lang.kind(n) {
            NodeKind::Break => bare_break(n),
            NodeKind::Block if lang.children(n).len() == 1 => bare_break(lang.children(n)[0]),
            _ => false,
        };
        let bare_continue = |n: JavaId| matches!(lang.data(n), NodeData::Continue { label: None });
        let ends_with_continue = |n: JavaId| -> bool {
            match lang.kind(n) {
                NodeKind::Continue => bare_continue(n),
                NodeKind::Block => lang
                    .children(n)
                    .last()
                    .is_some_and(|&l| lang.kind(l) == NodeKind::Continue && bare_continue(l)),
                _ => false,
            }
        };
        let strip_tail_continue = |n: JavaId| -> Vec<JavaId> {
            match lang.kind(n) {
                NodeKind::Continue => vec![],
                NodeKind::Block => {
                    let ch = lang.children(n).to_vec();
                    if ch
                        .last()
                        .is_some_and(|&l| lang.kind(l) == NodeKind::Continue && bare_continue(l))
                    {
                        ch[..ch.len() - 1].to_vec()
                    } else {
                        ch
                    }
                }
                _ => vec![n],
            }
        };

        match ich.len() {
            2 => {
                // while (true) { if (c) { break; } REST } → while (!c) { REST }
                if is_do || !is_break(ich[1]) {
                    return None;
                }
                let mut rest: Vec<JavaId> = bch[1..].to_vec();
                if let Some(&last) = rest.last() {
                    if lang.kind(last) == NodeKind::Continue && bare_continue(last) {
                        rest.pop();
                    }
                }
                let nc = lang.build_unary(UnOp::Not, c);
                let inner = lang.build_block(rest);
                let w = lang.while_(nc, inner);
                Some(Edit::Replace {
                    target: id,
                    with: w,
                })
            }
            3 => {
                let (then, els) = (ich[1], ich[2]);
                // 形态 B/C：then 以 continue 结尾、else 是 break → while (c) { then' }
                if ends_with_continue(then) && is_break(els) {
                    let rest = strip_tail_continue(then);
                    let inner = lang.build_block(rest);
                    let w = lang.while_(c, inner);
                    return Some(Edit::Replace {
                        target: id,
                        with: w,
                    });
                }
                // 形态 D：then 是 break、else 以 continue 结尾 → while (!c) { else' }
                if is_break(then) && ends_with_continue(els) {
                    let rest = strip_tail_continue(els);
                    let inner = lang.build_block(rest);
                    let nc = lang.build_unary(UnOp::Not, c);
                    let w = lang.while_(nc, inner);
                    return Some(Edit::Replace {
                        target: id,
                        with: w,
                    });
                }
                // 既有形态：while (true) { if (c) break; else { REST } … }
                if !is_do && is_break(then) {
                    let mut all = strip_tail_continue(els);
                    all.extend(bch[1..].iter().copied());
                    let inner = lang.build_block(all);
                    let nc = lang.build_unary(UnOp::Not, c);
                    let w = lang.while_(nc, inner);
                    return Some(Edit::Replace {
                        target: id,
                        with: w,
                    });
                }
                None
            }
            _ => None,
        }
    }
}

pub struct WhileIteratorToForEach;

impl Rule<JavaAst> for WhileIteratorToForEach {
    fn name(&self) -> &'static str {
        "while_iterator_to_for_each"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        // 先取结构信息（避免 lang 的可变借用与 ctx 方法冲突）
        let parent = ctx.parent(id)?;
        let idx = ctx.index(id)?;
        let lang = ctx.lang;
        if std::env::var("CURE_DBG").is_ok() {
            eprintln!("[cff] check called: {:?}", lang.kind(id));
        }
        if lang.kind(id) != NodeKind::While {
            return None;
        }
        let wch = lang.children(id).to_vec();
        let (cond, body) = (wch[0], wch[1]);
        // 前置声明：Iterator<…> it = <iterable>.iterator();
        if lang.kind(parent) != NodeKind::Block {
            return None;
        }
        if idx == 0 {
            return None;
        }
        let stmts = lang.children(parent).to_vec();
        let decl = stmts[idx - 1];
        if lang.kind(decl) != NodeKind::VarDecl {
            return None;
        }
        let it_name = lang.var_name(decl)?.to_string();
        let init_call = lang.children(decl).first().copied()?;
        if lang.kind(init_call) != NodeKind::Call {
            return None;
        }
        let ic = lang.children(init_call).to_vec();
        let NodeData::Member { name: m0 } = lang.data(*ic.first()?) else {
            return None;
        };
        if m0 != "iterator" || ic.len() != 1 {
            return None;
        }
        let iterable = lang.children(ic[0])[0];
        if subtree_has_var(&*lang, iterable, &it_name) {
            return None;
        }
        // cond: it.hasNext()
        if lang.kind(cond) != NodeKind::Call {
            return None;
        }
        let cc = lang.children(cond).to_vec();
        let NodeData::Member { name: m1 } = lang.data(*cc.first()?) else {
            return None;
        };
        if m1 != "hasNext" || cc.len() != 1 {
            return None;
        }
        if lang.var_name(lang.children(cc[0])[0]) != Some(it_name.as_str()) {
            return None;
        }
        // body: Block[ VarDecl e = it.next();, REST… ]
        if lang.kind(body) != NodeKind::Block {
            return None;
        }
        let bch = lang.children(body).to_vec();
        let first = *bch.first()?;
        if lang.kind(first) != NodeKind::VarDecl {
            // ===== 内联形态：body 中恰好一次 it.next()（可能 Cast/Paren 包裹）=====
            // it 在体内只能出现这一次（hasNext 已在 cond 消费）
            let it_refs_in_body: Vec<JavaId> = {
                let mut v = Vec::new();
                collect_var_refs(&*lang, body, &it_name, &mut v);
                v
            };
            if it_refs_in_body.len() != 1 {
                return None;
            }
            let next_recv = it_refs_in_body[0];
            // receiver 的父是 Member{next}，Member 的父才是 Call（无实参）
            let member = parent_of_recv(lang, body, next_recv);
            if lang.kind(member) != NodeKind::Member {
                return None;
            }
            let NodeData::Member { name: mn } = lang.data(member) else {
                return None;
            };
            if mn != "next" {
                return None;
            }
            let next_call = parent_of_recv(lang, body, member);
            if lang.kind(next_call) != NodeKind::Call {
                return None;
            }
            if lang.children(next_call).len() != 1 {
                return None;
            }
            // 包裹节点（要被替换的）：从 next_call 向上收集 Cast/Paren
            let mut wrapped = next_call;
            loop {
                let par = parent_of_recv(lang, body, wrapped);
                match lang.kind(par) {
                    NodeKind::Cast | NodeKind::Paren => wrapped = par,
                    _ => break,
                }
            }
            // 循环变量类型：从可迭代对象的声明类型推导（ArrayList<String>→String，
            // 裸类型→Object）。元素为 Object 时【保留 Cast】（裸集合上
            // for (String e : raw) 非法，必须 for (Object e) + (String) e）
            let elem = match lang.var_type(iterable) {
                Some(t) => generic_elem_ty(t).unwrap_or(JType::Ref("Object".into())),
                None => JType::Ref("Object".into()),
            };
            let is_object = matches!(&elem, JType::Ref(n) if n == "Object");
            let (replace_target, e_ty) = match lang.data(wrapped) {
                NodeData::Cast { .. } if is_object => (next_call, elem),
                NodeData::Cast { ty } => (wrapped, if is_object { ty.clone() } else { elem }),
                _ => (wrapped, elem),
            };
            // 新变量名（不与体内现有变量冲突）
            let e_name = if subtree_has_var(&*lang, body, "e") {
                "e2"
            } else {
                "e"
            };
            let e_ref = lang.var(e_name);
            let foreach = lang.for_each(e_name, e_ty, iterable, body);
            return Some(Edit::Multi(vec![
                Edit::Replace {
                    target: replace_target,
                    with: e_ref,
                },
                Edit::Splice {
                    node: parent,
                    index: idx - 1,
                    remove: 2,
                    insert: vec![foreach],
                },
            ]));
        }
        let NodeData::VarDecl { name: e_name, ty } = lang.data(first) else {
            return None;
        };
        let (e_name, ty) = (e_name.clone(), ty.clone());
        let next_call = lang.children(first).first().copied()?;
        let next_call = unwrap_paren_cast(&*lang, next_call);
        if lang.kind(next_call) != NodeKind::Call {
            return None;
        }
        let nc = lang.children(next_call).to_vec();
        let NodeData::Member { name: m2 } = lang.data(*nc.first()?) else {
            return None;
        };
        if m2 != "next" || nc.len() != 1 {
            return None;
        }
        if lang.var_name(lang.children(nc[0])[0]) != Some(it_name.as_str()) {
            return None;
        }
        // 裸集合（元素 Object）上 for (更窄类型 e : raw) 非法 → 保守拒绝
        let elem_is_object = match lang.var_type(iterable) {
            Some(it_t) => generic_elem_ty(it_t).is_none(),
            None => true,
        };
        if elem_is_object && ty != JType::Ref("Object".into()) {
            return None;
        }
        let rest = bch[1..].to_vec();
        for &r in &rest {
            if subtree_has_var(&*lang, r, &it_name) {
                return None;
            }
        }
        let new_body = lang.build_block(rest);
        let foreach = lang.for_each(&e_name, ty, iterable, new_body);
        // 用 for-each 同时替换 [decl, while] 两条语句
        Some(Edit::Splice {
            node: parent,
            index: idx - 1,
            remove: 2,
            insert: vec![foreach],
        })
    }
}


// ---------------------------------------------------------------------------
// 拼接中的 String.valueOf 剥离（混淆器/反编译器包装）：
//   String.valueOf(x) + y  →  x + y（y 为 String 字面量/已知 String 时）
//   y + String.valueOf(x)  →  y + x（同上）
// 安全性：valueOf 的字符串化语义与 + 拼接的隐式转换一致（null → "null"）；
// 守卫：另一侧必须可证为 String（保证 + 是拼接而非数值加法）。
// ---------------------------------------------------------------------------

pub struct ConcatValueOfDrop;

impl Rule<JavaAst> for ConcatValueOfDrop {
    fn name(&self) -> &'static str {
        "concat_value_of_drop"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        if lang.kind(id) != NodeKind::Binary {
            return None;
        }
        if lang.bin_op(id) != Some(BinOp::Add) {
            return None;
        }
        let ch = lang.children(id).to_vec();
        let (l, r) = (ch[0], ch[1]);
        let is_value_of = |n: JavaId| -> Option<JavaId> {
            if lang.kind(n) != NodeKind::Call {
                return None;
            }
            let c = lang.children(n).to_vec();
            if c.len() != 2 {
                return None;
            }
            let callee = c[0];
            if let NodeData::Member { name } = lang.data(callee) {
                if name == "valueOf" && lang.var_name(lang.children(callee)[0]) == Some("String") {
                    return Some(c[1]);
                }
            }
            if let NodeData::VarRef { name } = lang.data(callee) {
                if name == "valueOf" {
                    return Some(c[1]);
                }
            }
            None
        };
        let stringy = |n: JavaId| -> bool {
            matches!(lang.literal(n), Some(LitRef::Str(_)))
                || lang
                    .var_type(n)
                    .is_some_and(|t| matches!(t, JType::Ref(n) if n == "String"))
        };
        // String.valueOf(x) + y（y 可证 String）
        if let Some(x) = is_value_of(l) {
            if stringy(r) {
                let with = lang.build_bin(BinOp::Add, x, r);
                return Some(Edit::Replace {
                    target: id,
                    with,
                });
            }
        }
        // y + String.valueOf(x)
        if let Some(x) = is_value_of(r) {
            if stringy(l) {
                let with = lang.build_bin(BinOp::Add, l, x);
                return Some(Edit::Replace {
                    target: id,
                    with,
                });
            }
        }
        None
    }
}


// ---------------------------------------------------------------------------
// 语句级 StringBuilder 链还原（ddc/jcdc 经典产物）：
//   StringBuilder v0 = new StringBuilder().append(a0);
//   v0 = v0.append(a1);              // 重赋值形态
//   StringBuilder v1 = v0.append(a2); // 新变量形态
//   String s = v1.toString();
//   →  String s = <a0 + a1 + a2>;
// 守卫：链上每个变量的读次数恰为 1（仅作下一步 receiver），
// 首参非 String 时前置 ""（与表达式版一致）；容量构造不折叠。
// 语义：各实参按序求值一次，两种形态一致。
// ---------------------------------------------------------------------------

pub struct StringBuilderStatements;

fn count_var_uses_in(lang: &JavaAst, root: JavaId, name: &str) -> usize {
    // 语义上是【读】的 VarRef 计数：简单赋值的 target 是纯写（不计）；
    // 复合赋值（x += 1）与 inc/dec 的 target 是读+写（计）。
    let mut n = 0;
    let mut stack = vec![root];
    while let Some(x) = stack.pop() {
        if lang.kind(x) == NodeKind::Assign && lang.assign_op(x).is_none() {
            let ch = lang.children(x).to_vec();
            if let Some(&t) = ch.first() {
                let is_named_target =
                    lang.kind(t) == NodeKind::VarRef && lang.var_name(t) == Some(name);
                if !is_named_target {
                    stack.push(t);
                }
                for &c in &ch[1..] {
                    stack.push(c);
                }
            }
            continue;
        }
        if lang.kind(x) == NodeKind::VarRef && lang.var_name(x) == Some(name) {
            n += 1;
        }
        for &c in lang.children(x) {
            stack.push(c);
        }
    }
    n
}

impl Rule<JavaAst> for StringBuilderStatements {
    fn name(&self) -> &'static str {
        "string_builder_statements"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        // 取结构信息（避免与 ctx 方法借用冲突）
        let parent = ctx.parent(id)?;
        let idx = ctx.index(id)?;
        let lang = ctx.lang;

        if lang.kind(id) != NodeKind::VarDecl {
            return None;
        }
        if lang.kind(parent) != NodeKind::Block {
            return None;
        }
        let v0 = lang.var_name(id)?.to_string();
        // init: new SB().append(a0)
        let init = *lang.children(id).first()?;
        if lang.kind(init) != NodeKind::Call {
            return None;
        }
        let ic = lang.children(init).to_vec();
        if ic.len() != 2 {
            return None;
        }
        let NodeData::Member { name: m0 } = lang.data(ic[0]) else {
            return None;
        };
        if m0 != "append" {
            return None;
        }
        let sb_new = lang.children(ic[0])[0];
        let (ctor_arg, a0) = match lang.data(sb_new) {
            NodeData::New { ty, .. } => {
                let is_sb = matches!(ty, JType::Ref(n) if n == "StringBuilder" || n == "java.lang.StringBuilder" || n.ends_with(".StringBuilder"));
                if !is_sb {
                    return None;
                }
                let args = lang.children(sb_new).to_vec();
                match args.len() {
                    0 => (None, ic[1]),
                    1 => {
                        let a = args[0];
                        let stringy = matches!(lang.literal(a), Some(LitRef::Str(_)))
                            || lang
                                .var_type(a)
                                .is_some_and(|t| matches!(t, JType::Ref(n) if n == "String"));
                        if !stringy {
                            return None; // 容量构造
                        }
                        (Some(a), ic[1])
                    }
                    _ => return None,
                }
            }
            _ => return None,
        };

        let stmts = lang.children(parent).to_vec();
        // 沿链前进：收集 append 实参与链语句
        let mut parts: Vec<JavaId> = vec![a0];
        let mut expected_reads: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut chain_len = 1usize; // 已消费 v0 的 decl
        let mut cur = v0.clone();
        while idx + chain_len < stmts.len() {
            let s = stmts[idx + chain_len];
            let (recv, arg, new_var, consumed) = match lang.data(s) {
                // v = v.append(arg)（重赋值；ExprStmt 包裹或裸）
                NodeData::Assign { op: None } => {
                    let ch = lang.children(s).to_vec();
                    if ch.len() != 2 {
                        break;
                    }
                    let (t, v) = (ch[0], ch[1]);
                    if lang.kind(t) != NodeKind::VarRef
                        || lang.var_name(t) != Some(cur.as_str())
                        || lang.kind(v) != NodeKind::Call
                    {
                        break;
                    }
                    let vc = lang.children(v).to_vec();
                    if vc.len() != 2 {
                        break;
                    }
                    let NodeData::Member { name: mn } = lang.data(vc[0]) else {
                        break;
                    };
                    if mn != "append" {
                        break;
                    }
                    let recv = lang.children(vc[0])[0];
                    (recv, vc[1], false, s)
                }
                NodeData::ExprStmt => {
                    let ch = lang.children(s).to_vec();
                    if ch.len() != 1 {
                        break;
                    }
                    let inner = ch[0];
                    if !matches!(lang.data(inner), NodeData::Assign { op: None }) {
                        break;
                    }
                    let ach = lang.children(inner).to_vec();
                    if ach.len() != 2 {
                        break;
                    }
                    let (t, v) = (ach[0], ach[1]);
                    if lang.kind(t) != NodeKind::VarRef
                        || lang.var_name(t) != Some(cur.as_str())
                        || lang.kind(v) != NodeKind::Call
                    {
                        break;
                    }
                    let vc = lang.children(v).to_vec();
                    if vc.len() != 2 {
                        break;
                    }
                    let NodeData::Member { name: mn } = lang.data(vc[0]) else {
                        break;
                    };
                    if mn != "append" {
                        break;
                    }
                    let recv = lang.children(vc[0])[0];
                    (recv, vc[1], false, s)
                }
                // StringBuilder v2 = v.append(arg)（新变量）
                NodeData::VarDecl { .. } => {
                    let init = match lang.children(s).first() {
                        Some(&x) => x,
                        None => break,
                    };
                    if lang.kind(init) != NodeKind::Call {
                        break;
                    }
                    let vc = lang.children(init).to_vec();
                    if vc.len() != 2 {
                        break;
                    }
                    let NodeData::Member { name: mn } = lang.data(vc[0]) else {
                        break;
                    };
                    if mn != "append" {
                        break;
                    }
                    let recv = lang.children(vc[0])[0];
                    if lang.var_name(recv) != Some(cur.as_str()) {
                        break;
                    }
                    (recv, vc[1], true, s)
                }
                _ => break,
            };
            // receiver 必须是当前变量
            if lang.var_name(recv) != Some(cur.as_str()) {
                break;
            }
            parts.push(arg);
            *expected_reads.entry(cur.clone()).or_insert(0) += 1;
            if new_var {
                cur = lang.var_name(consumed)?.to_string();
            }
            chain_len += 1;
            let _ = consumed;
        }

        // 链后必须紧跟：String s = <cur>.toString();
        let final_stmt = *stmts.get(idx + chain_len)?;
        let NodeData::VarDecl { .. } = lang.data(final_stmt) else {
            return None;
        };
        let s_init = *lang.children(final_stmt).first()?;
        if lang.kind(s_init) != NodeKind::Call {
            return None;
        }
        let sc = lang.children(s_init).to_vec();
        if sc.len() != 1 {
            return None;
        }
        let NodeData::Member { name: mn } = lang.data(sc[0]) else {
            return None;
        };
        if mn != "toString" {
            return None;
        }
        if lang.var_name(lang.children(sc[0])[0]) != Some(cur.as_str()) {
            return None;
        }

        // 读次数守卫：实际读数 == 链内预期读数（receiver 次数 + toString 1 次），
        // 差值即存在链外使用 → 拒绝
        *expected_reads.entry(cur.clone()).or_insert(0) += 1; // toString receiver
        for (v, exp) in &expected_reads {
            if count_var_uses_in(&*lang, parent, v) != *exp {
                return None;
            }
        }

        // 构造拼接表达式
        let mut all_parts: Vec<JavaId> = Vec::new();
        if let Some(ca) = ctor_arg {
            all_parts.push(ca);
        }
        all_parts.extend(parts);
        if all_parts.is_empty() {
            let empty = lang.build_str("");
            return Some(Edit::Multi(vec![
                Edit::Replace {
                    target: s_init,
                    with: empty,
                },
                Edit::Splice {
                    node: parent,
                    index: idx,
                    remove: chain_len,
                    insert: Vec::new(),
                },
            ]));
        }
        let first = all_parts[0];
        let stringy_first = matches!(lang.literal(first), Some(LitRef::Str(_)))
            || lang
                .var_type(first)
                .is_some_and(|t| matches!(t, JType::Ref(n) if n == "String"));
        let mut acc = if stringy_first {
            first
        } else {
            let empty = lang.build_str("");
            lang.build_bin(BinOp::Add, empty, first)
        };
        for &p in &all_parts[1..] {
            acc = lang.build_bin(BinOp::Add, acc, p);
        }
        Some(Edit::Multi(vec![
            Edit::Replace {
                target: s_init,
                with: acc,
            },
            Edit::Splice {
                node: parent,
                index: idx,
                remove: chain_len,
                insert: Vec::new(),
            },
        ]))
    }
}


// ---------------------------------------------------------------------------
// 尾部 continue 删除（标签感知版，从引擎移入）：
//   循环体最后一条 continue：无标签，或标签即本循环 → 与落入下一轮等价，删除。
//   标签指向外层循环 → 语义不同，保留。
// ---------------------------------------------------------------------------

pub struct TrailingContinueJava;

impl Rule<JavaAst> for TrailingContinueJava {
    fn name(&self) -> &'static str {
        "trailing_continue"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let parent = ctx.parent(id)?;
        let lang = ctx.lang;
        let body = match lang.kind(id) {
            NodeKind::While | NodeKind::For | NodeKind::DoWhile | NodeKind::ForEach => {
                *lang.children(id).last()?
            }
            _ => return None,
        };
        if lang.kind(body) != NodeKind::Block {
            return None;
        }
        let ch = lang.children(body).to_vec();
        let last = *ch.last()?;
        if lang.kind(last) != NodeKind::Continue {
            return None;
        }
        let label = match lang.data(last) {
            NodeData::Continue { label } => label.clone(),
            _ => return None,
        };
        match label {
            None => Some(Edit::Delete { node: last }),
            Some(l) => {
                // 标签必须命名本循环：父链上是 Label{l} 包着本循环
                if lang.kind(parent) == NodeKind::Label {
                    if let NodeData::Label { name } = lang.data(parent) {
                        if *name == l {
                            return Some(Edit::Delete { node: last });
                        }
                    }
                }
                None
            }
        }
    }
}


// ---------------------------------------------------------------------------
// 异或噪声消除（Java 语义版）：Java 中 ^ 的操作数必为整型，
// 故无需 is_exact_int 类型证明即可折叠——覆盖含副作用/未知类型的场景：
//   (mark(5) ^ 0x5A) ^ 0x5A → mark(5)（调用保留、位置不变）
//   x ^ 0 / 0 ^ x → x
// ---------------------------------------------------------------------------

pub struct XorNoise;

impl Rule<JavaAst> for XorNoise {
    fn name(&self) -> &'static str {
        "xor_noise"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        if lang.kind(id) != NodeKind::Binary {
            return None;
        }
        let op = lang.bin_op(id)?;
        if op != BinOp::BitXor {
            return None;
        }
        let ch = lang.children(id).to_vec();
        let (l, r) = (ch[0], ch[1]);
        // x ^ 0 / 0 ^ x → x（x 任意：求值位置与次数都不变）
        let int_lit = |n: JavaId| lang.literal(n).and_then(|x| x.as_int());
        if int_lit(r) == Some(0) {
            return Some(Edit::Replace {
                target: id,
                with: l,
            });
        }
        if int_lit(l) == Some(0) {
            return Some(Edit::Replace {
                target: id,
                with: r,
            });
        }
        // (x ^ K1) ^ K2 → x ^ (K1^K2)；K1^K2 == 0 → x（x 任意，保留原位）
        if lang.kind(l) != NodeKind::Binary || lang.bin_op(l) != Some(BinOp::BitXor) {
            return None;
        }
        let ich = lang.children(l).to_vec();
        let k1 = lang.literal(ich[1]).and_then(|x| x.as_int())?;
        let k2 = int_lit(r)?;
        let x = ich[0];
        let k = k1 ^ k2;
        let with = if k == 0 {
            x
        } else {
            let lit = lang.build_int(k, false);
            lang.build_bin(BinOp::BitXor, x, lit)
        };
        Some(Edit::Replace {
            target: id,
            with,
        })
    }
}

// ---------------------------------------------------------------------------
// 字面量 length 折叠："abc".length() → 3（字符串字面量长度编译期已知）。
// ---------------------------------------------------------------------------

pub struct StrLenFold;

impl Rule<JavaAst> for StrLenFold {
    fn name(&self) -> &'static str {
        "str_len_fold"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        if lang.kind(id) != NodeKind::Call {
            return None;
        }
        let ch = lang.children(id).to_vec();
        if ch.len() != 1 {
            return None;
        }
        let NodeData::Member { name } = lang.data(ch[0]) else {
            return None;
        };
        if name != "length" {
            return None;
        }
        let recv = lang.children(ch[0])[0];
        if let Some(LitRef::Str(s)) = lang.literal(recv) {
            let with = lang.build_int(s.chars().count() as i64, false);
            return Some(Edit::Replace {
                target: id,
                with,
            });
        }
        None
    }
}


// ---------------------------------------------------------------------------
// 部分求值器（"虚拟执行"）：纯 JDK 方法 + 全字面量实参 → 编译期求值。
// 只在**求值成功**时折叠（失败/越界/异常路径保持原样交给运行时）。
// 覆盖混淆器常用的字符串藏匿手段：
//   "HelloWorld".substring(0,5) / .indexOf / .replace / .trim / .startsWith…
//   Integer.parseInt("42") / String.format("%s=%d",…) / String.valueOf
//   Math.abs/max/min / Character.isXxx / (char)('a'+2)
//   {"a","b"}[1]（字面量数组下标） / Base64 解码链 new String(decoder.decode("…"))
// 语义安全边界：不折 toUpperCase/toLowerCase（locale 敏感）、
// parseInt 解析失败不折、substring 越界不折。
// ---------------------------------------------------------------------------

pub struct LiteralEval;

impl Rule<JavaAst> for LiteralEval {
    fn name(&self) -> &'static str {
        "literal_eval"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        match lang.kind(id) {
            NodeKind::Call => self.eval_call(lang, id),
            NodeKind::Index => self.eval_index(lang, id),
            NodeKind::Cast => eval_cast_literal(lang, id),
            _ => None,
        }
    }
}

impl LiteralEval {
    fn eval_call(&self, lang: &mut JavaAst, id: JavaId) -> Option<Edit<JavaAst>> {
        let ch = lang.children(id).to_vec();
        let callee = ch[0];
        let NodeData::Member { name: method } = lang.data(callee) else {
            return None;
        };
        let method = method.clone();
        let recv = lang.children(callee)[0];
        // 实参必须全部为字面量
        let args: Vec<Lit> = ch[1..]
            .iter()
            .map(|&a| litref_to_lit(lang.literal(a)))
            .collect::<Option<Vec<_>>>()?;

        let result: Option<Lit> = if let Some(LitRef::Str(s)) = lang.literal(recv) {
            eval_str_method(s, &method, &args)
        } else if let Some(cls) = lang.var_name(recv) {
            eval_static_method(cls, &method, &args)
        } else {
            None
        };
        let lit = result?;
        let with = lit_to_node(lang, &lit);
        Some(Edit::Replace {
            target: id,
            with,
        })
    }

    fn eval_index(&self, lang: &mut JavaAst, id: JavaId) -> Option<Edit<JavaAst>> {
        // {"a","b"}[1] → "b"（字面量数组 + 字面量下标，界内）
        let ch = lang.children(id).to_vec();
        if lang.kind(ch[0]) != NodeKind::ArrayLit {
            return None;
        }
        let idx = lang.literal(ch[1])?.as_int()?;
        let elems = lang.children(ch[0]).to_vec();
        if idx < 0 || idx >= elems.len() as i64 {
            return None;
        }
        Some(Edit::Replace {
            target: id,
            with: elems[idx as usize],
        })
    }
}

fn litref_to_lit(l: Option<LitRef<'_>>) -> Option<Lit> {
    Some(match l? {
        LitRef::Str(s) => Lit::Str(s.to_string()),
        LitRef::Int(v) => Lit::Int(v),
        LitRef::Long(v) => Lit::Long(v),
        LitRef::Bool(b) => Lit::Bool(b),
        LitRef::Char(c) => Lit::Char(c),
        LitRef::Float(v) => Lit::Float(v),
        LitRef::Double(v) => Lit::Double(v),
        LitRef::Null => return None,
    })
}

fn lit_to_node(lang: &mut JavaAst, lit: &Lit) -> JavaId {
    match lit {
        Lit::Str(s) => lang.build_str(s),
        Lit::Int(v) => lang.build_int(*v, false),
        Lit::Long(v) => lang.build_int(*v, true),
        Lit::Bool(b) => lang.build_bool(*b),
        Lit::Char(c) => lang.build_char(*c),
        Lit::Float(v) => lang.lit(Lit::Float(*v)),
        Lit::Double(v) => lang.lit(Lit::Double(*v)),
        _ => lang.lit(Lit::Null),
    }
}

/// 字面量 cast 折叠：(char)('a'+1) → 'b、(int)'a' → 97、窄化仅在值域内。
fn eval_cast_literal(lang: &mut JavaAst, id: JavaId) -> Option<Edit<JavaAst>> {
    let NodeData::Cast { ty } = lang.data(id) else {
        return None;
    };
    let ty = ty.clone();
    let inner = lang.children(id)[0];
    let lit = litref_to_lit(lang.literal(inner))?;
    let with = match (&ty, &lit) {
        (JType::Char, Lit::Int(v)) if (0 as i64..=0xFFFF).contains(v) => {
            lang.build_char(char::from_u32(*v as u32)?)
        }
        (JType::Char, Lit::Char(_)) => inner,
        (JType::Int, Lit::Char(c)) => lang.build_int(*c as i64, false),
        (JType::Long, Lit::Char(c)) => lang.build_int(*c as i64, true),
        (JType::Long, Lit::Int(v)) => lang.build_int(*v, true),
        (JType::Int, Lit::Long(v)) if *v >= i32::MIN as i64 && *v <= i32::MAX as i64 => {
            lang.build_int(*v, false)
        }
        (JType::Int, Lit::Int(_)) => inner,
        _ => return None,
    };
    Some(Edit::Replace {
        target: id,
        with,
    })
}

fn eval_str_method(s: &str, method: &str, args: &[Lit]) -> Option<Lit> {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len() as i64;
    Some(match (method, args) {
        ("isEmpty", []) => Lit::Bool(s.is_empty()),
        ("trim", []) => Lit::Str(s.trim_matches(|c: char| c <= ' ').to_string()),
        ("concat", [Lit::Str(b)]) => Lit::Str(format!("{s}{b}")),
        ("startsWith", [Lit::Str(p)]) => Lit::Bool(s.starts_with(p.as_str())),
        ("endsWith", [Lit::Str(p)]) => Lit::Bool(s.ends_with(p.as_str())),
        ("contains", [Lit::Str(p)]) => Lit::Bool(s.contains(p.as_str())),
        ("equals", [Lit::Str(p)]) => Lit::Bool(s == p),
        ("equalsIgnoreCase", [Lit::Str(p)]) => {
            Lit::Bool(s.to_lowercase() == p.to_lowercase())
        }
        ("indexOf", [Lit::Char(c)]) => Lit::Int(
            s.find(*c).map(|i| i as i64).unwrap_or(-1),
        ),
        ("indexOf", [Lit::Str(p)]) => Lit::Int(
            s.find(p.as_str()).map(|i| i as i64).unwrap_or(-1),
        ),
        ("length", []) => Lit::Int(n),
        ("charAt", [Lit::Int(i)]) if *i >= 0 && *i < n => {
            Lit::Char(chars[*i as usize])
        }
        ("substring", [Lit::Int(b)]) if *b >= 0 && *b <= n => {
            Lit::Str(s.chars().skip(*b as usize).collect())
        }
        ("substring", [Lit::Int(b), Lit::Int(e)]) if *b >= 0 && *b <= *e && *e <= n => {
            Lit::Str(chars[*b as usize..*e as usize].iter().collect())
        }
        ("replace", [Lit::Char(a), Lit::Char(b)]) => {
            Lit::Str(s.chars().map(|c| if c == *a { *b } else { c }).collect())
        }
        ("replace", [Lit::Str(a), Lit::Str(b)]) if !a.is_empty() => {
            Lit::Str(s.replace(a.as_str(), b))
        }
        // String.hashCode 是 JLS 规定的确定性算法
        ("hashCode", []) => Lit::Int(
            s.chars()
                .fold(0i32, |h, c| h.wrapping_mul(31).wrapping_add(c as i32))
                as i64,
        ),
        _ => return None,
    })
}

fn eval_static_method(cls: &str, method: &str, args: &[Lit]) -> Option<Lit> {
    let cls = cls.rsplit('.').next().unwrap_or(cls);
    Some(match (cls, method, args) {
        ("Integer", "parseInt", [Lit::Str(s)]) => {
            Lit::Int(s.trim().parse::<i64>().ok()? as i64)
        }
        ("Integer", "toString", [Lit::Int(v)]) => Lit::Str(v.to_string()),
        ("Integer", "toString", [Lit::Long(v)]) => Lit::Str(v.to_string()),
        ("Long", "parseLong", [Lit::Str(s)]) => Lit::Long(s.trim().parse::<i64>().ok()?),
        ("Long", "toString", [Lit::Int(v)]) => Lit::Str(v.to_string()),
        ("Long", "toString", [Lit::Long(v)]) => Lit::Str(v.to_string()),
        ("Boolean", "parseBoolean", [Lit::Str(s)]) => {
            Lit::Bool(s.eq_ignore_ascii_case("true"))
        }
        ("Boolean", "toString", [Lit::Bool(b)]) => Lit::Str(b.to_string()),
        ("String", "valueOf", [Lit::Str(s)]) => Lit::Str(s.clone()),
        ("String", "valueOf", [Lit::Int(v)]) => Lit::Str(v.to_string()),
        ("String", "valueOf", [Lit::Long(v)]) => Lit::Str(v.to_string()),
        ("String", "valueOf", [Lit::Char(c)]) => Lit::Str(c.to_string()),
        ("String", "valueOf", [Lit::Bool(b)]) => Lit::Str(b.to_string()),
        ("String", "format", fmt_args @ [Lit::Str(_), ..]) => {
            let fmt = match &fmt_args[0] {
                Lit::Str(s) => s.clone(),
                _ => return None,
            };
            let out = eval_string_format(&fmt, &fmt_args[1..])?;
            Lit::Str(out)
        }
        ("Math", "abs", [Lit::Int(v)]) => Lit::Int(v.wrapping_abs()),
        ("Math", "abs", [Lit::Long(v)]) => Lit::Long(v.wrapping_abs()),
        ("Math", "max", [Lit::Int(a), Lit::Int(b)]) => Lit::Int(*a.max(b)),
        ("Math", "min", [Lit::Int(a), Lit::Int(b)]) => Lit::Int(*a.min(b)),
        ("Math", "max", [Lit::Long(a), Lit::Long(b)]) => Lit::Long(*a.max(b)),
        ("Math", "min", [Lit::Long(a), Lit::Long(b)]) => Lit::Long(*a.min(b)),
        ("Character", "isDigit", [Lit::Char(c)]) => Lit::Bool(c.is_ascii_digit()),
        ("Character", "isLetter", [Lit::Char(c)]) => Lit::Bool(c.is_alphabetic()),
        ("Character", "isWhitespace", [Lit::Char(c)]) => Lit::Bool(c.is_whitespace()),
        ("Character", "isUpperCase", [Lit::Char(c)]) => Lit::Bool(c.is_uppercase()),
        ("Character", "isLowerCase", [Lit::Char(c)]) => Lit::Bool(c.is_lowercase()),
        ("Character", "toUpperCase", [Lit::Char(c)]) => Lit::Char(c.to_uppercase().next()?),
        ("Character", "toLowerCase", [Lit::Char(c)]) => Lit::Char(c.to_lowercase().next()?),
        ("Character", "toString", [Lit::Char(c)]) => Lit::Str(c.to_string()),
        _ => return None,
    })
}

/// String.format 最小子集：%s、%d、%%；其余转换（%f/%x/宽度/精度）不折。
fn eval_string_format(fmt: &str, args: &[Lit]) -> Option<String> {
    let mut out = String::new();
    let mut it = fmt.chars().peekable();
    let mut arg_i = 0usize;
    while let Some(c) = it.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        match it.next()? {
            '%' => out.push('%'),
            's' => {
                let a = args.get(arg_i)?;
                arg_i += 1;
                match a {
                    Lit::Str(s) => out.push_str(s),
                    Lit::Char(ch) => out.push(*ch),
                    Lit::Bool(b) => out.push_str(&b.to_string()),
                    _ => out.push_str(&lit_debug_str(a)),
                }
            }
            'd' => {
                let a = args.get(arg_i)?;
                arg_i += 1;
                match a {
                    Lit::Int(v) => out.push_str(&v.to_string()),
                    Lit::Long(v) => out.push_str(&v.to_string()),
                    Lit::Char(ch) => out.push_str(&(*ch as i64).to_string()),
                    _ => return None,
                }
            }
            // 未知/未支持转换：交给运行时
            _ => return None,
        }
    }
    if arg_i != args.len() {
        return None; // 实参数量与转换符不匹配 → 运行时异常路径，不折
    }
    Some(out)
}

fn lit_debug_str(l: &Lit) -> String {
    match l {
        Lit::Int(v) => v.to_string(),
        Lit::Long(v) => v.to_string(),
        _ => "?".into(),
    }
}

// ---------------------------------------------------------------------------
// Base64 字符串解码还原（混淆器标准藏匿手段）：
//   new String(Base64.getDecoder().decode("aGVsbG8=")) → "hello"
//   （URL 解码器同理；解码字节须为合法 UTF-8）
// ---------------------------------------------------------------------------

pub struct Base64NewStringFold;

impl Rule<JavaAst> for Base64NewStringFold {
    fn name(&self) -> &'static str {
        "base64_new_string_fold"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        let NodeData::New { ty, .. } = lang.data(id) else {
            return None;
        };
        if !matches!(ty, JType::Ref(n) if n == "String" || n.ends_with(".String")) {
            return None;
        }
        let args = lang.children(id).to_vec();
        if args.len() != 1 {
            return None;
        }
        // 参数：Base64.getXxxDecoder().decode("lit")
        if lang.kind(args[0]) != NodeKind::Call {
            return None;
        }
        let dc = lang.children(args[0]).to_vec();
        if dc.len() != 2 {
            return None;
        }
        let NodeData::Member { name: dn } = lang.data(dc[0]) else {
            return None;
        };
        if dn != "decode" {
            return None;
        }
        // recv = Base64.getXxxDecoder()（调用）
        let recv = lang.children(dc[0])[0];
        if lang.kind(recv) != NodeKind::Call {
            return None;
        }
        let gc = lang.children(recv).to_vec();
        let NodeData::Member { name: gn } = lang.data(*gc.first()?) else {
            return None;
        };
        let url_safe = match gn.as_str() {
            "getDecoder" => false,
            "getUrlDecoder" => true,
            _ => return None,
        };
        // owner = java.util.Base64（Member 链）或裸 Base64（VarRef）
        let owner = lang.children(gc[0])[0];
        let owner_ok = match lang.data(owner) {
            NodeData::Member { name } => name == "Base64",
            _ => matches!(lang.var_name(owner), Some(n) if n == "Base64" || n.ends_with(".Base64")),
        };
        if !owner_ok {
            return None;
        }
        let LitRef::Str(b64) = lang.literal(dc[1])? else {
            return None;
        };
        let bytes = base64_decode(b64, url_safe)?;
        let text = String::from_utf8(bytes).ok()?;
        let with = lang.build_str(&text);
        Some(Edit::Replace {
            target: id,
            with,
        })
    }
}

/// 手写 base64 解码（标准/URL 字母表，允许缺省填充）。
fn base64_decode(s: &str, url_safe: bool) -> Option<Vec<u8>> {
    let val = |c: char| -> Option<u32> {
        match c {
            'A'..='Z' => Some(c as u32 - 'A' as u32),
            'a'..='z' => Some(c as u32 - 'a' as u32 + 26),
            '0'..='9' => Some(c as u32 - '0' as u32 + 52),
            '+' if !url_safe => Some(62),
            '/' if !url_safe => Some(63),
            '-' if url_safe => Some(62),
            '_' if url_safe => Some(63),
            _ => None,
        }
    };
    let clean: String = s.chars().filter(|&c| c != '=' && !c.is_whitespace()).collect();
    if clean.len() % 4 == 1 {
        return None;
    }
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0u32;
    for c in clean.chars() {
        buf = (buf << 6) | val(c)?;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Some(out)
}


// ---------------------------------------------------------------------------
// 控制流扁平化还原（CFF recovery，obfuscator.io / Allatori 风格）：
//   int s = 0;
//   while (true) {
//       switch (s) {
//           case 0: { A(); s = 1; break; }
//           case 1: { if (c) { s = 2; } else { s = 3; } break; }
//           case 2: { B(); s = 5; break; }
//           case 3: { C(); s = 5; break; }
//       }
//       if (s == 5) break;      // 或 default: return
//   }
//   →  A(); if (c) { B(); } else { C(); }
//
// 状态图 → 结构化控制流：
//   线性链顺序拼接；菱形找公共后继 if/else{前缀}+续接（单次发射）；
//   分支回环到条件状态 → while(cond){体}；链内后向边 → while(true){后缀}。
//   其余形态（发散/外部跳转/嵌套异常）保守拒绝。
// 守卫：v 仅作状态变量、循环后不使用、全部 case 可达、每状态体恰发射一次。
// ---------------------------------------------------------------------------

pub struct CffRecover;

struct CffCase {
    body: Vec<JavaId>,
    tail: CffTail,
}

enum CffTail {
    Goto(i64),
    Cond { cond: JavaId, a: i64, b: i64 },
    Term(JavaId),
    Exit,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Flow {
    Done,
    Stop,
    Cycle(usize),
    Cond(i64),
    External(i64),
}

struct CffCtx<'a> {
    lang: &'a mut JavaAst,
    cases: std::collections::HashMap<i64, CffCase>,
    /// default: return 的语句（goto 未知状态时发射）
    default_term: Option<Vec<JavaId>>,
    used_default: bool,
    /// 可达性记账（含试探性分支走查）
    reached: std::collections::HashSet<i64>,
    /// 已实际发射的状态（分支试探不得穿入）
    emitted: std::collections::HashSet<i64>,
}

impl Rule<JavaAst> for CffRecover {
    fn name(&self) -> &'static str {
        "cff_recover"
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let parent = ctx.parent(id)?;
        let idx = ctx.index(id)?;
        let lang = ctx.lang;
        if std::env::var("CURE_DBG").is_ok() {
            eprintln!("[cff] check called: {:?}", lang.kind(id));
        }
        if lang.kind(id) != NodeKind::While {
            return None;
        }
        let wch = lang.children(id).to_vec();
        if !matches!(lang.literal(wch[0]), Some(LitRef::Bool(true))) {
            return None;
        }
        if lang.kind(wch[1]) != NodeKind::Block {
            return None;
        }
                let body_ch = lang.children(wch[1]).to_vec();
        let (switch, exit_if) = match body_ch.len() {
            1 => (body_ch[0], None),
            2 => (body_ch[0], Some(body_ch[1])),
            _ => return None,
        };
        if lang.kind(switch) != NodeKind::Switch {
            if std::env::var("CURE_DBG").is_ok() { eprintln!("[cff] body[0] not switch: {:?}", lang.kind(switch)); }
            return None;
        }
        let sch = lang.children(switch).to_vec();
        if lang.kind(sch[0]) != NodeKind::VarRef || !lang.is_local_var(sch[0]) {
            return None;
        }
        let v = lang.var_name(sch[0])?.to_string();

        // 尾随出口：if (v == SENT) break;
        let sentinel = match exit_if {
            None => None,
            Some(ei) => {
                if lang.kind(ei) != NodeKind::If {
                    return None;
                }
                let ich = lang.children(ei).to_vec();
                if ich.len() != 2 || lang.bin_op(ich[0]) != Some(BinOp::Eq) {
                    return None;
                }
                let cch = lang.children(ich[0]).to_vec();
                let sent = if lang.kind(cch[0]) == NodeKind::VarRef
                    && lang.var_name(cch[0]) == Some(v.as_str())
                {
                    lang.literal(cch[1]).and_then(|x| x.as_int())
                } else if lang.kind(cch[1]) == NodeKind::VarRef
                    && lang.var_name(cch[1]) == Some(v.as_str())
                {
                    lang.literal(cch[0]).and_then(|x| x.as_int())
                } else {
                    return None;
                }?;
                let then_ok = match lang.kind(ich[1]) {
                    NodeKind::Break => matches!(lang.data(ich[1]), NodeData::Break { label: None }),
                    NodeKind::Block if lang.children(ich[1]).len() == 1 => {
                        matches!(lang.data(lang.children(ich[1])[0]), NodeData::Break { label: None })
                    }
                    _ => false,
                };
                if !then_ok {
                    return None;
                }
                Some(sent)
            }
        };

        // 解析 case 表
        let mut cases: std::collections::HashMap<i64, CffCase> = Default::default();
        let mut default_term: Option<Vec<JavaId>> = None;
        for &c in &sch[1..] {
            let NodeData::Case { labels, is_default, arrow } = lang.data(c) else {
                return None;
            };
            let (labels, is_default, arrow) = (*labels, *is_default, *arrow);
            let ch = lang.children(c).to_vec();
            let mut stmts: Vec<JavaId> = ch[labels as usize..].to_vec();
            // 经典形态：剥尾部 break（块外或块内）
            if !arrow
                && stmts
                    .last()
                    .is_some_and(|&n| matches!(lang.data(n), NodeData::Break { label: None }))
            {
                stmts.pop();
            }
            if stmts.len() == 1 && lang.kind(stmts[0]) == NodeKind::Block {
                let mut inner = lang.children(stmts[0]).to_vec();
                if !arrow
                    && inner
                        .last()
                        .is_some_and(|&n| matches!(lang.data(n), NodeData::Break { label: None }))
                {
                    inner.pop();
                }
                stmts = inner;
            }
            if is_default {
                if stmts.len() == 1 && lang.kind(stmts[0]) == NodeKind::Return {
                    default_term = Some(stmts);
                    continue;
                }
                return None;
            }
            // 状态标签：单一整数字面量
            if labels != 1 {
                return None;
            }
            let state = match lang.literal(ch[0]) {
                Some(LitRef::Int(v)) => v,
                _ => return None,
            };
            let tail = parse_cff_tail(lang, &v, &mut stmts)?;
            // 尾部剥离后，剩余体不得引用状态变量
            for &st in &stmts {
                if subtree_has_var(lang, st, &v) {
                    return None;
                }
            }
            cases.insert(state, CffCase { body: stmts, tail });
        }
        if cases.is_empty() {
            return None;
        }

        // 入口：while 前一条语句设置 v = IntLit
        if idx == 0 {
            if std::env::var("CURE_DBG").is_ok() { eprintln!("[cff] idx==0"); }
            return None;
        }
        let block_ch = lang.children(parent).to_vec();
        let init_stmt = block_ch.get(idx - 1).copied()?;
        let init = parse_cff_init(lang, &v, init_stmt)?;

        // 循环后 v 不得被使用
        for &st in &block_ch[idx + 1..] {
            if subtree_has_var(lang, st, &v) {
                return None;
            }
        }

        // Goto(未知状态) → Exit（须有出口机制）
        let mut to_exit: Vec<i64> = Vec::new();
        for (k, ci) in cases.iter() {
            if let CffTail::Goto(g) = ci.tail {
                if !cases.contains_key(&g) {
                    if !(sentinel == Some(g) || default_term.is_some()) {
                        return None;
                    }
                    to_exit.push(*k);
                }
            }
        }
        for k in to_exit {
            if let Some(ci) = cases.get_mut(&k) {
                ci.tail = CffTail::Exit;
            }
        }

        let mut c = CffCtx {
            lang,
            cases,
            default_term,
            used_default: false,
            reached: Default::default(),
            emitted: Default::default(),
        };
        let stmts = cff_run(&mut c, init)?;

        // 全部 case 可达；default 若存在必须被走到
        if !c.cases.keys().all(|k| c.reached.contains(k)) {
            return None;
        }
        if c.default_term.is_some() && !c.used_default {
            return None;
        }
        if stmts.is_empty() {
            return None;
        }

        Some(Edit::Splice {
            node: parent,
            index: idx - 1,
            remove: 2,
            insert: stmts,
        })
    }
}

fn parse_cff_tail(lang: &JavaAst, v: &str, stmts: &mut Vec<JavaId>) -> Option<CffTail> {
    let last = *stmts.last()?;
    if lang.kind(last) == NodeKind::Return {
        stmts.pop();
        return Some(CffTail::Term(last));
    }
    let assign_of = |n: JavaId| -> Option<JavaId> {
        let inner = match lang.kind(n) {
            NodeKind::Assign => n,
            NodeKind::ExprStmt => {
                let ch = lang.children(n);
                if ch.len() == 1 && lang.kind(ch[0]) == NodeKind::Assign {
                    ch[0]
                } else {
                    return None;
                }
            }
            _ => return None,
        };
        if lang.assign_op(inner).is_some() {
            return None;
        }
        let ch = lang.children(inner).to_vec();
        if lang.kind(ch[0]) != NodeKind::VarRef || lang.var_name(ch[0]) != Some(v) {
            return None;
        }
        Some(inner)
    };
    if let Some(assign) = assign_of(last) {
        let value = lang.children(assign)[1];
        if let Some(k) = lang.literal(value).and_then(|x| x.as_int()) {
            stmts.pop();
            return Some(CffTail::Goto(k));
        }
        if lang.kind(value) == NodeKind::Ternary {
            let tch = lang.children(value).to_vec();
            let a = lang.literal(tch[1]).and_then(|x| x.as_int())?;
            let b = lang.literal(tch[2]).and_then(|x| x.as_int())?;
            stmts.pop();
            return Some(CffTail::Cond { cond: tch[0], a, b });
        }
        return None;
    }
    if lang.kind(last) == NodeKind::If {
        let ich = lang.children(last).to_vec();
        if ich.len() != 3 {
            return None;
        }
        let arm = |n: JavaId| -> Option<i64> {
            let target = match lang.kind(n) {
                NodeKind::Assign | NodeKind::ExprStmt => assign_of(n)?,
                NodeKind::Block if lang.children(n).len() == 1 => assign_of(lang.children(n)[0])?,
                _ => return None,
            };
            lang.literal(lang.children(target)[1]).and_then(|x| x.as_int())
        };
        let a = arm(ich[1])?;
        let b = arm(ich[2])?;
        stmts.pop();
        return Some(CffTail::Cond { cond: ich[0], a, b });
    }
    None
}

fn parse_cff_init(lang: &JavaAst, v: &str, stmt: JavaId) -> Option<i64> {
    match lang.kind(stmt) {
        NodeKind::VarDecl => {
            let ch = lang.children(stmt).to_vec();
            if ch.len() != 1 || lang.var_name(stmt) != Some(v) {
                return None;
            }
            lang.literal(ch[0]).and_then(|x| x.as_int())
        }
        NodeKind::Assign => {
            if lang.assign_op(stmt).is_some() {
                return None;
            }
            let ch = lang.children(stmt).to_vec();
            if lang.kind(ch[0]) != NodeKind::VarRef || lang.var_name(ch[0]) != Some(v) {
                return None;
            }
            lang.literal(ch[1]).and_then(|x| x.as_int())
        }
        NodeKind::ExprStmt => {
            let ch = lang.children(stmt).to_vec();
            if ch.len() == 1 {
                parse_cff_init(lang, v, ch[0])
            } else {
                None
            }
        }
        _ => None,
    }
}

fn flatten(pairs: &[(i64, Vec<JavaId>)], out: &mut Vec<JavaId>, c: &mut CffCtx) {
    for (s, stmts) in pairs {
        c.emitted.insert(*s);
        out.extend(stmts.iter().copied());
    }
}

fn flatten_prefix(pairs: &[(i64, Vec<JavaId>)], m: i64, out: &mut Vec<JavaId>) {
    for (s, stmts) in pairs {
        if *s == m {
            break;
        }
        out.extend(stmts.iter().copied());
    }
}

fn suffix_pairs(pairs: &[(i64, Vec<JavaId>)], m: i64) -> Vec<(i64, Vec<JavaId>)> {
    let pos = pairs.iter().position(|(s, _)| *s == m).unwrap();
    pairs[pos..].to_vec()
}

/// 纯收集：沿无条件 goto 前进，不消费条件 case。
fn collect(c: &mut CffCtx, start: i64, stop: Option<i64>) -> Option<(Vec<(i64, Vec<JavaId>)>, Flow)> {
    let mut path: Vec<(i64, Vec<JavaId>)> = Vec::new();
    let mut cur = start;
    let mut guard = 0usize;
    loop {
        guard += 1;
        if guard > 10_000 {
            return None;
        }
        if Some(cur) == stop {
            return Some((path, Flow::Stop));
        }
        if let Some(i) = path.iter().position(|(s, _)| *s == cur) {
            return Some((path, Flow::Cycle(i)));
        }
        if c.emitted.contains(&cur) {
            return Some((path, Flow::External(cur)));
        }
        c.reached.insert(cur);
        let ci = c.cases.get(&cur)?;
        let mut body = ci.body.clone();
        match ci.tail {
            CffTail::Goto(k) => {
                path.push((cur, body));
                cur = k;
            }
            CffTail::Term(r) => {
                body.push(r);
                path.push((cur, body));
                return Some((path, Flow::Done));
            }
            CffTail::Exit => {
                if c.default_term.is_some() {
                    if let Some(dt) = c.default_term.clone() {
                        body.extend(dt);
                        c.used_default = true;
                    }
                }
                path.push((cur, body));
                return Some((path, Flow::Done));
            }
            CffTail::Cond { .. } => return Some((path, Flow::Cond(cur))),
        }
    }
}

/// 顶层：从入口状态结构化整个状态机。
fn cff_run(c: &mut CffCtx, init: i64) -> Option<Vec<JavaId>> {
    let (path, flow) = collect(c, init, None)?;
    cff_structure(c, path, flow)
}

/// 从某状态起递归结构化（混合终点分支的续接构造用）。
fn cff_run_from(c: &mut CffCtx, start: i64) -> Option<Vec<JavaId>> {
    let (path, flow) = collect(c, start, None)?;
    cff_structure(c, path, flow)
}

/// 核心结构化循环：携带当前链（path + flow），直到终止。
/// 混合终点（一支终止、另一支续接）时，续接必须放进继续分支**内部**
/// （续接代码只在走该分支时执行）。
fn cff_structure(c: &mut CffCtx, mut path: Vec<(i64, Vec<JavaId>)>, mut flow: Flow) -> Option<Vec<JavaId>> {
    let mut stmts: Vec<JavaId> = Vec::new();
    loop {
        match flow {
            Flow::Done => {
                flatten(&path, &mut stmts, c);
                return Some(stmts);
            }
            Flow::Cycle(i) => {
                flatten(&path[..i], &mut stmts, c);
                let mut suffix = Vec::new();
                flatten(&path[i..], &mut suffix, c);
                let inner = c.lang.build_block(suffix);
                let cond = c.lang.build_bool(true);
                let w = c.lang.while_(cond, inner);
                stmts.push(w);
                return Some(stmts);
            }
            Flow::Stop | Flow::External(_) => return None,
            Flow::Cond(state) => {
                flatten(&path, &mut stmts, c);
                path.clear();
                let ci = c.cases.get(&state)?;
                stmts.extend(ci.body.iter().copied());
                c.emitted.insert(state);
                let CffTail::Cond { cond, a, b } = ci.tail else {
                    return None;
                };
                // 循环形态：分支走回条件状态
                let (pa, fa) = collect(c, a, Some(state))?;
                if fa == Flow::Stop {
                    let mut lb = Vec::new();
                    flatten(&pa, &mut lb, c);
                    let inner = c.lang.build_block(lb);
                    let w = c.lang.while_(cond, inner);
                    stmts.push(w);
                    let (p2, f2) = collect(c, b, None)?;
                    path = p2;
                    flow = f2;
                    continue;
                }
                let (pb, fb) = collect(c, b, Some(state))?;
                if fb == Flow::Stop {
                    let mut lb = Vec::new();
                    flatten(&pb, &mut lb, c);
                    let inner = c.lang.build_block(lb);
                    let nc = c.lang.build_unary(UnOp::Not, cond);
                    let w = c.lang.while_(nc, inner);
                    stmts.push(w);
                    let (p2, f2) = collect(c, a, None)?;
                    path = p2;
                    flow = f2;
                    continue;
                }
                // 菱形：两支共享后继
                let common = pa
                    .iter()
                    .find(|(s, _)| pb.iter().any(|(t, _)| *s == *t))
                    .map(|(s, _)| *s);
                let same_end = match (fa, fb) {
                    (Flow::Done, Flow::Done) => true,
                    (Flow::Cond(x), Flow::Cond(y)) => x == y,
                    _ => false,
                };
                match common {
                    Some(m) if same_end => {
                        let mut pre_a = Vec::new();
                        let mut pre_b = Vec::new();
                        flatten_prefix(&pa, m, &mut pre_a);
                        flatten_prefix(&pb, m, &mut pre_b);
                        let ba = c.lang.build_block(pre_a);
                        let bb = c.lang.build_block(pre_b);
                        let ifs = c.lang.if_(cond, ba, Some(bb));
                        stmts.push(ifs);
                        path = suffix_pairs(&pa, m);
                        flow = fa;
                        continue;
                    }
                    Some(_) => return None,
                    None => {
                        // 无公共后继的分叉
                        let mut fa_stmts = Vec::new();
                        let mut fb_stmts = Vec::new();
                        flatten(&pa, &mut fa_stmts, c);
                        flatten(&pb, &mut fb_stmts, c);
                        match (fa, fb) {
                            (Flow::Done, Flow::Done) => {
                                let ba = c.lang.build_block(fa_stmts);
                                let bb = c.lang.build_block(fb_stmts);
                                let ifs = c.lang.if_(cond, ba, Some(bb));
                                stmts.push(ifs);
                                return Some(stmts);
                            }
                            // 两支都续接到同一条件 → if/else 后无条件续接（合流）
                            (Flow::Cond(x), Flow::Cond(y)) if x == y => {
                                let ba = c.lang.build_block(fa_stmts);
                                let bb = c.lang.build_block(fb_stmts);
                                let ifs = c.lang.if_(cond, ba, Some(bb));
                                stmts.push(ifs);
                                let (p2, f2) = collect(c, x, None)?;
                                path = p2;
                                flow = f2;
                                continue;
                            }
                            // 混合：一支终止、另一支续接 → 续接放进继续分支【内部】
                            (Flow::Done, Flow::Cond(x)) => {
                                let cont = cff_run_from(c, x)?;
                                fb_stmts.extend(cont);
                                let ba = c.lang.build_block(fa_stmts);
                                let bb = c.lang.build_block(fb_stmts);
                                let ifs = c.lang.if_(cond, ba, Some(bb));
                                stmts.push(ifs);
                                return Some(stmts);
                            }
                            (Flow::Cond(x), Flow::Done) => {
                                let cont = cff_run_from(c, x)?;
                                fa_stmts.extend(cont);
                                let ba = c.lang.build_block(fa_stmts);
                                let bb = c.lang.build_block(fb_stmts);
                                let ifs = c.lang.if_(cond, ba, Some(bb));
                                stmts.push(ifs);
                                return Some(stmts);
                            }
                            _ => return None,
                        }
                    }
                }
            }
        }
    }
}


// ---------------------------------------------------------------------------
// 门面
// ---------------------------------------------------------------------------

/// Java 默认规则集（引擎通用规则 + Java 特有规则；保守集，不含 DCE 类）。
pub fn default_java_rules() -> Vec<Box<dyn Rule<JavaAst>>> {
    let mut rules: Vec<Box<dyn Rule<JavaAst>>> = cure_engine::rules::default_rules();
    rules.push(Box::new(CastSimplify));
    rules.push(Box::new(SelfCompare));
    rules.push(Box::new(StringBuilderFold));
    rules.push(Box::new(BoxUnboxChain));
    rules.push(Box::new(IteratorToForEach));
    rules.push(Box::new(NewStringFold));
    rules.push(Box::new(LoopHeadBreak));
    rules.push(Box::new(WhileIteratorToForEach));
    rules.push(Box::new(ConcatValueOfDrop));
    rules.push(Box::new(StringBuilderStatements));
    rules.push(Box::new(TrailingContinueJava));
    rules.push(Box::new(XorNoise));
    rules.push(Box::new(StrLenFold));
    rules.push(Box::new(LiteralEval));
    rules.push(Box::new(Base64NewStringFold));
    rules.push(Box::new(CffRecover));
    rules
}

/// 全量规则（含 DCE 类选配项：不可达语句删除等）。
pub fn all_java_rules() -> Vec<Box<dyn Rule<JavaAst>>> {
    let mut rules = default_java_rules();
    let mut engine_all = cure_engine::rules::all_rules();
    rules.append(&mut engine_all);
    rules
}

/// 对方法体（或任意语句 Block）运行 Java 规则集直到 fixed point。
pub fn simplify(ast: &mut JavaAst, root: JavaId, cfg: &Config) -> Report {
    let rules = default_java_rules();
    cure_engine::simplify(ast, root, &rules, cfg)
}

/// 整个编译单元：逐个方法体/初始化块/字段初始化器优化，签名不动。
pub fn simplify_unit(ast: &mut JavaAst, unit: &mut CompilationUnit, cfg: &Config) -> Report {
    let mut total = Report::default();
    for ty in &mut unit.types {
        simplify_type(ast, ty, cfg, &mut total);
    }
    total
}

fn simplify_type(ast: &mut JavaAst, ty: &mut TypeDecl, cfg: &Config, total: &mut Report) {
    for m in &mut ty.members {
        simplify_member(ast, m, cfg, total);
    }
}

fn simplify_member(ast: &mut JavaAst, m: &mut Member, cfg: &Config, total: &mut Report) {
    // 方法参数进入作用域基表，供类型解析使用
    let params: Vec<(String, JType)> = match m {
        Member::Method { params, .. } | Member::Constructor { params, .. } => params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect(),
        _ => Vec::new(),
    };
    ast.set_param_scope(&params);
    match m {
        Member::Method { body: Some(b), .. }
        | Member::Constructor { body: Some(b), .. } => {
            let r = simplify(ast, *b, cfg);
            merge_report(total, r);
        }
        Member::Initializer { body, .. } => {
            let r = simplify(ast, *body, cfg);
            merge_report(total, r);
        }
        Member::Field { declarators, .. } => {
            for d in declarators.iter_mut() {
                if let Some(init) = d.init {
                    let r = simplify(ast, init, cfg);
                    merge_report(total, r);
                }
            }
        }
        Member::Type(t) => simplify_type(ast, t, cfg, total),
        Member::Raw(_) => {}
        _ => {}
    }
}

fn merge_report(total: &mut Report, r: Report) {
    total.edits += r.edits;
    total.iterations += r.iterations;
    for (k, v) in r.by_rule {
        *total.by_rule.entry(k).or_insert(0) += v;
    }
}

/// 默认配置。
pub fn default_config() -> Config {
    Config::default()
}
