//! # cure-java-simplify
//!
//! Java 简化门面：引擎通用规则 + Java 特有规则，一站式 [`simplify`]。
//!
//! 引擎规则（布尔、自赋值、常量条件、局部传播……）定义在 cure-engine，
//! 通过 `Lang` 抽象复用；本 crate 只放需要 Java 类型/负载信息的规则。

mod vexec;

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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Cast]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Binary]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Call]
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let root = ctx.root();
        let lang = ctx.lang;
        // 外层：X.toString()
        if lang.kind(id) != NodeKind::Call {
            return None;
        }
        let callee = lang.children(id)[0];
        if let NodeData::Member { name } = lang.data(callee) {
            if lang.sn(*name) != "toString" || !lang.children(id)[1..].is_empty() {
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
                        if lang.sn(*name) == "append" && c.len() == 2 {
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
                            // String 内容构造参与拼接；int/long 字面量必为
                            // 容量构造（无接收 int 的内容重载）——容量只是
                            // 分配提示，无语义差异，按无参处理；变量实参
                            // 类型不可证，保守拒绝
                            let a = args[0];
                            let stringy = matches!(lang.literal(a), Some(LitRef::Str(_)))
                                || lang
                                    .var_type(a)
                                    .is_some_and(|t| matches!(t, JType::Ref(n) if n == "String"));
                            if stringy {
                                parts.push(a);
                            } else if !matches!(
                                lang.literal(a),
                                Some(LitRef::Int(_) | LitRef::Long(_))
                            ) {
                                return None;
                            }
                        }
                        _ => return None,
                    }
                    break;
                }
                _ => return None,
            }
        }
        // 全字面量链折叠结果是常量表达式（javac 池化），而 sb.toString() 是
        // 运行期新建未池化的串——== 语义会变 → 身份守卫。混合链（含变量/调用）
        // 折叠后是非常量拼接，运行期行为与 toString 一致，无需守卫。
        if parts.iter().all(|&p| lang.literal(p).is_some())
            && lang.has_string_identity_compare(root)
        {
            return None;
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Call]
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
                if lang.sn(*name) != "valueOf" {
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
        if expected != lang.sn(unbox_name) {
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::For]
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
        if lang.sn(*m0) != "iterator" || ic.len() != 1 {
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
        if lang.sn(*m1) != "hasNext" || cc.len() != 1 {
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
        if lang.sn(*m2) != "next" || nc.len() != 1 {
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

/// for-each 元素类型合法性：`?`/`? extends …`/`? super …`（通配符）不是
/// 合法变量类型 → Object（jdk-sources Subject 抓获：`Iterator<?> ce =
/// c.iterator()` 还原成 `for (? e2 : c)` 非法 Java）
fn for_each_elem_ok(ty: &JType) -> bool {
    match ty {
        JType::Ref(n) => !n.trim_start().starts_with('?'),
        _ => true,
    }
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::New]
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let root = ctx.root();
        let lang = ctx.lang;
        let NodeData::New { ty, .. } = lang.data(id) else {
            return None;
        };
        if !matches!(ty, JType::Ref(n) if n == "String" || n == "java.lang.String" || n.ends_with(".String")) {
            return None;
        }
        // 折叠产出池化字面量，改变 String 引用身份 → 守卫
        if lang.has_string_identity_compare(root) {
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
// new String(char 字面量数组) → 字符串字面量（ZKM/Allatori 的字符串藏匿形态）。
// 守卫与 new String(lit) 相同：产出池化字面量改变 == 引用语义。
// ---------------------------------------------------------------------------

pub struct NewStringCharArrayFold;

impl Rule<JavaAst> for NewStringCharArrayFold {
    fn name(&self) -> &'static str {
        "new_string_char_array_fold"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::New]
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let root = ctx.root();
        let lang = ctx.lang;
        let NodeData::New { ty, .. } = lang.data(id) else {
            return None;
        };
        if !matches!(ty, JType::Ref(n) if n == "String" || n.ends_with(".String")) {
            return None;
        }
        if lang.has_string_identity_compare(root) {
            return None;
        }
        // 唯一实参：new char[]{...}（NewArray sized=0，唯一孩子 ArrayLit）
        let args = lang.children(id).to_vec();
        if args.len() != 1 {
            return None;
        }
        let NodeData::NewArray { sized, .. } = lang.data(args[0]) else {
            return None;
        };
        if *sized != 0 {
            return None;
        }
        let arr_children = lang.children(args[0]).to_vec();
        let lit = *arr_children.first()?;
        if lang.kind(lit) != NodeKind::ArrayLit {
            return None;
        }
        let mut text = String::new();
        for &c in lang.children(lit) {
            match lang.literal(c) {
                Some(LitRef::Char(ch)) => text.push(ch),
                _ => return None, // 非字符字面量（变量/表达式）不折
            }
        }
        let with = lang.build_str(&text);
        Some(Edit::Replace {
            target: id,
            with,
        })
    }
}

// ---------------------------------------------------------------------------
// 空 finally 剥除（dex2jar/ProGuard 产物）：
//   try { B } finally {}（无 catch、无资源）→ B
//   try { B } catch… finally {} → try { B } catch…（空 finally 删除）
// 语义：空 finally 无可观察行为；有 catch 时保留 try/catch 结构。
// ---------------------------------------------------------------------------

pub struct EmptyFinallyStrip;

impl Rule<JavaAst> for EmptyFinallyStrip {
    fn name(&self) -> &'static str {
        "empty_finally_strip"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Try]
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        if lang.kind(id) != NodeKind::Try {
            return None;
        }
        let ch = lang.children(id).to_vec();
        // children: [resource…, try_block, catch…, (finally)?]
        // 【关键】try_block = 第一个 Block 孩子；finally 只能是 try_block 与
        // catch 之后的最后孩子。**try_block 本身绝不在此删**（空 try 块交给
        // TryUnwrapNoCatch 整体解包）——曾经"last 为空块就删"会把空 try_block
        // 误当 finally 剥掉，try-with-resources 的资源直接暴露在 try 头外
        //（spoon sniperPrinter 真实语料抓获）。
        let is_empty_block = |n: JavaId| -> bool {
            lang.kind(n) == NodeKind::Block && lang.children(n).is_empty()
        };
        let try_idx = ch.iter().position(|&c| lang.kind(c) == NodeKind::Block)?;
        if ch.len() < try_idx + 2 {
            return None; // try_block 后没有孩子（无 finally）
        }
        // finally 存在性：最后孩子在 try_idx 之后且不是 catch
        let last = *ch.last()?;
        if lang.kind(last) == NodeKind::Catch {
            return None;
        }
        if !is_empty_block(last) {
            return None;
        }
        // last 距 try_idx 至少隔 1（try_idx 本身不可是 last——上面已保证
        // ch.len() ≥ try_idx+2，但 last 也可能是紧随 try_block 的 catch 后的 finally）
        Some(Edit::Splice {
            node: id,
            index: ch.len() - 1,
            remove: 1,
            insert: Vec::new(),
        })
    }
}

// ---------------------------------------------------------------------------
// 无 catch + 无资源 + 空/无 finally → 完全解包 try：
//   try { B } finally {} / try { B } → B
// ---------------------------------------------------------------------------


// ---------------------------------------------------------------------------
// 纯重抛 catch 剥除（fernflower 异常表残迹——审查统计 ≥23 处/11 文件）：
//   try { S } catch (T v) { throw v; }   →  S            （唯一 catch）
//   try { S } catch (A a) { throw a; } catch (B b) { …真处理… } →
//   try { S } catch (B b) { …真处理… }                   （只删重抛臂）
// 守卫：catch 体**只有**一条 throw 语句且抛的正是 catch 绑定变量自身
//（变量在其他处无使用——单一语句保证）；finally / 资源不动。
// 语义：捕获后原样重抛对求值顺序、副作用、异常路径与堆栈完全透明
//（栈深 +1 帧的差异源码层不可观察）。
// ---------------------------------------------------------------------------


// ---------------------------------------------------------------------------
// 虚拟执行 V2（static_exec）：对**根块**（方法体 / 静态初始化块 / 字段
// init）的纯计算段整体求值，重写为「逃逸局部 = 常量」+「字段写 = 常量」
// 的直线赋值。
//
// 目标形态（fernflower 语料 12/66 文件、残留噪声主体）：静态字符串
// 解密机——`"密文".toCharArray()` + `switch(i%5)` 常量键 + `(char)(c^k)`
// 循环 + `new String(arr).intern()`。语义全部由 vexec 解释器保守
// 求值（任何未知读/副作用/超预算立即放弃）。
//
// 正确性不变量：
//   - 仅根块（walk 无父）——嵌套块的存储无法区分外层局部/字段；
//   - 方法参数写 → 放弃（param 守卫）；
//   - 单路径确定性：条件/循环的条件全部求值为常量，执行走过的就是
//     全部活路径；
//   - 提前 Return/Break 只允许发生在**最后一条**顶层语句（此前返回
//     则后续语句在别的路径可达，单路径值不构成证明）；
//   - 逃逸局部（后半块句法引用的段内局部）与字段写的值必须可
//     材料化（Str/Int/Long/Bool/Str[]），否则放弃；
//   - 值中含未配对代理（UTF-8 不可表示）→ 放弃。
// ---------------------------------------------------------------------------


/// 幂等守卫用的宽松等价：字面量按**值**等价（NumRaw{val:Int(0)} ≡
/// Int(0)——打印相同），VarDecl 忽略类型标注差异（Int ≡ var）。
/// static_exec 产物每轮被零打印差编辑重写（ASTParserTokenManager
/// 抓获：ExprStmt→裸Assign/NumRaw→Int 交替），须在此归零。
fn print_equiv(lang: &JavaAst, a: JavaId, b: JavaId) -> bool {
    if lang.kind(a) != lang.kind(b) {
        // ExprStmt{X} ≡ X（裸语句形态差异）
        let (ia, ib) = (lang.kind(a), lang.kind(b));
        let unwrap = |k: NodeKind, x: JavaId| -> Option<JavaId> {
            if k == NodeKind::ExprStmt {
                lang.children(x).first().copied()
            } else {
                Some(x)
            }
        };
        return match (unwrap(ia, a), unwrap(ib, b)) {
            (Some(x), Some(y)) => print_equiv_inner(lang, x, y),
            _ => false,
        };
    }
    print_equiv_inner(lang, a, b)
}

fn print_equiv_inner(lang: &JavaAst, a: JavaId, b: JavaId) -> bool {
    if lang.kind(a) != lang.kind(b) {
        return false;
    }
    // 字面量按值等价
    match (lang.literal(a), lang.literal(b)) {
        (Some(x), Some(y)) => {
            if x != y {
                return false;
            }
        }
        (None, None) => {}
        _ => return false,
    }
    let (ca, cb) = (lang.children(a), lang.children(b));
    if ca.len() != cb.len() {
        return false;
    }
    ca.iter().zip(cb.iter()).all(|(&x, &y)| print_equiv_inner(lang, x, y))
}

pub struct StaticExec;

impl Rule<JavaAst> for StaticExec {
    fn name(&self) -> &'static str {
        "static_exec"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Block]
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        // 仅根块：嵌套块的裸名存储无法区分外层局部与字段
        if ctx.parent(id).is_some() {
            return None;
        }
        let lang = ctx.lang;
        if lang.kind(id) != NodeKind::Block {
            return None;
        }
        let stmts = lang.children(id).to_vec();
        if stmts.len() < 3 {
            return None; // 少于 3 条语句的段无折叠价值
        }
        // 逐条执行（前缀语义：abort 处截断）
        let mut ex = vexec::Exec::new(lang);
        let mut prefix = 0usize;
        let mut early_exit: Option<vexec::Flow> = None;
        for (i, &st) in stmts.iter().enumerate() {
            match ex.exec_stmt(st) {
                Ok(vexec_flow) => {
                    prefix = i + 1;
                    match vexec_flow {
                        vexec::Flow::Normal => {}
                        other => {
                            // Return/Break 逃逸到顶层：仅允许最后一条语句
                            early_exit = Some(other);
                            if i + 1 != stmts.len() {
                                return None;
                            }
                            break;
                        }
                    }
                }
                Err(()) => break,
            }
        }
        let rest = &stmts[prefix..];
        // 步数预算耗尽：中途状态非收敛值——整段放弃（不回写任何常量）
        if ex.budget_exhausted() {
            return None;
        }
        // 截断守卫：前缀已写字段若在后半段再被赋值/读取 → 放弃整段。
        // 「半途倾倒」陷阱：出口含不可求值调用时执行中止，顶层提升的
        // 字段赋值与残留出口赋值构成 final 双重赋值；倾倒的循环状态
        // （索引/边界/数组）来自不同字符串互不匹配 → AIOOBE
        //（差分审查抓获 r/a4/a5/a3 四文件不可编译/必崩）
        if prefix < stmts.len() {
            let writes: Vec<u32> =
                ex.field_writes().iter().map(|(k, _)| *k).collect();
            if !writes.is_empty() {
                for &k in &writes {
                    if let Some(name) = lang.name_of_key(k) {
                        let re_referenced = rest.iter().any(|&s| {
                            cure_engine::analysis::subtree_contains(lang, s, |n| {
                                lang.kind(n) == NodeKind::VarRef
                                    && lang.var_name(n) == Some(name.as_str())
                            })
                        });
                        if re_referenced {
                            return None;
                        }
                    }
                }
            }
        }
        if prefix < 3 {
            return None;
        }
        // ---- 逃逸分析：后半块句法引用的段内局部（含数组基）----
        // 逃逸局部必须材料化（否则删除前缀会丢失其值/数组内容）
        let mut escaping: Vec<u32> = Vec::new();
        // 只考虑**存活**局部（前缀末仍在作用域）：循环体声明已随块弹出，
        // 后半段对同名者的引用属于**它们自己的**声明（bd.java 的 var682
        // 在 34 个段里同名复用——按 decl_order 全算会假逃逸出 Undef）
        for &k in ex.decl_order() {
            if ex.var_value(k).is_none() {
                continue; // 已随作用域弹出的声明——非本段存活局部
            }
            let name = lang.name_of_key(k)?;
            let referenced = rest.iter().any(|&st| {
                cure_engine::analysis::subtree_contains(lang, st, |n| {
                    lang.kind(n) == NodeKind::VarRef && lang.var_name(n) == Some(name.as_str())
                })
            });
            if referenced {
                escaping.push(k);
            }
        }

        // ---- 材料化输出（先提取数据再 drop 执行器，解除 &mut 借用）----
        // Return 载荷是段输出的一部分：`int m(){…;return x;}` 的 x 必须
        // 物化为 `return <常量>;`（CLI 测试抓获：方法体曾被吞成空体）
        let ret_payload: Option<Option<vexec::VVal>> = match &early_exit {
            Some(vexec::Flow::Return(v)) => Some(v.clone()),
            _ => None,
        };
        let writes: Vec<(u32, vexec::VVal)> =
            ex.field_writes().iter().map(|(k, v)| (*k, v.clone())).collect();
        let escaped: Vec<(u32, vexec::VVal)> = escaping
            .iter()
            .map(|&k| (k, ex.var_value(k).cloned().unwrap_or(vexec::VVal::Undef)))
            .collect();
        drop(ex);
        let mut new_stmts: Vec<JavaId> = Vec::new();
        // 字段写（静态语境裸名赋值；按首次写顺序）
        for (fk, v) in &writes {
            // 参数守卫（方法根：字段名撞参数名 → 放弃）
            let name = lang.name_of_key(*fk)?;
            if lang.is_param_name(&name) {
                return None;
            }
            let (val, mut extra) = materialize(lang, v)?;
            let tgt = lang.var(&name);
            new_stmts.push(lang.assign(tgt, val));
            new_stmts.append(&mut extra);
        }
        // 逃逸局部（保持原声明序）：值不可材料化 → 放弃整段重写
        for (k, v) in &escaped {
            let name = lang.name_of_key(*k)?;
            let (val, mut extra) = materialize(lang, v)?;
            let stmt = lang.var_decl(&name, JType::Var, Some(val));
            new_stmts.push(stmt);
            new_stmts.append(&mut extra);
        }
        // return 语句（载荷物化；void 裸 return 不发——方法体末尾可省，
        // trailing_return 会清理）
        if let Some(v) = &ret_payload.flatten() {
            let (val, mut extra) = materialize(lang, v)?;
            let stmt = lang.ret(Some(val));
            new_stmts.push(stmt);
            new_stmts.append(&mut extra);
        }
        if new_stmts.is_empty() && !escaped.is_empty() {
            return None;
        }
        if new_stmts.is_empty() && escaped.is_empty() && prefix == stmts.len() && early_exit.is_none() {
            // 纯死段（无输出无逃逸）：整段删除
        }
        // 幂等守卫：产物与输入完全一致（同数量同类型同字节）→ 跳过
        //（ASTParserTokenManager 曾每轮 1 次零差编辑——振荡误报）
        if new_stmts.len() == stmts[..prefix].len() {
            let identical = new_stmts
                .iter()
                .zip(stmts[..prefix].iter())
                .all(|(&n, &o)| print_equiv(lang, n, o));
            if identical {
                return None;
            }
        }
        Some(Edit::Splice {
            node: id,
            index: 0,
            remove: prefix,
            insert: new_stmts,
        })
    }
}

