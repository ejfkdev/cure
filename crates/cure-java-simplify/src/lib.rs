//! # cure-java-simplify
//!
//! Java 简化门面：引擎通用规则 + Java 特有规则，一站式 [`simplify`]。
//!
//! 引擎规则（布尔、自赋值、常量条件、局部传播……）定义在 cure-engine，
//! 通过 `Lang` 抽象复用；本 crate 只放需要 Java 类型/负载信息的规则。

use cure_engine::kind::{BinOp, UnOp};
use cure_engine::{Config, Edit, Lang, LitRef, NodeKind, Report, RewriteCtx, Rule};
use cure_java_ast::{CompilationUnit, JavaAst, JavaId, JType, Member, NodeData, TypeDecl};

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
        if lang.kind(id) != NodeKind::While {
            return None;
        }
        let ch = lang.children(id).to_vec();
        let (cond, body) = (ch[0], ch[1]);
        if !matches!(lang.literal(cond), Some(LitRef::Bool(true))) {
            return None;
        }
        if lang.kind(body) != NodeKind::Block {
            return None;
        }
        let bch = lang.children(body).to_vec();
        let first = *bch.first()?;
        if lang.kind(first) != NodeKind::If {
            return None;
        }
        let ich = lang.children(first).to_vec();
        let c = ich[0];
        // then 分支必须是单个无标签 break
        let bare_break = |n: JavaId| matches!(lang.data(n), NodeData::Break { label: None });
        let then_break = match lang.kind(ich[1]) {
            NodeKind::Break => bare_break(ich[1]),
            NodeKind::Block if lang.children(ich[1]).len() == 1 => {
                bare_break(lang.children(ich[1])[0])
            }
            _ => false,
        };
        if !then_break {
            return None;
        }
        // REST = else 分支（若有）+ body 其余语句
        let mut rest: Vec<JavaId> = Vec::new();
        if ich.len() == 3 {
            match lang.kind(ich[2]) {
                NodeKind::Block => rest.extend(lang.children(ich[2]).iter().copied()),
                _ => rest.push(ich[2]),
            }
        }
        rest.extend(bch[1..].iter().copied());
        let nc = lang.build_unary(UnOp::Not, c);
        let new_body = lang.build_block(rest);
        Some(Edit::Splice {
            node: id,
            index: 0,
            remove: 2,
            insert: vec![nc, new_body],
        })
    }
}

// ---------------------------------------------------------------------------
// while 形式迭代器还原（jadx 常见，jcdc/ddc 同样产出）：
//   Iterator<E> it = c.iterator();
//   while (it.hasNext()) { E e = it.next(); REST }
//   →  for (E e : c) { REST }
// 安全性：与 for-each 脱糖同构；守卫：`it` 在 REST 中不再被引用。
// ---------------------------------------------------------------------------

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