/// 值 → 常量字面量节点（不可材料化 → None）。
/// 值 → 常量表达式节点 + 追加语句（部分填充数组需 new T[n] + 分槽赋值）。
fn materialize(lang: &mut JavaAst, v: &vexec::VVal) -> Option<(JavaId, Vec<JavaId>)> {
    use vexec::VVal;
    match v {
        VVal::I(x) => Some((lang.lit(Lit::Int(*x as i64)), vec![])),
        VVal::L(x) => Some((lang.lit(Lit::Long(*x)), vec![])),
        VVal::B(b) => Some((lang.lit(Lit::Bool(*b)), vec![])),
        VVal::S(s) => Some((lang.lit(Lit::Str(s.clone())), vec![])),
        VVal::CA(a) => {
            // char[] 字面量：new char[]{...}
            let chars: Vec<char> = a.borrow().clone();
            let lits: Vec<JavaId> = chars.iter().map(|&c| lang.lit(Lit::Char(c))).collect();
            let lit_arr = lang.array_lit(lits);
            let node = lang.new_array(JType::Char, 1, 0, vec![], Some(lit_arr));
            Some((node, vec![]))
        }
        VVal::SA(elems) => {
            // String[]：全填充 → 字面量数组；部分填充 → new String[n] +
            // 逐元素赋值（null 槽保留——后半段继续填充）
            let arr = elems.borrow().clone();
            if arr.iter().all(|e| matches!(e, VVal::S(_))) {
                let lits: Vec<JavaId> = arr
                    .iter()
                    .map(|e| match e {
                        VVal::S(sv) => lang.lit(Lit::Str(sv.clone())),
                        _ => unreachable!(),
                    })
                    .collect();
                let lit_arr = lang.array_lit(lits);
                let node =
                    lang.new_array(JType::Ref("String".to_string()), 1, 0, vec![], Some(lit_arr));
                Some((node, vec![]))
            } else {
                // 部分填充（含 Undef 槽）：元素静态类型未知——
                // new T[n] 的 T 无法从值推断（差分抓获：new int[5]
                // 曾被物化成 new String[5]，类型损坏不可编译）。
                // 保守放弃（整段重写回退）
                None
            }
        }
        // Undef → 不可材料化（逃逸未初始化值可疑，放弃）
        VVal::Undef => None,
    }
}

pub struct TryUnwrapRethrow;

impl Rule<JavaAst> for TryUnwrapRethrow {
    fn name(&self) -> &'static str {
        "try_unwrap_rethrow"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Try]
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        if lang.kind(id) != NodeKind::Try {
            return None;
        }
        let ch = lang.children(id).to_vec();
        // 布局 [resource…, try_block, catch…, (finally)?]
        let try_block = *ch.iter().find(|&&c| lang.kind(c) == NodeKind::Block)?;
        let catches: Vec<JavaId> =
            ch.iter().copied().filter(|&c| lang.kind(c) == NodeKind::Catch).collect();
        if catches.is_empty() {
            return None;
        }
        let rethrow_only = |c: JavaId| -> bool {
            // Catch 布局 children: [block]；绑定名在 NodeData::Catch
            let block = match lang.children(c).first() {
                Some(&b) => b,
                None => return false,
            };
            let stmts = lang.children(block);
            if stmts.len() != 1 {
                return false;
            }
            let stmt = stmts[0];
            if lang.kind(stmt) != NodeKind::Throw {
                return false;
            }
            let Some(&thrown) = lang.children(stmt).first() else { return false };
            // 抛的必须是 catch 绑定变量自身（整枝唯一使用 ⇒ 无别名逃逸）
            if lang.kind(thrown) != NodeKind::VarRef {
                return false;
            }
            match &lang.node(c).data {
                NodeData::Catch { name, .. } => lang.var_name(thrown) == Some(name.as_str()),
                _ => false,
            }
        };
        let all_rethrow = catches.iter().all(|&c| rethrow_only(c));
        if all_rethrow {
            // 无 finally / 资源（try_block 之外只允许这些 catch）→ 整体剥壳
            let extras: Vec<JavaId> = ch
                .iter()
                .copied()
                .filter(|&c| c != try_block && lang.kind(c) != NodeKind::Catch)
                .collect();
            if extras.is_empty() {
                return Some(Edit::Replace {
                    target: id,
                    with: try_block,
                });
            }
            return None;
        }
        // 混合：只删重抛臂（Splice 掉对应 catch 槽）——定位第一个重抛臂
        for (slot, &c) in ch.iter().enumerate() {
            if c != try_block && lang.kind(c) == NodeKind::Catch && rethrow_only(c) {
                return Some(Edit::Splice {
                    node: id,
                    index: slot,
                    remove: 1,
                    insert: vec![],
                });
            }
        }
        None
    }
}

pub struct TryUnwrapNoCatch;

impl Rule<JavaAst> for TryUnwrapNoCatch {
    fn name(&self) -> &'static str {
        "try_unwrap_no_catch"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Try]
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        if lang.kind(id) != NodeKind::Try {
            return None;
        }
        let ch = lang.children(id).to_vec();
        if ch.is_empty() {
            return None;
        }
        // 布局 [resource…, try_block, catch…, (finally)?]：
        // catch 节点识别（NodeData::Catch）；资源是语句节点（VarDecl/ExprStmt）。
        let has_catch = ch.iter().any(|&c| lang.kind(c) == NodeKind::Catch);
        if has_catch {
            return None;
        }
        // 资源识别：try 块之前、非 try-block 的孩子数 —— 保守做法：
        // try_block = 第一个 Block 孩子（资源不会是 Block）；其后的孩子
        // 只能是 finally（无 catch）。
        let try_block = *ch.iter().find(|&&c| lang.kind(c) == NodeKind::Block)?;
        let rest: Vec<JavaId> = ch
            .iter()
            .copied()
            .filter(|&c| c != try_block)
            .collect();
        // 其余（资源 + finally）必须全部为空 Block（即无 finally 或空 finally）
        // 且无资源语句。
        for &r in &rest {
            let is_empty_block = lang.kind(r) == NodeKind::Block && lang.children(r).is_empty();
            if !is_empty_block {
                return None; // 有资源或非空 finally
            }
        }
        Some(Edit::Replace {
            target: id,
            with: try_block,
        })
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::While, NodeKind::DoWhile]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::While]
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
        let body = wch[1];
        // 前置声明：Iterator<…> it = <iterable>.iterator();
        if lang.kind(parent) != NodeKind::Block {
            return None;
        }
        if idx == 0 {
            return None;
        }
        let stmts = lang.children(parent).to_vec();
        // 向上扫描（最多 4 条）找 it 的 VarDecl（声明与 while 之间可能夹
        // 无关语句，如 acc 声明）；中间语句引用 it 则拒
        let cond = wch[0];
        let it_hint = match lang.kind(cond) {
            NodeKind::Call => {
                let cc0 = lang.children(cond).to_vec();
                match lang.data(*cc0.first()?) {
                    NodeData::Member { name } if lang.sn(*name) == "hasNext" => {
                        lang.var_name(lang.children(cc0[0])[0]).map(|x| x.to_string())
                    }
                    _ => None,
                }
            }
            _ => None,
        };
        let mut decl = None;
        if let Some(hint) = it_hint {
            let mut up = idx;
            for _ in 0..4 {
                if up == 0 {
                    break;
                }
                up -= 1;
                let st = stmts[up];
                if lang.kind(st) == NodeKind::VarDecl && lang.var_name(st) == Some(hint.as_str()) {
                    decl = Some(st);
                    break;
                }
                if subtree_has_var(lang, st, &hint) {
                    break;
                }
            }
        }
        let decl = decl?;
        let it_name = lang.var_name(decl)?.to_string();
        let init_call = lang.children(decl).first().copied()?;
        if lang.kind(init_call) != NodeKind::Call {
            return None;
        }
        let ic = lang.children(init_call).to_vec();
        let NodeData::Member { name: m0 } = lang.data(*ic.first()?) else {
            return None;
        };
        if lang.sn(*m0) != "iterator" || ic.len() != 1 {
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
        if lang.sn(*m1) != "hasNext" || cc.len() != 1 {
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
            if lang.sn(*mn) != "next" {
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
            let elem = if for_each_elem_ok(&elem) {
                elem
            } else {
                JType::Ref("Object".into())
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
                    index: idx,
                    remove: 1,
                    insert: vec![foreach],
                },
                Edit::Delete { node: decl },
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
        if lang.sn(*m2) != "next" || nc.len() != 1 {
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
        // 通配符元素（Iterator<?> → for (? e : …) 非法）→ Object 兜底
        let ty = if for_each_elem_ok(&ty) { ty } else { JType::Ref("Object".into()) };
        let rest = bch[1..].to_vec();
        for &r in &rest {
            if subtree_has_var(&*lang, r, &it_name) {
                return None;
            }
        }
        let new_body = lang.build_block(rest);
        let foreach = lang.for_each(&e_name, ty, iterable, new_body);
        // 用 for-each 替换 while + 删除 it 声明（两处独立位置，
        // 顺序先 Splice（较大索引）后 Delete）
        Some(Edit::Multi(vec![
            Edit::Splice {
                node: parent,
                index: idx,
                remove: 1,
                insert: vec![foreach],
            },
            Edit::Delete { node: decl },
        ]))
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Binary]
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let root = ctx.root();
        let lang = ctx.lang;
        if lang.kind(id) != NodeKind::Binary {
            return None;
        }
        if lang.bin_op(id) != Some(BinOp::Add) {
            return None;
        }
        let ch = lang.children(id).to_vec();
        let (l, r) = (ch[0], ch[1]);
        // 折叠后若成为常量表达式（双方均字面量），拼接串会被 javac 池化，
        // 而原 valueOf(...) 调用是运行期新建 → == 语义会变 → 身份守卫
        let becomes_constant = |x: JavaId, other: JavaId| {
            lang.literal(x).is_some() && matches!(lang.literal(other), Some(LitRef::Str(_)))
        };
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
                if lang.sn(*name) == "valueOf" && lang.var_name(lang.children(callee)[0]) == Some("String") {
                    return Some(c[1]);
                }
            }
            if let NodeData::VarRef { name } = lang.data(callee) {
                if lang.sn(*name) == "valueOf" {
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
                if becomes_constant(x, r) && lang.has_string_identity_compare(root) {
                    return None;
                }
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
                if becomes_constant(x, l) && lang.has_string_identity_compare(root) {
                    return None;
                }
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::VarDecl]
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        // 取结构信息（避免与 ctx 方法借用冲突）
        let parent = ctx.parent(id)?;
        let idx = ctx.index(id)?;
        let root = ctx.root();
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
        if lang.sn(*m0) != "append" {
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
                        if stringy {
                            (Some(a), ic[1])
                        } else if matches!(
                            lang.literal(a),
                            Some(LitRef::Int(_) | LitRef::Long(_))
                        ) {
                            // int/long 字面量必为容量构造——无语义差异
                            (None, ic[1])
                        } else {
                            return None; // 变量实参类型不可证
                        }
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
                    if lang.sn(*mn) != "append" {
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
                    if lang.sn(*mn) != "append" {
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
                    if lang.sn(*mn) != "append" {
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
        if lang.sn(*mn) != "toString" {
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

        // 构造拼接表达式（全字面量链会折叠成常量表达式 → 池化 → 身份守卫）
        let mut all_parts: Vec<JavaId> = Vec::new();
        if let Some(ca) = ctor_arg {
            all_parts.push(ca);
        }
        all_parts.extend(parts);
        if all_parts.iter().all(|&p| lang.literal(p).is_some())
            && lang.has_string_identity_compare(root)
        {
            return None;
        }
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::While, NodeKind::For, NodeKind::DoWhile, NodeKind::ForEach]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Binary]
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Call]
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
        if lang.sn(*name) != "length" {
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Call, NodeKind::Index, NodeKind::Cast]
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let root = ctx.root();
        let lang = ctx.lang;
        let edit = match lang.kind(id) {
            NodeKind::Call => self.eval_call(&mut *lang, id),
            NodeKind::Index => self.eval_index(&mut *lang, id),
            NodeKind::Cast => eval_cast_literal(&mut *lang, id),
            _ => None,
        };
        // Str 字面量结果：原表达式（valueOf/toString/concat/substring…）是运行期
        // 新建的串，折成字面量会引入池化引用；若方法内存在 String 的 ==/!=，
        // 身份语义可能改变 → 守卫。Bool/Int 结果走值语义，不受影响。
        if let Some(Edit::Replace { with, .. }) = &edit {
            if matches!(lang.literal(*with), Some(LitRef::Str(_)))
                && lang.has_string_identity_compare(root)
            {
                if std::env::var("CURE_DEBUG_GATE").is_ok() {
                    eprintln!("[gate] blocked literal_eval at {id:?}");
                }
                return None;
            }
        }
        edit
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
            eval_str_method(s, lang.sn(method), &args)
        } else if let Some(cls) = lang.var_name(recv) {
            eval_static_method(cls, lang.sn(method), &args)
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
        // {"a","b"}[1] / new String[]{"a","b"}[1] → "b"（字面量数组 + 字面量下标，界内）
        let ch = lang.children(id).to_vec();
        // new T[]{…}（解析器归一化形态，sized=0 时唯一子节点是 ArrayLit）
        let arr = match lang.kind(ch[0]) {
            NodeKind::ArrayLit => ch[0],
            NodeKind::NewArray => {
                let nc = lang.children(ch[0]);
                let NodeData::NewArray { sized, .. } = lang.data(ch[0]) else {
                    return None;
                };
                if *sized != 0 || nc.len() != 1 || lang.kind(nc[0]) != NodeKind::ArrayLit {
                    return None;
                }
                nc[0]
            }
            _ => return None,
        };
        let idx = lang.literal(ch[1])?.as_int()?;
        let elems = lang.children(arr).to_vec();
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
        // JLS：字面量编译期即驻留池——"lit".intern() ≡ "lit"（同引用）
        ("intern", []) => Lit::Str(s.to_string()),
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


// ---------------------------------------------------------------------------
// URL 解码还原（混淆器 URL 编码字符串的逆操作）：
//   URLDecoder.decode("%E4%BD%A0", "UTF-8") → "你"
// 守卫：
//   - 两个参数全字面量；UTF-8/Locale.CHINA 等常规字符集名（无 charset
//     的单参形态按 UTF-8——JLS 规定 URLDecoder 无 charset 重载不存在，
//     单参是 6 字节 UTF-8 缺省过时形态，保守拒绝）
//   - 解码产物必须**全部可见**（控制字符/代理位不转——用户规则：
//     不可见字符不转换）
//   - 编码串 ≤ 8KB（超长解码性能保护——用户规则：特别长的跳过）
//   - 解码失败（非法 % 序列/未知 charset）→ 保持原样
// ---------------------------------------------------------------------------


/// Member 节点的成员名（Sym → str）。
fn member_name(ast: &JavaAst, m: JavaId) -> Option<String> {
    match ast.data(m) {
        NodeData::Member { name } => Some(ast.sn(*name).to_string()),
        _ => None,
    }
}

pub struct UrlDecodeFold;

impl Rule<JavaAst> for UrlDecodeFold {
    fn name(&self) -> &'static str {
        "url_decode_fold"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Call]
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        // 形态：URLDecoder.decode(s, charset)——callee 是 Member{decode}，
        // 接收者是 VarRef/Member 链（校验末端名字）
        let ch = lang.children(id).to_vec();
        if ch.len() != 3 {
            return None; // 只处理双参形态
        }
        let callee = ch[0];
        if lang.kind(callee) != NodeKind::Member {
            return None;
        }
        if member_name(lang, callee).as_deref() != Some("decode") {
            return None;
        }
        // 接收者链：URLDecoder（Member{URLDecoder} 或 Member{net.URLDecoder}）
        let recv = *lang.children(callee).first()?;
        let recv_name = match lang.kind(recv) {
            NodeKind::Member => member_name(lang, recv)?,
            NodeKind::VarRef => lang.var_name(recv)?.to_string(),
            _ => return None,
        };
        if !recv_name.ends_with("URLDecoder") {
            return None;
        }
        let s_lit = match lang.literal(ch[1])? {
            LitRef::Str(s) => s.to_string(),
            _ => return None,
        };
        // charset：仅接受 "UTF-8"（其余保守拒绝）
        let cs = match lang.literal(ch[2])? {
            LitRef::Str(s) => s.to_string(),
            _ => return None,
        };
        if cs != "UTF-8" {
            return None;
        }
        // 超长跳过
        if s_lit.len() > 8192 {
            return None;
        }
        let decoded = url_decode(&s_lit)?;
        // 可见性：控制字符/代理位 → 不转换
        if decoded.chars().any(|c| {
            c.is_control() || (c as u32) >= 0xD800 && (c as u32) <= 0xDFFF
        }) {
            return None;
        }
        let lit = lang.lit(Lit::Str(decoded));
        Some(Edit::Replace { target: id, with: lit })
    }
}

/// 手写 URL 解码（%XX + '+' → 空格），UTF-8 字节重组。
/// 非法序列（%后非 hex、孤立 UTF-8 字节）→ None。
fn url_decode(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0usize;
    while i < b.len() {
        match b[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' => {
                if i + 3 > b.len() {
                    return None;
                }
                let hi = hex_val(b[i + 1])?;
                let lo = hex_val(b[i + 2])?;
                out.push(hi * 16 + lo);
                i += 3;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

pub struct Base64NewStringFold;

impl Rule<JavaAst> for Base64NewStringFold {
    fn name(&self) -> &'static str {
        "base64_new_string_fold"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::New]
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let root = ctx.root();
        let lang = ctx.lang;
        let NodeData::New { ty, .. } = lang.data(id) else {
            return None;
        };
        if !matches!(ty, JType::Ref(n) if n == "String" || n.ends_with(".String")) {
            return None;
        }
        // 折叠产出池化字面量，改变 String 引用身份 → 守卫
        if lang.has_string_identity_compare(root) {
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
        if lang.sn(*dn) != "decode" {
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
        let url_safe = match lang.sn(*gn) {
            "getDecoder" => false,
            "getUrlDecoder" => true,
            _ => return None,
        };
        // owner = java.util.Base64（Member 链）或裸 Base64（VarRef）
        let owner = lang.children(gc[0])[0];
        let owner_ok = match lang.data(owner) {
            NodeData::Member { name } => lang.sn(*name) == "Base64",
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
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::While]
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let parent = ctx.parent(id)?;
        let idx = ctx.index(id)?;
        let lang = ctx.lang;
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

        // 入口：向上扫描（最多 8 条）找设置 v = IntLit 的语句；
        // 中间语句不得有 v 事件（读写皆拒）
        if idx == 0 {
            return None;
        }
        let block_ch = lang.children(parent).to_vec();
        let mut init = None;
        let mut init_idx = idx;
        {
            let mut up = idx;
            for _ in 0..8 {
                if up == 0 {
                    break;
                }
                up -= 1;
                let st = block_ch[up];
                match scan_cff_event(lang, st, &v) {
                    Some(CffEvent::Read) => return None,
                    Some(CffEvent::Write) | Some(CffEvent::Decl) => {
                        init = parse_cff_init(lang, &v, st);
                        init_idx = up;
                        break;
                    }
                    None => continue,
                }
            }
        }
        let init = init?;

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

        // 不可达 case 是死代码（从入口不可达 → 永不执行），可安全丢弃；
        // default 未被走到同理。至少要求发射序列非空。
        if stmts.is_empty() {
            return None;
        }

        // 用恢复序列替换 while + 删除 init 语句（两处独立位置：init 与 while
        // 之间的无关语句保留；顺序先 Splice（较大索引）后 Delete）
        Some(Edit::Multi(vec![
            Edit::Splice {
                node: parent,
                index: idx,
                remove: 1,
                insert: stmts,
            },
            Edit::Delete {
                node: block_ch[init_idx],
            },
        ]))
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


/// CFF 入口向上扫描的事件判定（必经路径语义）。
enum CffEvent {
    Read,
    Write,
    /// v 自身的声明（含 init 时为入口源）
    Decl,
}

fn scan_cff_event(lang: &JavaAst, node: JavaId, name: &str) -> Option<CffEvent> {
    match lang.kind(node) {
        NodeKind::VarRef => {
            if lang.var_name(node) == Some(name) {
                Some(CffEvent::Read)
            } else {
                None
            }
        }
        NodeKind::VarDecl => {
            if lang.var_name(node) == Some(name) {
                Some(CffEvent::Decl)
            } else {
                for &c in lang.children(node) {
                    if let Some(e) = scan_cff_event(lang, c, name) {
                        return Some(e);
                    }
                }
                None
            }
        }
        NodeKind::Assign => {
            let ch = lang.children(node);
            if let Some(&val) = ch.get(1) {
                if let Some(e) = scan_cff_event(lang, val, name) {
                    if matches!(e, CffEvent::Read) {
                        return Some(e);
                    }
                }
            }
            if lang.assign_op(node).is_none() {
                if let Some(&t) = ch.first() {
                    if lang.kind(t) == NodeKind::VarRef && lang.var_name(t) == Some(name) {
                        return Some(CffEvent::Write);
                    }
                }
            }
            None
        }
        NodeKind::If | NodeKind::While => scan_cff_event(lang, *lang.children(node).first()?, name),
        NodeKind::Ternary => scan_cff_event(lang, *lang.children(node).first()?, name),
        NodeKind::Binary => {
            let op = lang.bin_op(node)?;
            let ch = lang.children(node);
            if op.is_short_circuit() {
                scan_cff_event(lang, *ch.first()?, name)
            } else {
                for &c in ch {
                    if let Some(e) = scan_cff_event(lang, c, name) {
                        return Some(e);
                    }
                }
                None
            }
        }
        _ => {
            for &c in lang.children(node) {
                if let Some(e) = scan_cff_event(lang, c, name) {
                    return Some(e);
                }
            }
            None
        }
    }
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
// 跨方法常量知识（string-array 解密器的过程间分析入口）：
// simplify_unit 把类级常量字段与单 return 方法体填入 JavaAst 侧表，
// 下面两条规则在 fixed point 中消费，与部分求值/数组下标/Base64 折叠
// 自然组合：d(0) → 方法体 → T[0] → "c3Vw" → Base64 解码 → "sup"。
// ---------------------------------------------------------------------------

/// 收集类级常量：static final 字面量/数组字段（无写、无局部同名）
/// 与单 return 方法体。
pub fn collect_unit_consts(ast: &mut JavaAst, unit: &CompilationUnit) {
    ast.const_fields.clear();
    ast.inline_methods.clear();

    // 第一遍：所有体内对字段名的写与局部/参数同名 → 污染集合
    let mut tainted: std::collections::HashSet<String> = Default::default();
    let mut bodies: Vec<JavaId> = Vec::new();
    for ty in &unit.types {
        for m in &ty.members {
            match m {
                Member::Method { body: Some(b), params, .. }
                | Member::Constructor { body: Some(b), params, .. } => {
                    bodies.push(*b);
                    for p in params {
                        tainted.insert(p.name.clone());
                    }
                }
                Member::Initializer { body, .. } => bodies.push(*body),
                Member::Field { declarators, .. } => {
                    for d in declarators {
                        if let Some(init) = d.init {
                            bodies.push(init);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    // 污染扫描：字段写（含数组元素写/incdec）+ 局部同名声明（遮蔽）
    for &root in &bodies {
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            if ast.kind(n) == NodeKind::VarDecl {
                if let Some(nm) = ast.var_name(n) {
                    tainted.insert(nm.to_string());
                }
            }
            match ast.data(n) {
                NodeData::Assign { .. } => {
                    let t = ast.children(n)[0];
                    let mut ts = vec![t];
                    while let Some(x) = ts.pop() {
                        if ast.kind(x) == NodeKind::VarRef {
                            if let Some(nm) = ast.var_name(x) {
                                tainted.insert(nm.to_string());
                            }
                        }
                        for &c in ast.children(x) {
                            ts.push(c);
                        }
                    }
                }
                NodeData::Unary { op } if op.is_incdec() => {
                    let t = ast.children(n)[0];
                    if ast.kind(t) == NodeKind::VarRef {
                        if let Some(nm) = ast.var_name(t) {
                            tainted.insert(nm.to_string());
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

    // 第二遍：常量字段收集
    for ty in &unit.types {
        for m in &ty.members {
            if let Member::Field { mods, declarators, .. } = m {
                if !mods.contains("final") {
                    continue;
                }
                for d in declarators {
                    if let Some(init) = d.init {
                        if is_const_init(&ast, init) && !tainted.contains(&d.name) {
                            if ast.const_fields.contains_key(&d.name) {
                                ast.const_fields.remove(&d.name);
                            } else {
                                ast.const_fields.insert(d.name.clone(), init);
                            }
                        }
                    }
                }
            }
        }
    }

    // 单 return 方法收集
    for ty in &unit.types {
        for m in &ty.members {
            if let Member::Method { name, params, body: Some(b), .. } = m {
                let ch = ast.children(*b).to_vec();
                if ch.len() == 1 && ast.kind(ch[0]) == NodeKind::Return {
                    if let Some(&expr) = ast.children(ch[0]).first() {
                        if params.len() <= 3
                            && !subtree_calls_self(&ast, expr, name)
                            // 类型敏感守卫：字面量替换会改变形参位置的静态类型
                            // （switch 模式选择器 / instanceof 被测式 / Raw 不
                            // 可见构造）——`match(42)` 内联成 switch(42) 遭
                            // javac 拒绝（Adv6 差分抓获）。混淆 helper（纯算
                            // 术/字符串体）不受影响。
                            && !subtree_has_type_sensitive(&ast, expr)
                        {
                            let ps: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                            if ast.inline_methods.contains_key(name) {
                                ast.inline_methods.remove(name);
                            } else {
                                ast.inline_methods.insert(name.clone(), (ps, expr));
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 常量初始化判定：字面量，或字面量数组（new T[]{…} / {…}）。
fn is_const_init(ast: &JavaAst, init: JavaId) -> bool {
    match ast.kind(init) {
        NodeKind::Literal => true,
        NodeKind::ArrayLit => ast.children(init).iter().all(|&e| ast.kind(e) == NodeKind::Literal),
        NodeKind::NewArray => {
            let ch = ast.children(init).to_vec();
            ch.len() == 1
                && ast.kind(ch[0]) == NodeKind::ArrayLit
                && ast.children(ch[0]).iter().all(|&e| ast.kind(e) == NodeKind::Literal)
        }
        _ => false,
    }
}

/// 子树含类型敏感构造（switch / instanceof / Raw）：字面量替换形参
/// 会改变这些位置的静态类型 → 编译器拒绝（`match(42)` → `switch(42)`
/// 遭 javac 否决，Adv6 差分抓获）。这类方法不进内联候选。
fn subtree_has_type_sensitive(ast: &JavaAst, root: JavaId) -> bool {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        match ast.kind(n) {
            NodeKind::Switch | NodeKind::InstanceOf | NodeKind::Raw => return true,
            _ => {}
        }
        for &c in ast.children(n) {
            stack.push(c);
        }
    }
    false
}

fn subtree_calls_self(ast: &JavaAst, root: JavaId, name: &str) -> bool {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if ast.kind(n) == NodeKind::Call {
            let callee = ast.children(n)[0];
            if ast.var_name(callee) == Some(name) {
                return true;
            }
            // 成员调用名也算（Boolean.getBoolean——jdk-sources TCPEndpoint
            // 抓获：库方法与本地方法恰好同名时，匹配器按
            // "大写接收方.方法名" 也会命中 → 自我乒乓无限克隆 → 爆栈；
            // 保守拒绝（误拒仅损失一次内联机会）
            if let NodeData::Member { name: m } = ast.data(callee) {
                if ast.sn(*m) == name {
                    return true;
                }
            }
        }
        for &c in ast.children(n) {
            stack.push(c);
        }
    }
    false
}

// ---------------------------------------------------------------------------
// 常量字段下标折叠：T[i]（T 为字面量数组常量字段）→ 元素字面量
// ---------------------------------------------------------------------------

pub struct StaticArrayIndexFold;

impl Rule<JavaAst> for StaticArrayIndexFold {
    fn name(&self) -> &'static str {
        "static_array_index_fold"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Index]
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        if lang.kind(id) != NodeKind::Index {
            return None;
        }
        let ch = lang.children(id).to_vec();
        if lang.kind(ch[0]) != NodeKind::VarRef {
            return None;
        }
        let name = lang.var_name(ch[0])?.to_string();
        let idx = lang.literal(ch[1]).and_then(|x| x.as_int())?;
        let init = *lang.const_fields.get(&name)?;
        let arr = match lang.kind(init) {
            NodeKind::ArrayLit => init,
            NodeKind::NewArray => {
                let ic = lang.children(init).to_vec();
                if ic.len() == 1 && lang.kind(ic[0]) == NodeKind::ArrayLit {
                    ic[0]
                } else {
                    return None;
                }
            }
            _ => return None,
        };
        let elems = lang.children(arr).to_vec();
        if idx < 0 || idx >= elems.len() as i64 {
            return None;
        }
        let elem = lang.copy_subtree(elems[idx as usize]);
        Some(Edit::Replace {
            target: id,
            with: elem,
        })
    }
}

// ---------------------------------------------------------------------------
// 常量实参方法内联（解密 helper）：d(0) → 方法体（参数→实参字面量替换）。
// 语义恒安全：单 return 表达式在调用点求值一次，两种形态一致；
// 自递归方法拒绝；实参须全为字面量。
// 【结构性规则】豁免成本门槛：内联暂时变贵、由后续常量折叠回本。
// 终止性：守卫"体内不含任何内联名的裸调用"——每次内联严格消费一个
// 此类调用点且不引入新的同类节点。
// ---------------------------------------------------------------------------

pub struct ConstMethodInline;

impl Rule<JavaAst> for ConstMethodInline {
    fn name(&self) -> &'static str {
        "const_method_inline"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Call]
    }
    fn structural(&self) -> bool {
        true
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let lang = ctx.lang;
        if lang.kind(id) != NodeKind::Call {
            return None;
        }
        let ch = lang.children(id).to_vec();
        let method_name = match lang.data(ch[0]) {
            NodeData::VarRef { name } => name.clone(),
            NodeData::Member { name } => {
                let recv = lang.children(ch[0])[0];
                match lang.var_name(recv) {
                    Some(r) if r == "this" || r.chars().next().is_some_and(|c| c.is_uppercase()) => {
                        name.clone()
                    }
                    _ => return None,
                }
            }
            _ => return None,
        };
        let entry = lang.inline_methods.get(lang.sn(method_name)).cloned()?;
        let (params, body_expr) = entry;
        // 终止守卫：方法体内不得含任何【内联名】的裸调用
        {
            let names: Vec<String> = lang.inline_methods.keys().cloned().collect();
            for nm in &names {
                if subtree_calls_self(lang, body_expr, nm) {
                    return None;
                }
            }
        }
        let args = ch[1..].to_vec();
        if args.len() != params.len() || params.is_empty() {
            return None;
        }
        for &a in &args {
            if lang.literal(a).is_none() {
                return None;
            }
        }
        let map: std::collections::HashMap<String, JavaId> = params
            .iter()
            .zip(args.iter())
            .map(|(p, &a)| (p.clone(), a))
            .collect();
        let with = copy_subst(lang, body_expr, &map);
        Some(Edit::Replace {
            target: id,
            with,
        })
    }
}

/// 深拷贝 + VarRef 参数替换。
fn copy_subst(
    lang: &mut JavaAst,
    node: JavaId,
    map: &std::collections::HashMap<String, JavaId>,
) -> JavaId {
    if let NodeData::VarRef { name } = lang.data(node) {
        if let Some(&repl) = map.get(lang.sn(*name)) {
            return lang.copy_subtree(repl);
        }
    }
    let children: Vec<JavaId> = lang.children(node).to_vec();
    let new_children: Vec<JavaId> = children
        .iter()
        .map(|&c| copy_subst(lang, c, map))
        .collect();
    lang.clone_node(node, new_children)
}


// ---------------------------------------------------------------------------
// try-with-resources 还原（twr_recover）：两种反编译器直出形态。
//
// family B（ddc：javac→d8 字节码的紧凑渲染）：
//   R r = new R(...);            // 资源（new——无卫语句形态下唯一非空证明）
//   try { B }
//   catch (Throwable t) {
//       try { r.close(); }
//       catch (Throwable sup) { t.addSuppressed(sup); }
//       throw t;
//   }
//   r.close();                   // 正常路径尾部直调
//   → try (R r = new R(...)) { B }
//
// family A（javac 源级翻译；CFR/ProGuard 常见）：
//   R r = init; Throwable primary = null;
//   try { B }
//   catch (Throwable t) { primary = t; throw t; }
//   finally {
//       if (r != null) {
//           if (primary != null) {
//               try { r.close(); } catch (Throwable sup) { primary.addSuppressed(sup); }
//           } else { r.close(); }
//       }
//   }
//   → try (R r = init) { B }      // 含 null 资源跳过 close 的路径（卫语句保证）
//
// 多资源：嵌套形态由不动点逐层还原（内层先→外层 try 体含 try(res)——
// JLS 上嵌套 TWR ≡ 多资源 TWR，无需合并）。守卫：r/primary/t/sup 的全部
// 引用必须落在模式消费的子树内（否则不还原）；同名遮蔽一律拒绝。
// ---------------------------------------------------------------------------

pub struct TwrRecover;

fn twr_is_throwable(ty_raw: &str) -> bool {
    ty_raw == "Throwable" || ty_raw == "java.lang.Throwable"
}

/// 表达式语句解包：ExprStmt{e} → e（赋值/调用等语句位置）。
fn unwrap_expr_stmt(lang: &JavaAst, n: JavaId) -> JavaId {
    if lang.kind(n) == NodeKind::ExprStmt {
        if let Some(&e) = lang.children(n).first() {
            return e;
        }
    }
    n
}

/// 语句序列（Block 展开；单语句原样）。
fn twr_stmts(lang: &JavaAst, n: JavaId) -> Vec<JavaId> {
    if lang.kind(n) == NodeKind::Block {
        lang.children(n).to_vec()
    } else {
        vec![n]
    }
}

/// `r.close()` 调用表达式（ExprStmt 解包后）。
fn twr_is_close_call(lang: &JavaAst, e: JavaId, r: &str) -> bool {
    if lang.kind(e) != NodeKind::Call {
        return false;
    }
    let ch = lang.children(e);
    if ch.len() != 1 {
        return false;
    }
    match lang.data(ch[0]) {
        NodeData::Member { name } if lang.sn(*name) == "close" => {
            let Some(&recv) = lang.children(ch[0]).first() else {
                return false;
            };
            lang.kind(recv) == NodeKind::VarRef && lang.var_name(recv) == Some(r)
        }
        _ => false,
    }
}

fn twr_is_close_stmt(lang: &JavaAst, n: JavaId, r: &str) -> bool {
    if lang.kind(n) != NodeKind::ExprStmt {
        return false;
    }
    lang.children(n)
        .first()
        .is_some_and(|&e| twr_is_close_call(lang, e, r))
}

/// `primary.addSuppressed(sup)` 语句。
fn twr_is_add_suppressed(lang: &JavaAst, n: JavaId, primary: &str, sup: &str) -> bool {
    if lang.kind(n) != NodeKind::ExprStmt {
        return false;
    }
    let Some(&e) = lang.children(n).first() else {
        return false;
    };
    if lang.kind(e) != NodeKind::Call {
        return false;
    }
    let ch = lang.children(e);
    if ch.len() != 2 {
        return false;
    }
    match lang.data(ch[0]) {
        NodeData::Member { name } if lang.sn(*name) == "addSuppressed" => {
            let Some(&recv) = lang.children(ch[0]).first() else {
                return false;
            };
            lang.kind(recv) == NodeKind::VarRef
                && lang.var_name(recv) == Some(primary)
                && lang.kind(ch[1]) == NodeKind::VarRef
                && lang.var_name(ch[1]) == Some(sup)
        }
        _ => false,
    }
}

fn node_contains(lang: &JavaAst, root: JavaId, target: JavaId) -> bool {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if n == target {
            return true;
        }
        stack.extend(lang.children(n).iter().copied());
    }
    false
}

/// root 内存在 name 的引用且不在任何 allowed 子树内（模式外的逃逸引用）。
fn refs_escape(lang: &JavaAst, root: JavaId, name: &str, allowed: &[JavaId]) -> bool {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if lang.kind(n) == NodeKind::VarRef && lang.var_name(n) == Some(name) {
            let ok = allowed
                .iter()
                .any(|&a| a == n || node_contains(lang, a, n));
            if !ok {
                return true;
            }
        }
        if lang.kind(n) == NodeKind::VarDecl {
            if let Some(nm) = lang.var_name(n) {
                if nm == name && !allowed.contains(&n) {
                    // 同名再声明（遮蔽风险）→ 拒绝
                    return true;
                }
            }
        }
        stack.extend(lang.children(n).iter().copied());
    }
    false
}

/// Try{try 块, 单 catch(Throwable, sup), 无 finally} 且 catch 体 =
/// [addSuppressed(primary, sup)]、try 体 = [r.close()]。
/// 返回 (r 名, sup 名)。primary 传闭包外确定。
fn twr_close_with_suppressed(
    lang: &JavaAst,
    try_node: JavaId,
    primary: &str,
) -> Option<(String, String)> {
    let ch = lang.children(try_node).to_vec();
    if ch.len() != 2 || lang.kind(ch[0]) != NodeKind::Block {
        return None;
    }
    let NodeData::Catch { ty_raw, name: sup } = lang.data(ch[1]) else {
        return None;
    };
    if !twr_is_throwable(ty_raw) {
        return None;
    }
    let sup = sup.clone();
    let catch_stmts = twr_stmts(lang, *lang.children(ch[1]).first()?);
    if catch_stmts.len() != 1 || !twr_is_add_suppressed(lang, catch_stmts[0], primary, &sup) {
        return None;
    }
    let close_stmts = twr_stmts(lang, ch[0]);
    if close_stmts.len() != 1 {
        return None;
    }
    if lang.kind(close_stmts[0]) != NodeKind::ExprStmt {
        return None;
    }
    let e = *lang.children(close_stmts[0]).first()?;
    let cc = lang.children(e);
    if cc.len() != 1 {
        return None;
    }
    let NodeData::Member { name } = lang.data(cc[0]) else {
        return None;
    };
    if lang.sn(*name) != "close" {
        return None;
    }
    let recv = *lang.children(cc[0]).first()?;
    if lang.kind(recv) != NodeKind::VarRef {
        return None;
    }
    Some((lang.var_name(recv)?.to_string(), sup))
}

impl Rule<JavaAst> for TwrRecover {
    fn name(&self) -> &'static str {
        "twr_recover"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Try]
    }
    fn structural(&self) -> bool {
        true
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        // 先取 ctx 查询（lang 的 &mut 移出后 ctx 不可再用）
        let (parent, idx, walk_root) = (ctx.parent(id)?, ctx.index(id)?, ctx.root());
        let lang = ctx.lang;
        if lang.kind(id) != NodeKind::Try {
            return None;
        }
        let ch = lang.children(id).to_vec();
        // 资源已存在的（嵌套已还原）不再匹配
        let try_idx = ch
            .iter()
            .position(|&c| lang.kind(c) == NodeKind::Block)?;
        if try_idx != 0 {
            return None;
        }
        let try_block = ch[0];
        if lang.kind(parent) != NodeKind::Block {
            return None;
        }
        let pch = lang.children(parent).to_vec();

        // ===== family B：[decl_r, try{B}catch, close] =====
        if ch.len() == 2 {
            if let NodeData::Catch { ty_raw, name: t } = lang.data(ch[1]) {
                if twr_is_throwable(ty_raw) {
                    let t = t.clone();
                    let catch_stmts = twr_stmts(lang, *lang.children(ch[1]).first()?);
                    if catch_stmts.len() == 2 {
                        // [Try{close,sup}, throw t]
                        let (inner_try, throw_stmt) = (catch_stmts[0], catch_stmts[1]);
                        let throw_ok = lang.kind(throw_stmt) == NodeKind::Throw
                            && lang
                                .children(throw_stmt)
                                .first()
                                .is_some_and(|&x| {
                                    lang.kind(x) == NodeKind::VarRef
                                        && lang.var_name(x) == Some(t.as_str())
                                });
                        if throw_ok && lang.kind(inner_try) == NodeKind::Try {
                            if let Some((r, sup)) =
                                twr_close_with_suppressed(lang, inner_try, t.as_str())
                            {
                                if idx >= 1 && idx + 1 < pch.len() {
                                    let decl_r = pch[idx - 1];
                                    let trailing = pch[idx + 1];
                                    if let NodeData::VarDecl { name, ty } = lang.data(decl_r) {
                                        if name == &r {
                                            let (name, ty) = (name.clone(), ty.clone());
                                            let init =
                                                lang.children(decl_r).first().copied();
                                            if let Some(init) = init {
                                                // 无卫语句形态：init 须为 new（非空证明）
                                                let provably_new = lang.kind(init)
                                                    == NodeKind::New;
                                                let close_ok =
                                                    twr_is_close_stmt(lang, trailing, &r);
                                                if provably_new && close_ok {
                                                    // 引用逃逸扫描
                                                    let root = walk_root;
                                                    let r_ok = !refs_escape(
                                                        lang,
                                                        root,
                                                        &r,
                                                        &[
                                                            decl_r,
                                                            init,
                                                            *lang
                                                                .children(inner_try)
                                                                .first()
                                                                .unwrap_or(&inner_try),
                                                            trailing,
                                                            try_block,
                                                        ],
                                                    );
                                                    let t_ok = !refs_escape(
                                                        lang, root, &t,
                                                        &[throw_stmt, ch[1]],
                                                    );
                                                    let sup_ok = !refs_escape(
                                                        lang, root, &sup,
                                                        &[ch[1]],
                                                    );
                                                    if r_ok && t_ok && sup_ok {
                                                        let init_copy =
                                                            lang.copy_subtree(init);
                                                        let res = lang.var_decl(
                                                            &name, ty,
                                                            Some(init_copy),
                                                        );
                                                        let new_try = lang.try_(
                                                            vec![res],
                                                            try_block,
                                                            Vec::new(),
                                                            None,
                                                        );
                                                        return Some(Edit::Splice {
                                                            node: parent,
                                                            index: idx - 1,
                                                            remove: 3,
                                                            insert: vec![new_try],
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // ===== family A：[decl_r, decl_primary, try{B}catch{primary=t;throw}finally{卫语句}] =====
        if ch.len() == 3 {
            let NodeData::Catch { ty_raw, name: t } = lang.data(ch[1]) else {
                return None;
            };
            if !twr_is_throwable(ty_raw) {
                return None;
            }
            let t = t.clone();
            // finally 块
            if lang.kind(ch[2]) != NodeKind::Block {
                return None;
            }
            let fin_stmts = twr_stmts(lang, ch[2]);
            if fin_stmts.len() != 1 || lang.kind(fin_stmts[0]) != NodeKind::If {
                return None;
            }
            // catch 体 = [primary = t, throw t]
            let catch_stmts = twr_stmts(lang, *lang.children(ch[1]).first()?);
            if catch_stmts.len() != 2 {
                return None;
            }
            let (assign_stmt, throw_stmt) = (catch_stmts[0], catch_stmts[1]);
            let assign_expr = unwrap_expr_stmt(lang, assign_stmt);
            let NodeData::Assign { op: None } = lang.data(assign_expr) else {
                return None;
            };
            let as_ch = lang.children(assign_expr);
            if as_ch.len() != 2 {
                return None;
            }
            let primary = match lang.data(as_ch[0]) {
                NodeData::VarRef { name } => lang.sn(*name).to_string(),
                _ => return None,
            };
            let assign_ok = lang.kind(as_ch[1]) == NodeKind::VarRef
                && lang.var_name(as_ch[1]) == Some(t.as_str())
                && lang.kind(as_ch[0]) == NodeKind::VarRef
                && lang.var_name(as_ch[0]) == Some(primary.as_str());
            let throw_ok = lang.kind(throw_stmt) == NodeKind::Throw
                && lang.children(throw_stmt)
                    .first()
                    .is_some_and(|&x| {
                        lang.kind(x) == NodeKind::VarRef
                            && lang.var_name(x) == Some(t.as_str())
                    });
            if !assign_ok || !throw_ok {
                return None;
            }
            // 卫语句：if (r != null) { if (primary != null) { try{r.close()}catch{sup} } else { r.close() } }
            let if1 = fin_stmts[0];
            let i1 = lang.children(if1).to_vec();
            if i1.len() < 2 {
                return None;
            }
            let cond1 = i1[0];
            let NodeData::Binary { op: BinOp::Ne } = lang.data(cond1) else {
                return None;
            };
            let c1 = lang.children(cond1);
            let r = match lang.data(c1[0]) {
                NodeData::VarRef { name } if lang.literal(c1[1]) == Some(LitRef::Null) => {
                    lang.sn(*name).to_string()
                }
                _ => {
                    return None;
                }
            };
            let then1 = i1[1];
            let inner_stmts = twr_stmts(lang, then1);
            if inner_stmts.len() != 1 || lang.kind(inner_stmts[0]) != NodeKind::If {
                return None;
            }
            let if2 = inner_stmts[0];
            let i2 = lang.children(if2).to_vec();
            if i2.len() != 3 {
                return None;
            }
            let NodeData::Binary { op: BinOp::Ne } = lang.data(i2[0]) else {
                return None;
            };
            let c2 = lang.children(i2[0]);
            let primary_ok = lang.kind(c2[0]) == NodeKind::VarRef
                && lang.var_name(c2[0]) == Some(primary.as_str())
                && lang.literal(c2[1]) == Some(LitRef::Null);
            if !primary_ok {
                return None;
            }
            let (then2, els2) = (i2[1], i2[2]);
            // then2: Try{[r.close()], catch{primary.addSuppressed(sup)}}
            let t2_stmts = twr_stmts(lang, then2);
            if t2_stmts.len() != 1 || lang.kind(t2_stmts[0]) != NodeKind::Try {
                return None;
            }
            let Some((r2, sup)) =
                twr_close_with_suppressed(lang, t2_stmts[0], primary.as_str())
            else {
                return None;
            };
            if r2 != r {
                return None;
            }
            // els2: [r.close()]
            let e2_stmts = twr_stmts(lang, els2);
            if e2_stmts.len() != 1 || !twr_is_close_stmt(lang, e2_stmts[0], &r) {
                return None;
            }
            // 父块三连：decl_r, decl_primary, try
            if idx < 2 {
                return None;
            }
            let decl_r = pch[idx - 2];
            let decl_primary = pch[idx - 1];
            let (r_decl, primary_decl) = match (lang.data(decl_r), lang.data(decl_primary)) {
                (
                    NodeData::VarDecl { name: rn, .. },
                    NodeData::VarDecl { name: pn, .. },
                ) => (rn.clone(), pn.clone()),
                _ => return None,
            };
            if r_decl != r || primary_decl != primary {
                return None;
            }
            // primary 初始化为 null
            let pinit_null = lang
                .children(decl_primary)
                .first()
                .is_some_and(|&n| lang.literal(n) == Some(LitRef::Null));
            if !pinit_null {
                return None;
            }
            let r_init = lang.children(decl_r).first().copied()?;
            let ty = match lang.data(decl_r) {
                NodeData::VarDecl { ty, .. } => ty.clone(),
                _ => return None,
            };
            // 引用逃逸扫描
            let root = walk_root;
            // finally 整体被消费：卫语句内的 r/primary 引用均合法
            let r_allowed = [decl_r, r_init, ch[2], try_block];
            let t_allowed = [throw_stmt, assign_stmt, ch[1], ch[2]];
            let p_allowed = [decl_primary, assign_stmt, ch[2]];
            if refs_escape(lang, root, &r, &r_allowed)
                || refs_escape(lang, root, &primary, &p_allowed)
                || refs_escape(lang, root, &t, &t_allowed)
                || refs_escape(lang, root, &sup, &[ch[2]])
            {
                return None;
            }
            let init_copy = lang.copy_subtree(r_init);
            let res = lang.var_decl(&r, ty, Some(init_copy));
            let new_try = lang.try_(vec![res], try_block, Vec::new(), None);
            return Some(Edit::Splice {
                node: parent,
                index: idx - 2,
                remove: 3,
                insert: vec![new_try],
            });
        }
        None
    }
}

// ---------------------------------------------------------------------------
// 字符串 switch 还原（string_switch_recover）：javac v7 索引二级 switch
// 形态（ddc 直出；javac 字节码本相——hashCode 定位 + equals 守卫 + 索引
// switch 分发）：
//   int v7 = 0;
//   switch (s.hashCode()) {
//       case H_i: {
//           boolean e = s.equals("L_i");
//           if (!e) { break; } else {
//               v7 = K_i;
//               switch (v7) { case K_1: BODY_1; ... default: BODY_DEF; }
//           }
//       } ...
//   }
//   v7 = -1;
//   switch (v7) { case K_1: BODY_1; ... default: BODY_DEF; }   // 无匹配路径
//   → switch (s) { case "L_1": BODY_1; ... default: BODY_DEF; }
//
// 守卫：选择器与 equals 接收方同为纯 VarRef（一次求值语义）；
// 各索引 switch（含尾部）签名全同；每个 BODY 以终止语句收尾（无落穿，
// 标签序无关）；v/e 引用全部落在被消费子树内；H_i ≡ "L_i".hashCode()。
// 哈希碰撞（同 case 双守卫）形态更复杂 → 整体不还原（负控锁定）。
// ---------------------------------------------------------------------------

pub struct StringSwitchRecover;

fn subtree_sig(lang: &JavaAst, n: JavaId, out: &mut String) {
    out.push_str(&format!("{:?} ", lang.data(n)));
    for &c in lang.children(n) {
        subtree_sig(lang, c, out);
    }
}

/// 从 v7 索引 switch 提取 case K → 语句、default → 语句。
fn ssw_bodies(lang: &JavaAst, sw: JavaId) -> Option<Vec<(Option<i64>, Vec<JavaId>)>> {
    let ch = lang.children(sw).to_vec();
    let mut out = Vec::new();
    for &c in &ch[1..] {
        let NodeData::Case { labels, is_default, .. } = lang.data(c) else {
            return None;
        };
        let is_default = *is_default;
        let cc = lang.children(c);
        let label = if is_default {
            None
        } else {
            if *labels != 1 {
                return None; // 多标签 → 复杂形态，不还原
            }
            let lab = cc[0];
            let lit = lang.literal(lab)?;
            let v = match lit {
                LitRef::Int(v) | LitRef::Long(v) => v,
                _ => return None, // 非整型标签（字符串 case 是还原产物/负控）
            };
            Some(v)
        };
        out.push((label, cc[*labels as usize..].to_vec()));
    }
    Some(out)
}

fn ssw_body_terminal(lang: &JavaAst, stmts: &[JavaId]) -> bool {
    let Some(&last) = stmts.last() else {
        return false; // 空体无终止
    };
    let last = if lang.kind(last) == NodeKind::Block {
        match lang.children(last).last() {
            Some(&l) => l,
            None => return false,
        }
    } else {
        last
    };
    matches!(
        lang.kind(last),
        NodeKind::Return | NodeKind::Break | NodeKind::Throw | NodeKind::Continue
    )
}

impl Rule<JavaAst> for StringSwitchRecover {
    fn name(&self) -> &'static str {
        "string_switch_recover"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Switch]
    }
    fn structural(&self) -> bool {
        true
    }
    fn check(&self, ctx: RewriteCtx<'_, JavaAst>, id: JavaId) -> Option<Edit<JavaAst>> {
        let (parent, idx, ctx_root) = (ctx.parent(id)?, ctx.index(id)?, ctx.root());
        let lang = ctx.lang;
        if lang.kind(id) != NodeKind::Switch {
            return None;
        }
        let ch = lang.children(id).to_vec();
        if ch.len() < 2 {
            return None;
        }
        // 选择器 x.hashCode()
        let x = match lang.data(ch[0]) {
            NodeData::Call { .. } => {
                let cc = lang.children(ch[0]);
                if cc.len() != 1 {
                    return None;
                }
                match lang.data(cc[0]) {
                    NodeData::Member { name } if lang.sn(*name) == "hashCode" => {
                        let recv = *lang.children(cc[0]).first()?;
                        if lang.kind(recv) != NodeKind::VarRef {
                            return None;
                        }
                        lang.var_name(recv)?.to_string()
                    }
                    _ => return None,
                }
            }
            _ => {
                return None;
            }
        };
        // 各 hash case：单整型标签 + [decl_e?, if(!guard) break else {v=K, switch(v)}]
        struct CaseInfo {
            lit: String,
            k: i64,
            inner_switch: JavaId,
        }
        let mut infos: Vec<CaseInfo> = Vec::new();
        let mut v_name: Option<String> = None;
        let mut v_nodes: Vec<JavaId> = Vec::new(); // v 的全部合法出现（供逃逸扫描）
        let mut guard_nodes: Vec<JavaId> = Vec::new();
        for &c in &ch[1..] {
            let NodeData::Case { labels, is_default, .. } = lang.data(c) else {
                return None;
            };
            let is_default = *is_default;
            if is_default {
                return None; // hash switch 不应有 default（无匹配由尾部处理）
            }
            if *labels != 1 {
                return None;
            }
            let cc = lang.children(c);
            let h = match lang.literal(cc[0])? {
                LitRef::Int(v) | LitRef::Long(v) => v as i32,
                _ => return None,
            };
            let body = cc[1..].to_vec();
            // 单 Block 解包
            let body = if body.len() == 1 && lang.kind(body[0]) == NodeKind::Block {
                lang.children(body[0]).to_vec()
            } else {
                body
            };
            let mut rest = body;
            // 可选物化 boolean
            let mut _e_decl: Option<(String, JavaId)> = None;
            if let Some(&first) = rest.first() {
                if let NodeData::VarDecl { name, .. } = lang.data(first) {
                    if let Some(&init) = lang.children(first).first() {
                        if is_x_equals(lang, init, &x) {
                            _e_decl = Some((name.clone(), first));
                            rest = rest[1..].to_vec();
                        }
                    }
                }
            }
            if rest.len() != 1 || lang.kind(rest[0]) != NodeKind::If {
                return None;
            }
            let ifn = rest[0];
            let ic = lang.children(ifn).to_vec();
            if ic.len() != 3 {
                return None;
            }
            // cond = !guard（guard = e 或 x.equals(L)）
            let NodeData::Unary { op: UnOp::Not } = lang.data(ic[0]) else {
                return None;
            };
            let guard = *lang.children(ic[0]).first()?;
            let lit = match lang.data(guard) {
                NodeData::VarRef { name } => {
                    // 物化 boolean：其唯一定义须是 x.equals("L")
                    let Some((en, decl)) = &_e_decl else {
                        return None;
                    };
                    if lang.sn(*name) != en {
                        return None;
                    }
                    let init = *lang.children(*decl).first()?;
                    equals_lit(lang, init, &x)?
                }
                NodeData::Call { .. } => equals_lit(lang, guard, &x)?,
                _ => return None,
            };
            guard_nodes.push(guard);
            if let Some((_, decl)) = &_e_decl {
                guard_nodes.push(*decl);
            }
            // then = break（无标签）
            let then_ok = twr_stmts(lang, ic[1]).len() == 1
                && matches!(lang.data(twr_stmts(lang, ic[1])[0]),
                    NodeData::Break { label: None });
            if !then_ok {
                return None;
            }
            // els = [v = K, switch(v)] 或 [switch(K)]
            let els = twr_stmts(lang, ic[2]);
            let (k, inner_switch, assign_node) = match els.len() {
                2 => {
                    let assign_expr = unwrap_expr_stmt(lang, els[0]);
                    let NodeData::Assign { op: None } = lang.data(assign_expr) else {
                        return None;
                    };
                    let a_ch = lang.children(assign_expr);
                    let vn = match lang.data(a_ch[0]) {
                        NodeData::VarRef { name } => lang.sn(*name).to_string(),
                        _ => return None,
                    };
                    let k = match lang.literal(a_ch[1])? {
                        LitRef::Int(v) | LitRef::Long(v) => v,
                        _ => return None,
                    };
                    if lang.kind(els[1]) != NodeKind::Switch {
                        return None;
                    }
                    // switch 选择器须为 v
                    let sel = *lang.children(els[1]).first()?;
                    if lang.kind(sel) != NodeKind::VarRef
                        || lang.var_name(sel) != Some(vn.as_str())
                    {
                        return None;
                    }
                    match &v_name {
                        Some(v) if *v != vn => return None,
                        _ => v_name = Some(vn),
                    }
                    (k, els[1], Some(els[0]))
                }
                1 => {
                    if lang.kind(els[0]) != NodeKind::Switch {
                        return None;
                    }
                    // 常量折叠后的形态：switch(K)
                    let sel = *lang.children(els[0]).first()?;
                    let k = match lang.literal(sel)? {
                        LitRef::Int(v) | LitRef::Long(v) => v,
                        _ => return None,
                    };
                    (k, els[0], None)
                }
                _ => return None,
            };
            if let Some(a) = assign_node {
                v_nodes.push(a);
            }
            v_nodes.push(*lang.children(inner_switch).first()?);
            // H ≡ hash(L)
            let hlit: i32 = lit
                .chars()
                .fold(0i32, |acc, ch| acc.wrapping_mul(31).wrapping_add(ch as i32));
            if hlit != h {
                return None;
            }
            let _ = assign_node;
            infos.push(CaseInfo {
                lit,
                k,
                inner_switch,
            });
        }
        if infos.is_empty() {
            return None;
        }
        // 父块 + 尾部
        if lang.kind(parent) != NodeKind::Block {
            return None;
        }
        let pch = lang.children(parent).to_vec();
        let mut k = idx + 1;
        // 可选 v = -1
        if k < pch.len() {
            let texpr = unwrap_expr_stmt(lang, pch[k]);
            if let NodeData::Assign { op: None } = lang.data(texpr) {
                let a_ch = lang.children(texpr);
                if a_ch.len() == 2
                    && lang.kind(a_ch[0]) == NodeKind::VarRef
                    && lang.var_name(a_ch[0]) == v_name.as_deref()
                    && lang.literal(a_ch[1]) == Some(LitRef::Int(-1))
                {
                    v_nodes.push(pch[k]);
                    k += 1;
                }
            }
        }
        if k >= pch.len() || lang.kind(pch[k]) != NodeKind::Switch {
            return None;
        }
        let trailing = pch[k];
        // 尾部选择器：VarRef v 或常量
        let tsel = *lang.children(trailing).first()?;
        match lang.data(tsel) {
            NodeData::VarRef { name } if Some(lang.sn(*name)) == v_name.as_deref() => {
                v_nodes.push(tsel);
            }
            NodeData::Literal(Lit::Int(_)) | NodeData::Literal(Lit::Long(_)) => {}
            _ => {
                if lang.literal(tsel).is_none() {
                    return None;
                }
            }
        }
        // 签名一致性：所有 inner switch + trailing 的**case 体**全同
        //（选择器除外——尾部的 v 常被赋值传播折叠成字面量）
        fn switch_cases_sig(lang: &JavaAst, sw: JavaId, out: &mut String) {
            for &c in &lang.children(sw)[1..] {
                subtree_sig(lang, c, out);
            }
        }
        let mut sig0 = String::new();
        switch_cases_sig(lang, infos[0].inner_switch, &mut sig0);
        for info in &infos[1..] {
            let mut sig = String::new();
            switch_cases_sig(lang, info.inner_switch, &mut sig);
            if sig != sig0 {
                return None;
            }
        }
        {
            let mut sig = String::new();
            switch_cases_sig(lang, trailing, &mut sig);
            if sig != sig0 {
                return None;
            }
        }
        // 提取 body（用第一个 inner switch）
        let bodies = ssw_bodies(lang, infos[0].inner_switch)?;
        let mut body_map: std::collections::HashMap<i64, Vec<JavaId>> =
            std::collections::HashMap::new();
        let mut default_body: Option<Vec<JavaId>> = None;
        for (lab, stmts) in &bodies {
            match lab {
                Some(v) => {
                    body_map.insert(*v, stmts.clone());
                }
                None => default_body = Some(stmts.clone()),
            }
        }
        let Some(default_body) = default_body else {
            return None;
        };
        // 终止性
        for stmts in body_map.values().chain(std::iter::once(&default_body)) {
            if !ssw_body_terminal(lang, stmts) {
                return None;
            }
        }
        // v/e 逃逸扫描（v 的声明本身合法——索引变量；其引用全在消费子树内）
        let root = ctx_root;
        if let Some(vn) = &v_name {
            let mut allowed: Vec<JavaId> = v_nodes.clone();
            allowed.push(trailing);
            if idx >= 1 {
                let cand = pch[idx - 1];
                if let NodeData::VarDecl { name, .. } = lang.data(cand) {
                    if name == vn {
                        allowed.push(cand);
                        // 声明位置紧邻（ddc 形态）；隔开的声明仍拒绝
                        if idx != 1 {
                            allowed.pop();
                        }
                    }
                }
            }
            if refs_escape(lang, root, vn, &allowed) {
                return None;
            }
        }
        // 构建 switch (x) { case "L": BODY_K; ... default: BODY_DEF }
        let mut new_cases: Vec<JavaId> = Vec::new();
        for info in &infos {
            let Some(body) = body_map.get(&info.k) else {
                return None;
            };
            let lab = lang.lit(Lit::Str(info.lit.clone()));
            let case = lang.case_(vec![lab], false, false, body.clone());
            new_cases.push(case);
        }
        let def_case = lang.case_(Vec::new(), true, false, default_body);
        new_cases.push(def_case);
        let sel = lang.var(&x);
        let new_switch = lang.switch_(sel, new_cases);
        // 消费 [hash switch, (v=-1)?, trailing switch]
        let remove = k - idx + 1;
        Some(Edit::Splice {
            node: parent,
            index: idx,
            remove,
            insert: vec![new_switch],
        })
    }
}

/// Call{x.equals("L")} → L
fn equals_lit(lang: &JavaAst, e: JavaId, x: &str) -> Option<String> {
    if lang.kind(e) != NodeKind::Call {
        return None;
    }
    let cc = lang.children(e);
    if cc.len() != 2 {
        return None;
    }
    match lang.data(cc[0]) {
        NodeData::Member { name } if lang.sn(*name) == "equals" => {
            let recv = *lang.children(cc[0]).first()?;
            if lang.kind(recv) != NodeKind::VarRef || lang.var_name(recv) != Some(x) {
                return None;
            }
            match lang.literal(cc[1])? {
                LitRef::Str(s) => Some(s.to_string()),
                _ => None,
            }
        }
        _ => None,
    }
}

fn is_x_equals(lang: &JavaAst, e: JavaId, x: &str) -> bool {
    lang.kind(e) == NodeKind::Call && equals_lit(lang, e, x).is_some()
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
    rules.push(Box::new(NewStringCharArrayFold));
    rules.push(Box::new(EmptyFinallyStrip));
    rules.push(Box::new(TryUnwrapNoCatch));
    rules.push(Box::new(TryUnwrapRethrow));
    rules.push(Box::new(StaticExec));
    rules.push(Box::new(LoopHeadBreak));
    rules.push(Box::new(WhileIteratorToForEach));
    rules.push(Box::new(ConcatValueOfDrop));
    rules.push(Box::new(StringBuilderStatements));
    rules.push(Box::new(TrailingContinueJava));
    rules.push(Box::new(XorNoise));
    rules.push(Box::new(StrLenFold));
    rules.push(Box::new(LiteralEval));
    rules.push(Box::new(Base64NewStringFold));
    rules.push(Box::new(UrlDecodeFold));
    rules.push(Box::new(CffRecover));
    rules.push(Box::new(StaticArrayIndexFold));
    rules.push(Box::new(ConstMethodInline));
    rules.push(Box::new(TwrRecover));
    rules.push(Box::new(StringSwitchRecover));
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
    // 收敛外循环：引擎单次调用跑至"本轮无应用"即停，但末轮被丢弃的结构
    // 提案（目标父在搬移后过期，需新鲜 walk 复活）会漏到下一次调用——
    // 外层重跑到不动点（上限防乒乓；真实语料 1-2 轮清零，javaparser 6 个
    // 幂等失败文件抓获：const_method_inline/if_else_empty 二轮仍有编辑）
    for _cycle in 0..6 {
        let mut round = Report::default();
        collect_unit_consts(ast, unit);
        for ty in &mut unit.types {
            simplify_type(ast, ty, cfg, &mut round);
        }
        if cfg.remove_dead_methods {
            round.edits += remove_dead_private_methods(ast, unit);
        }
        total.edits += round.edits;
        total.iterations += round.iterations;
        total.discarded += round.discarded;
        for (k, v) in round.by_rule {
            *total.by_rule.entry(k).or_insert(0) += v;
        }
        // 注：确认轮不可省——mega 实测驳回过"零丢弃即不动点"短路：跨轮
        // 编辑的主来源是 collect 表过期（内联/折叠后新常量/新单 return 方法
        // 涌现，重建表后才可见），非引擎丢弃提案的复活。RSS +2-3MB 是
        // 收敛正确性的价格。
        if round.edits == 0 {
            break;
        }
    }
    total
}

// ---------------------------------------------------------------------------
// 死私有方法删除（opt-in，Config::remove_dead_methods）：
// const_method_inline / 解密器还原后残留的 helper 清理。
// 安全域：
//   - 仅 private（private 不可能被外部调用或被子类覆盖）；
//   - 全单元零引用：Call 的 callee 名（Member 名/裸名）与 MethodRef 名，
//     名字匹配不判签名——匹配只会导致"保留"，方向保守；
//   - 名字与本单元其他声明（方法重载/字段）撞名 → 保留（混淆器单字母
//     名高频撞名，保守不删）；
//   - 修饰符串含 @（注解方法，可能是框架入口）→ 保留；
//   - 构造器不删（单例模式 private ctor 是活的）。
// 反射调用无法静态排除 → opt-in 而非默认。
// 传递性死代码：迭代到不动点（删一层后重收集引用）。
// ---------------------------------------------------------------------------

fn remove_dead_private_methods(ast: &JavaAst, unit: &mut CompilationUnit) -> usize {
    use std::collections::{HashMap, HashSet};
    let mut removed = 0usize;
    loop {
        // 1) 全单元引用名（不可变借用阶段）
        let mut referenced: HashSet<String> = HashSet::new();
        let mut roots: Vec<JavaId> = Vec::new();
        for_each_type(unit, &mut |ty| collect_member_roots(ty, &mut roots));
        for body in roots {
            collect_call_names(ast, body, &mut referenced);
        }
        // 2) 声明名字统计（重载/字段撞名判定）
        let mut decl_counts: HashMap<String, usize> = HashMap::new();
        for_each_type(unit, &mut |ty| {
            for m in &ty.members {
                match m {
                    Member::Method { name, .. }
                    | Member::Constructor { name, .. } => {
                        *decl_counts.entry(name.clone()).or_insert(0) += 1;
                    }
                    Member::Field { declarators, .. } => {
                        for d in declarators {
                            *decl_counts.entry(d.name.clone()).or_insert(0) += 1;
                        }
                    }
                    _ => {}
                }
            }
        });
        // 3) 删除（可变借用阶段）
        let mut changed = false;
        for_each_type_mut(unit, &mut |ty: &mut TypeDecl| {
            let before = ty.members.len();
            ty.members.retain(|m| {
                if let Member::Method { mods, name, .. } = m {
                    let is_private = mods.split_whitespace().any(|w| w == "private");
                    let annotated = mods.contains('@');
                    let keep = !is_private
                        || annotated
                        || referenced.contains(name)
                        // 撞名（同名重载/字段）：名字引用无法区分指向 → 保留
                        || decl_counts.get(name).copied().unwrap_or(0) > 1;
                    keep
                } else {
                    true
                }
            });
            if ty.members.len() != before {
                removed += before - ty.members.len();
                changed = true;
            }
        });
        if !changed {
            return removed;
        }
    }
}

/// 递归访问（含嵌套类型），只读。
fn for_each_type(unit: &CompilationUnit, f: &mut dyn FnMut(&TypeDecl)) {
    for ty in &unit.types {
        for_each_type_inner(ty, f);
    }
}
fn for_each_type_inner(ty: &TypeDecl, f: &mut dyn FnMut(&TypeDecl)) {
    f(ty);
    for m in &ty.members {
        if let Member::Type(t) = m {
            for_each_type_inner(t, f);
        }
    }
}

/// 递归访问（含嵌套类型），可变。
fn for_each_type_mut(unit: &mut CompilationUnit, f: &mut dyn FnMut(&mut TypeDecl)) {
    for ty in &mut unit.types {
        for_each_type_mut_inner(ty, f);
    }
}
fn for_each_type_mut_inner(ty: &mut TypeDecl, f: &mut dyn FnMut(&mut TypeDecl)) {
    f(ty);
    for m in &mut ty.members {
        if let Member::Type(t) = m {
            for_each_type_mut_inner(t, f);
        }
    }
}

/// 汇集一个类型所有可走子树的根（方法体/初始化块/字段初始化器）。
fn collect_member_roots(ty: &TypeDecl, out: &mut Vec<JavaId>) {
    for m in &ty.members {
        match m {
            Member::Method { body: Some(b), .. }
            | Member::Constructor { body: Some(b), .. } => out.push(*b),
            Member::Initializer { body, .. } => out.push(*body),
            Member::Field { declarators, .. } => {
                for d in declarators {
                    if let Some(init) = d.init {
                        out.push(init);
                    }
                }
            }
            Member::Type(t) => collect_member_roots(t, out),
            _ => {}
        }
    }
}

/// 子树内收集所有"被调用"的名字：Call 的 callee（Member 名 / 裸 VarRef 名）
/// 与 MethodRef 名（`recv::name`）。名字匹配，不判签名。
fn collect_call_names(ast: &JavaAst, root: JavaId, out: &mut std::collections::HashSet<String>) {
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        match ast.data(id) {
            NodeData::Call => {
                if let Some(&callee) = ast.children(id).first() {
                    match ast.data(callee) {
                        NodeData::Member { name } => {
                            out.insert(ast.sn(*name).to_string());
                        }
                        NodeData::VarRef { name } => {
                            out.insert(ast.sn(*name).to_string());
                        }
                        _ => {}
                    }
                }
            }
            NodeData::MethodRef { name } => {
                // `recv::name` / `recv::new`
                if let Some(short) = ast.sn(*name).rsplit("::").next() {
                    out.insert(short.to_string());
                }
            }
            _ => {}
        }
        for &c in ast.children(id) {
            stack.push(c);
        }
    }
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
                    // 根折叠：字段 init 节点就是 walk 根——Replace 无父
                    // 槽可写（引擎既有边界），由本侧拿到新根回写声明
                    //（static int x=1+2 曾永不折——本轮修复）
                    let rules = default_java_rules();
                    let mut cur: JavaId = init;
                    for _ in 0..8 {
                        match cure_engine::fold_root(ast, cur, &rules, cfg) {
                            Some((new_root, name)) => {
                                d.init = Some(new_root);
                                cur = new_root;
                                *total.by_rule.entry(name).or_insert(0) += 1;
                                total.edits += 1;
                                // 新根内部可能再折
                                let r = simplify(ast, new_root, cfg);
                                merge_report(total, r);
                            }
                            None => break,
                        }
                    }
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
