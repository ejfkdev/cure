//! TS/JS 专属规则包——与 Java 规则集对等的能力面：
//!
//! - [`TsLiteralEval`]：字面量上的纯调用求值（Math.* / String 方法 /
//!   `String.fromCharCode` / `parseInt`/`parseFloat`/`Number` / `atob` /
//!   `decodeURIComponent` / `.length` / `[a,b].join(...)` / `String(x)`）
//!   ——混淆器字符串编码层的核心对抗（Java 的 literal_eval /
//!   base64_new_string_fold / url_decode_fold / str_len_fold 合集）
//! - [`TsTryCleanup`]：空 finally 剥离 / `catch(e){throw e}` 解包 /
//!   无 catch 的 try 解包（Java empty_finally_strip / try_unwrap_*）
//! - [`TsTrailingElseFlatten`]：then 终结后的 else 展平
//! - [`TsTrailingAssignReturn`]：`x = e; return x` → `return e`（局部）
//! - [`TsDeadPureCall`]：丢弃结果的纯调用语句删除（Math.* 等白名单）
//!
//! JS 语义守卫（与 Java 版的差异点）：
//! - 数值结果 NaN/±Infinity **不折**（无法以字面量表示——NaN 是全局名）
//! - -0 结果可折（打印器输出 "-0"，1/-0 语义保持）
//! - 字符串结果含**孤立代理项**不折（Rust String 无法表示 UTF-16 单代理）
//! - `String.fromCharCode` 按 UTF-16 单元组合（代理对合并）
//! - `toLowerCase`/`toUpperCase` 使用 Rust 默认 Unicode 映射（与 JS 的
//!   非 locale 版本一致；locale 变体不在折叠域）
//! - `x + y` 的字符串胁迫不在本层（引擎 fold_lit_bin 已按 JS 域处理）

use std::collections::HashMap;

use cure_engine::kind::{LitRef, NodeKind};
use cure_engine::{Config, Edit, Effect, Lang, Report, RewriteCtx, Rule};
use cure_ts_ast::{format_js_number, JsLit, NodeData, TsAst, TsId};

// ---------------------------------------------------------------------------
// 字面量求值
// ---------------------------------------------------------------------------

/// 字面量上的纯调用求值。
pub struct TsLiteralEval;

impl Rule<TsAst> for TsLiteralEval {
    fn name(&self) -> &'static str {
        "ts_literal_eval"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Call, NodeKind::Member, NodeKind::Index, NodeKind::New]
    }
    fn check(&self, ctx: RewriteCtx<'_, TsAst>, id: TsId) -> Option<Edit<TsAst>> {
        // 写目标/删除位：`.length` 等折叠成字面量后 `[].length = x` 会变
        // `0 = x`（非法）、`delete a[0]` 变 `delete lit`（返回值/副作用
        // 改变）——一律不折。
        if let Some(p) = ctx.parent(id) {
            let k = ctx.lang.kind(p);
            if k == NodeKind::Assign && ctx.lang.children(p).first() == Some(&id) {
                return None;
            }
            if k == NodeKind::Unary {
                if let Some(op) = ctx.lang.un_op(p) {
                    if matches!(
                        op,
                        cure_engine::kind::UnOp::Delete
                            | cure_engine::kind::UnOp::PreInc
                            | cure_engine::kind::UnOp::PreDec
                            | cure_engine::kind::UnOp::PostInc
                            | cure_engine::kind::UnOp::PostDec
                    ) {
                        return None;
                    }
                }
            }
        }
        let RewriteCtx { lang, .. } = ctx;
        match lang.kind(id) {
            NodeKind::Call => eval_call(lang, id),
            NodeKind::Member => eval_member(lang, id),
            // arr[i] 常量字面量下标（[5,6][0]——非 const 名形态）
            NodeKind::Index => eval_index(lang, id),
            NodeKind::New => eval_new(lang, id),
            _ => None,
        }
    }
}

/// `new String("a")`：**不可折**为字面量（对象同一性/类型改变——
/// `new String("a") === "a"` 为 false，折了变 true）。`new Number(5)`
/// 同理。本规则只处理 `new String(lit)` 的 `.toString()` 链？v1 保守
/// 跳过全部 new。
fn eval_new(_lang: &TsAst, _id: TsId) -> Option<Edit<TsAst>> {
    None
}

/// `"abc".length` / `[1,2].length` / `"a,b".split(",").length`？
/// 仅前两者（split 返回数组的 length 需要数组模型——见 eval_call 的
/// join 先折再 length 连锁）。
fn eval_member(lang: &mut TsAst, id: TsId) -> Option<Edit<TsAst>> {
    let ch = lang.children(id).to_vec();
    let base = *ch.first()?;
    let name = lang.member_name(id)?;
    if name != "length" {
        return None;
    }
    match lang.data(base) {
        NodeData::Literal(JsLit::Str(s)) => {
            let n = s.encode_utf16().count();
            if n > i32::MAX as usize {
                return None;
            }
            Some(Edit::Replace {
                target: id,
                with: lang.build_int(n as i64, false),
            })
        }
        NodeData::ArrayLit => {
            // 元素数（稀疏洞在 AST 里是 raw("") 占位——计数仍准确？
            // parser 对每个 elision 推一个 raw 占位 ✓）
            let n = lang.children(base).len();
            if n > i32::MAX as usize {
                return None;
            }
            Some(Edit::Replace {
                target: id,
                with: lang.build_int(n as i64, false),
            })
        }
        _ => None,
    }
}

/// `[5,6][0]` 纯字面量下标。
fn eval_index(lang: &mut TsAst, id: TsId) -> Option<Edit<TsAst>> {
    let ch = lang.children(id).to_vec();
    let (base, idx) = (ch[0], ch[1]);
    if !matches!(lang.data(id), NodeData::Index { optional: false }) {
        return None;
    }
    if lang.kind(base) != NodeKind::ArrayLit {
        return None;
    }
    let elems = lang.children(base).to_vec();
    if !elems.iter().all(|&e| lang.kind(e) == NodeKind::Literal) {
        return None;
    }
    let i = match lang.literal(idx) {
        Some(LitRef::Int(v)) if v >= 0 => usize::try_from(v).ok()?,
        _ => return None,
    };
    let &elem = elems.get(i)?;
    Some(Edit::Replace { target: id, with: elem })
}

/// 调用求值：callee 形态分派。
fn eval_call(lang: &mut TsAst, id: TsId) -> Option<Edit<TsAst>> {
    let ch = lang.children(id).to_vec();
    let callee = *ch.first()?;
    let args = &ch[1..];
    // optional call 不折（链短路语义）
    if matches!(lang.data(id), NodeData::Call { optional: true }) {
        return None;
    }
    match lang.data(callee) {
        // Math.xxx(...) / Number.xxx / String.xxx / Object.xxx
        NodeData::Member { .. } => {
            let base = lang.children(callee).first().copied()?;
            let member = lang.member_name(callee)?.to_string();
            // VarRef 基座 → 静态白名单（Math/Number/String/Object/Array）
            if let NodeData::VarRef { name } = lang.data(base) {
                let base_name = lang.sn(*name).to_string();
                return match base_name.as_str() {
                    "Math" => eval_math(lang, &member, args),
                    "Number" => eval_number_static(lang, &member, args),
                    "String" => eval_string_static(lang, &member, args),
                    "Object" => eval_object_static(lang, &member, args),
                    "Array" => eval_array_static(lang, &member, args),
                    _ => None,
                }
                .map(|with| Edit::Replace { target: id, with });
            }
            // 其余基座（字面量/数组字面量）→ 方法路径
            eval_method_on_literal(lang, callee, args)
                .map(|with| Edit::Replace { target: id, with })
        }
        // 裸函数：parseInt / parseFloat / Number / String / Boolean /
        // isNaN?（返回布尔 ✓ 但依赖全局——`isNaN` 可被遮蔽！白名单
        // 全局函数仅在名字未被本模块声明遮蔽时安全——保守：仅当
        // 名字不出现在任何声明中……v1 直接检查 var_name 不在 const/
        // shadow 集不可得 → 使用合成节点判定：仅当 callee 是**未被
        // 任何 VarDecl 声明的名字**。此处简化：仅折 parseInt/
        // parseFloat/Number/String/atob/decodeURIComponent/
        // decodeURI（全局不可赋值性由收集期 shadow 检测覆盖？
        // 不覆盖——保守接受：这些是 JS 内建全局，遮蔽极罕见且
        // 用户模块内同名函数会被 locals 解析……为安全起见要求
        // 模块内无同名声明由调用方保证不了——接受内建语义）
        NodeData::VarRef { name } => {
            let fname = lang.sn(*name).to_string();
            match fname.as_str() {
                "parseInt" => eval_parseint(lang, args),
                "parseFloat" => eval_parsefloat(lang, args),
                "Number" => eval_number_fn(lang, args),
                "String" => eval_string_fn(lang, args),
                "atob" => eval_atob(lang, args),
                // decodeURI 保留保留字符转义（"%3B" 不变成 ";"）——
                // 语义面太大，只折 decodeURIComponent。
                "decodeURIComponent" => eval_decode_uri(lang, args),
                _ => None,
            }
            .map(|with| Edit::Replace { target: id, with })
        }
        // 字面量接收者的方法：`"abc".slice(1)` / `[1,2].join("")`
        _ => eval_method_on_literal(lang, callee, args).map(|with| Edit::Replace { target: id, with }),
    }
}

/// 字符串/数组字面量接收者的方法调用。
fn eval_method_on_literal(lang: &mut TsAst, callee: TsId, args: &[TsId]) -> Option<TsId> {
    // callee = Member{name}(base)
    let NodeData::Member { .. } = lang.data(callee) else {
        return None;
    };
    let base = lang.children(callee).first().copied()?;
    let m = lang.member_name(callee)?.to_string();
    // 实参须全为可求值字面量（纯）
    for &a in args {
        if lang.kind(a) != NodeKind::Literal || lang.effect(a) != Effect::Pure {
            return None;
        }
    }
    match lang.data(base) {
        NodeData::Literal(JsLit::Str(s)) => {
            let s = s.clone();
            let arg_vals: Vec<ArgVal> = args.iter().map(|&a| arg_val(lang, a)).collect::<Option<_>>()?;
            str_method(&s, &m, &arg_vals).and_then(|v| lit_of(lang, v))
        }
        NodeData::ArrayLit => {
            if m == "join" {
                let sep = if args.is_empty() {
                    ",".to_string()
                } else {
                    match lang.literal(args[0]) {
                        Some(LitRef::Str(s)) => s.to_string(),
                        _ => return None,
                    }
                };
                let elems = lang.children(base).to_vec();
                let mut parts: Vec<String> = Vec::new();
                for &e in &elems {
                    match lang.literal(e) {
                        Some(LitRef::Str(s)) => parts.push(s.to_string()),
                        Some(LitRef::Int(v)) => parts.push(v.to_string()),
                        Some(LitRef::Double(v)) => {
                            let t = format_js_number(v);
                            if t == "NaN" || t.contains("Infinity") {
                                return None;
                            }
                            parts.push(t)
                        }
                        Some(LitRef::Bool(b)) => parts.push(if b { "true" } else { "false" }.into()),
                        Some(LitRef::Null) => parts.push(String::new()),
                        _ => return None,
                    }
                }
                Some(lang.build_str(&parts.join(&sep)))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// 实参值（str/int/num/bool/null）。
#[derive(Clone, Debug)]
enum ArgVal {
    Str(String),
    Int(i64),
    Num(f64),
}

fn arg_val(lang: &mut TsAst, a: TsId) -> Option<ArgVal> {
    match lang.literal(a)? {
        LitRef::Str(s) => Some(ArgVal::Str(s.to_string())),
        LitRef::Int(v) => Some(ArgVal::Int(v)),
        LitRef::Double(v) => Some(ArgVal::Num(v)),
        LitRef::Bool(b) => Some(ArgVal::Int(if b { 1 } else { 0 })),
        LitRef::Null => Some(ArgVal::Int(0)),
        _ => None,
    }
}

/// 折叠结果值。
enum FoldVal {
    Str(String),
    Int(i64),
    Num(f64),
    Bool(bool),
}

/// 值 → 字面量节点（NaN/∞/孤立代理 → None）。
fn lit_of(lang: &mut TsAst, v: FoldVal) -> Option<TsId> {
    match v {
        FoldVal::Str(s) => {
            // 孤立代理检查（Rust String 表示不了 UTF-16 单代元——但从
            // Rust String 构造的串天然由码位组成；slice 类操作按 UTF-16
            // 单元切片可能切开代理对 → 结果构造成 String 时已由
            // char::from_u32 过滤）
            Some(lang.build_str(&s))
        }
        FoldVal::Int(i) => Some(lang.build_int(i, false)),
        FoldVal::Bool(b) => Some(lang.build_bool(b)),
        FoldVal::Num(f) => {
            if f.is_nan() || f.is_infinite() {
                return None; // 无法字面量化（NaN 是全局名）
            }
            Some(lang.clone_node_of_double(f))
        }
    }
}

// ---------------------------------------------------------------------------
// Math.*
// ---------------------------------------------------------------------------

fn eval_math(lang: &mut TsAst, m: &str, args: &[TsId]) -> Option<TsId> {
    let nums: Vec<f64> = args.iter().map(|&a| num_arg(lang, a)).collect::<Option<_>>()?;
    let r: f64 = match m {
        "abs" if nums.len() == 1 => nums[0].abs(),
        "ceil" if nums.len() == 1 => nums[0].ceil(),
        "floor" if nums.len() == 1 => nums[0].floor(),
        "round" if nums.len() == 1 => js_round(nums[0]),
        "trunc" if nums.len() == 1 => nums[0].trunc(),
        "sign" if nums.len() == 1 => {
            if nums[0].is_nan() {
                return None;
            }
            if nums[0] > 0.0 {
                1.0
            } else if nums[0] < 0.0 {
                -1.0
            } else {
                nums[0] // ±0 保持符号
            }
        }
        "sqrt" if nums.len() == 1 => {
            if nums[0] < 0.0 {
                return None; // NaN
            }
            nums[0].sqrt()
        }
        "cbrt" if nums.len() == 1 => nums[0].cbrt(),
        "exp" if nums.len() == 1 => nums[0].exp(),
        "lon" | "log" if nums.len() == 1 => {
            if nums[0] < 0.0 {
                return None;
            }
            nums[0].ln()
        }
        "log2" if nums.len() == 1 => {
            if nums[0] < 0.0 {
                return None;
            }
            nums[0].log2()
        }
        "log10" if nums.len() == 1 => {
            if nums[0] < 0.0 {
                return None;
            }
            nums[0].log10()
        }
        "sin" if nums.len() == 1 => nums[0].sin(),
        "cos" if nums.len() == 1 => nums[0].cos(),
        "tan" if nums.len() == 1 => nums[0].tan(),
        "atan" if nums.len() == 1 => nums[0].atan(),
        "pow" if nums.len() == 2 => nums[0].powf(nums[1]),
        "min" if !nums.is_empty() => {
            let mut acc = f64::INFINITY;
            for &v in &nums {
                if v.is_nan() {
                    return None;
                }
                acc = acc.min(v);
            }
            // Math.min(0, -0) === -0（min 保持负零）
            if nums.iter().any(|v| *v == 0.0 && v.is_sign_negative()) && acc == 0.0 {
                -0.0
            } else {
                acc
            }
        }
        "max" if !nums.is_empty() => {
            let mut acc = f64::NEG_INFINITY;
            for &v in &nums {
                if v.is_nan() {
                    return None;
                }
                acc = acc.max(v);
            }
            if nums.iter().any(|v| v.is_sign_positive()) && acc == 0.0 {
                0.0
            } else {
                acc
            }
        }
        "hypot" if !nums.is_empty() => nums.iter().fold(0.0, |a, v| a.hypot(*v)),
        _ => return None,
    };
    if r.is_nan() || r.is_infinite() {
        return None;
    }
    // 整数结果 → Int 字面量（打印形态一致）
    if r.fract() == 0.0 && r.abs() <= (1u64 << 53) as f64 && !(r == 0.0 && r.is_sign_negative()) {
        Some(lang.build_int(r as i64, false))
    } else {
        Some(lang.clone_node_of_double(r))
    }
}

/// 数值字面量 AST（含一元 +/- 包裹）。
fn is_numeric_ast(lang: &TsAst, a: TsId) -> bool {
    let is_num = |n: TsId| {
        matches!(
            lang.data(n),
            NodeData::Literal(JsLit::Int(_) | JsLit::Double(_) | JsLit::NumRaw { .. })
        )
    };
    match lang.data(a) {
        NodeData::Literal(JsLit::Int(_) | JsLit::Double(_) | JsLit::NumRaw { .. }) => true,
        NodeData::Unary { op: cure_engine::kind::UnOp::Neg | cure_engine::kind::UnOp::Plus } => {
            lang.children(a).first().is_some_and(|&c| is_num(c))
        }
        _ => false,
    }
}

fn num_arg(lang: &mut TsAst, a: TsId) -> Option<f64> {
    // 一元 ± 字面量（-0.4 / -3 / +1）→ 取值
    if let NodeData::Unary { op } = lang.data(a) {
        if matches!(op, cure_engine::kind::UnOp::Neg | cure_engine::kind::UnOp::Plus) {
            let neg = *op == cure_engine::kind::UnOp::Neg;
            let inner = *lang.children(a).first()?;
            return match lang.literal(inner)? {
                LitRef::Int(v) => Some(if neg { -(v as f64) } else { v as f64 }),
                LitRef::Double(v) => Some(if neg { -v } else { v }),
                _ => None,
            };
        }
    }
    match lang.literal(a)? {
        LitRef::Int(v) => Some(v as f64),
        LitRef::Double(v) => Some(v),
        LitRef::Bool(b) => Some(if b { 1.0 } else { 0.0 }),
        LitRef::Null => Some(0.0),
        _ => None,
    }
}

/// JS Math.round：.5 向 +∞（-0.5 → -0）。
fn js_round(x: f64) -> f64 {
    if x.is_nan() || x.is_infinite() {
        return x;
    }
    let f = x.floor();
    let r = if x - f >= 0.5 { f + 1.0 } else { f };
    // round(-0.4) → -0（符号保持）；round(+0) 必须保持 +0
    //（`x == -0.0` 对 +0 也成立——曾把 Math.round(0) 折成 -0，
    //  Test262 Math/round 抓获）
    if r == 0.0 && x.is_sign_negative() {
        return -0.0;
    }
    r
}

// ---------------------------------------------------------------------------
// String.* / Number.* / Object.* / Array.* 静态方法
// ---------------------------------------------------------------------------

fn eval_string_static(lang: &mut TsAst, m: &str, args: &[TsId]) -> Option<TsId> {
    match m {
        "fromCharCode" => {
            let mut units: Vec<u16> = Vec::with_capacity(args.len());
            for &a in args {
                let v = match lang.literal(a)? {
                    LitRef::Int(v) => v,
                    LitRef::Double(v) if v.fract() == 0.0 => v as i64,
                    _ => return None,
                };
                units.push((v & 0xFFFF) as u16);
            }
            let s = String::from_utf16(&units).ok()?; // 孤立代理 → None
            Some(lang.build_str(&s))
        }
        "fromCodePoint" => {
            let mut s = String::new();
            for &a in args {
                let v = match lang.literal(a)? {
                    LitRef::Int(v) if (0..=0x10FFFF).contains(&v) => v as u32,
                    _ => return None,
                };
                s.push(char::from_u32(v)?);
            }
            Some(lang.build_str(&s))
        }
        "raw" => None,
        _ => None,
    }
}

fn eval_number_static(lang: &mut TsAst, m: &str, args: &[TsId]) -> Option<TsId> {
    match m {
        "isInteger" | "isSafeInteger" | "isFinite" | "isNaN" => {
            let a = *args.first()?;
            // 这些判定**不做类型转换**：非 Number 字面量（bool/null/
            // string/对象字面量）一律 false；**非字面量**（变量/调用，
            // 如 NaN、参数 arg）不可折（曾把 isNaN(arg) 折成 false 导致
            // 三元走错分支、把 isFinite(-10) 折成 false——
            // Test262 annexB substr / Number 家族抓获）。
            if !is_numeric_ast(lang, a) {
                if matches!(lang.data(a), NodeData::Literal(_)) {
                    return Some(lang.build_bool(false));
                }
                return None;
            }
            let v = num_arg(lang, a)?;
            let b = match m {
                "isInteger" => v.fract() == 0.0 && v.is_finite(),
                // 安全整数是 |v| ≤ 2^53 − 1（2^53 本身不安全——
                // Test262 not-safe-integer 抓获）
                "isSafeInteger" => v.fract() == 0.0 && v.abs() < (1u64 << 53) as f64,
                "isFinite" => v.is_finite(),
                _ => v.is_nan(),
            };
            Some(lang.build_bool(b))
        }
        "parseInt" => eval_parseint(lang, args),
        "parseFloat" => eval_parsefloat(lang, args),
        _ => None,
    }
}

fn eval_object_static(_lang: &TsAst, _m: &str, _args: &[TsId]) -> Option<TsId> {
    // Object.keys/entries 返回数组——需数组模型，v1 跳过
    None
}

fn eval_array_static(lang: &mut TsAst, m: &str, args: &[TsId]) -> Option<TsId> {
    match m {
        "isArray" => {
            let a = *args.first()?;
            if matches!(lang.data(a), NodeData::ArrayLit) {
                Some(lang.build_bool(true))
            } else if lang.kind(a) == NodeKind::Literal {
                Some(lang.build_bool(false))
            } else {
                None
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// 全局函数
// ---------------------------------------------------------------------------

fn eval_parseint(lang: &mut TsAst, args: &[TsId]) -> Option<TsId> {
    let s = match lang.literal(*args.first()?)? {
        LitRef::Str(s) => s.to_string(),
        LitRef::Int(v) => {
            // parseInt(数字) → ToString 后解析
            format_js_number(v as f64)
        }
        LitRef::Double(v) => {
            let t = format_js_number(v);
            if t.contains("Infinity") || t == "NaN" {
                return None;
            }
            t
        }
        _ => return None,
    };
    let radix = if args.len() >= 2 {
        match lang.literal(args[1])? {
            LitRef::Int(r) if (2..=36).contains(&r) => Some(r as u32),
            LitRef::Double(r) if (2.0..=36.0).contains(&r) => Some(r as u32),
            _ => return None,
        }
    } else {
        None
    };
    let v = js_parse_int(&s, radix)?;
    if v.unsigned_abs() > (1u64 << 53) {
        return None;
    }
    // parseInt("-0") = -0（Int 字面量只有 +0；Test262 sm/parseInt 抓获）
    if v == 0 && s.trim_start().starts_with('-') {
        return Some(lang.clone_node_of_double(-0.0));
    }
    Some(lang.build_int(v, false))
}

fn eval_parsefloat(lang: &mut TsAst, args: &[TsId]) -> Option<TsId> {
    let s = match lang.literal(*args.first()?)? {
        LitRef::Str(s) => s.to_string(),
        LitRef::Int(v) => format_js_number(v as f64),
        LitRef::Double(v) => {
            let t = format_js_number(v);
            if t.contains("Infinity") || t == "NaN" {
                return None;
            }
            t
        }
        _ => return None,
    };
    let v = js_parse_float(&s)?;
    if v.is_nan() || v.is_infinite() {
        return None;
    }
    Some(lang.clone_node_of_double(v))
}

fn eval_number_fn(lang: &mut TsAst, args: &[TsId]) -> Option<TsId> {
    let a = *args.first()?;
    let v = match lang.literal(a)? {
        LitRef::Int(v) => v as f64,
        LitRef::Double(v) => v,
        LitRef::Str(s) => js_to_number(s)?,
        LitRef::Bool(b) => {
            if b {
                1.0
            } else {
                0.0
            }
        }
        LitRef::Null => 0.0,
        _ => return None,
    };
    if v.is_nan() || v.is_infinite() {
        return None;
    }
    // -0 必须保留符号：Int 字面量只有 +0（Test262 Number('-0') 抓获）
    if v == 0.0 && v.is_sign_negative() {
        return Some(lang.clone_node_of_double(-0.0));
    }
    if v.fract() == 0.0 && v.abs() <= (1u64 << 53) as f64 {
        Some(lang.build_int(v as i64, false))
    } else {
        Some(lang.clone_node_of_double(v))
    }
}

fn eval_string_fn(lang: &mut TsAst, args: &[TsId]) -> Option<TsId> {
    let a = *args.first()?;
    let s = js_to_string_lit(lang, a)?;
    Some(lang.build_str(&s))
}

fn eval_atob(lang: &mut TsAst, args: &[TsId]) -> Option<TsId> {
    let s = match lang.literal(*args.first()?)? {
        LitRef::Str(s) => s.to_string(),
        _ => return None,
    };
    let bytes = base64_decode(&s)?;
    // atob 语义：字节序列按 Latin-1 单元解成字符串
    let out: String = bytes.iter().map(|&b| b as char).collect();
    Some(lang.build_str(&out))
}

fn eval_decode_uri(lang: &mut TsAst, args: &[TsId]) -> Option<TsId> {
    let s = match lang.literal(*args.first()?)? {
        LitRef::Str(s) => s.to_string(),
        _ => return None,
    };
    let bytes = percent_decode_utf8(&s)?;
    let out = String::from_utf8(bytes).ok()?;
    Some(lang.build_str(&out))
}

// ---------------------------------------------------------------------------
// JS 语义辅助函数（纯 Rust 实现）
// ---------------------------------------------------------------------------

/// JS ToString（字面量域）。对象（数组/对象字面量）返回其 JS 字符串
/// 形态仅对数组有意义（join）——此处保守限标量 + 数组。
fn js_to_string_lit(lang: &mut TsAst, n: TsId) -> Option<String> {
    match lang.data(n) {
        NodeData::Literal(JsLit::Str(s)) => Some(s.clone()),
        NodeData::Literal(JsLit::Int(v)) => Some(v.to_string()),
        NodeData::Literal(JsLit::Double(v)) => {
            let t = format_js_number(*v);
            if t.contains("Infinity") || t == "NaN" {
                None
            } else {
                Some(t)
            }
        }
        NodeData::Literal(JsLit::Bool(b)) => Some(if *b { "true" } else { "false" }.into()),
        NodeData::Literal(JsLit::Null) => Some("null".into()),
        _ => None,
    }
}

/// JS ToNumber（字符串域，ECMA 保守子集：十进制/十六进制/空白/空串）。
fn js_to_number(s: &str) -> Option<f64> {
    let t = s.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
    if t.is_empty() {
        return Some(0.0);
    }
    let (sign, rest, signed) = match t.strip_prefix('-') {
        Some(r) => (-1.0, r, true),
        None => match t.strip_prefix('+') {
            Some(r) => (1.0, r, true),
            None => (1.0, t, false),
        },
    };
    if rest.eq_ignore_ascii_case("infinity") {
        return None; // 非有限
    }
    // Number('+0x10') / Number('-0x10') 是 NaN（十六进制串不允许符号）
    if signed && (rest.starts_with("0x") || rest.starts_with("0X")) {
        return None;
    }
    if let Some(h) = rest.strip_prefix("0x").or_else(|| rest.strip_prefix("0X")) {
        if h.chars().all(|c| c.is_ascii_hexdigit()) && !h.is_empty() {
            return u64::from_str_radix(h, 16).ok().map(|v| sign * v as f64);
        }
        return None;
    }
    if rest.chars().all(|c| c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-') {
        if let Ok(v) = rest.parse::<f64>() {
            return Some(sign * v);
        }
    }
    None
}

/// ToIntegerOrInfinity 截断后的位置参数（缺省 → `default`；NaN → 0；
/// ±Infinity → 0/len；负 → 0；超界 → len）。字符串位置以 UTF-16 单元计。
fn pos_arg(a: Option<&ArgVal>, len: usize, default: usize) -> Option<usize> {
    let Some(a) = a else { return Some(default) };
    let v = match a {
        ArgVal::Int(i) => *i as f64,
        ArgVal::Num(n) => *n,
        ArgVal::Str(_) => return None, // 字符串位置（ToNumber 可折，保守跳过）
    };
    if v.is_nan() {
        return Some(0);
    }
    if v >= len as f64 {
        return Some(len);
    }
    if v <= 0.0 {
        return Some(0);
    }
    Some(v.trunc() as usize)
}

/// JS parseInt（允许前缀合法性，忽略尾部垃圾；radix 默认 10/0x→16）。
fn js_parse_int(s: &str, radix: Option<u32>) -> Option<i64> {
    let t = s.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
    let (sign, rest) = match t.strip_prefix('-') {
        Some(r) => (-1i64, r),
        None => (1i64, t.strip_prefix('+').unwrap_or(t)),
    };
    let (rest, radix) = match radix {
        // 显式基数 16 同样剥 0x 前缀（parseInt("0x1", 16) === 1；
        // Test262 parseInt A7.2_T3 抓获）
        Some(16) => (
            rest.strip_prefix("0x")
                .or_else(|| rest.strip_prefix("0X"))
                .unwrap_or(rest),
            16u32,
        ),
        Some(r) => (rest, r),
        None => {
            if let Some(h) = rest.strip_prefix("0x").or_else(|| rest.strip_prefix("0X")) {
                (h, 16u32)
            } else {
                (rest, 10u32)
            }
        }
    };
    let mut acc: i64 = 0;
    let mut any = false;
    for c in rest.chars() {
        let d = match c.to_digit(radix) {
            Some(d) => d as i64,
            None => break,
        };
        acc = acc.checked_mul(radix as i64)?.checked_add(d)?;
        any = true;
    }
    if !any {
        return None;
    }
    Some(sign * acc)
}

/// JS parseFloat（最长合法前缀；无合法数字 → NaN）。
fn js_parse_float(s: &str) -> Option<f64> {
    let t = s.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
    // 含 Infinity
    let lower = t.to_ascii_lowercase();
    for pfx in ["infinity", "+infinity", "-infinity"] {
        if lower.starts_with(pfx) {
            return None; // 非有限——不折
        }
    }
    // 最长数字前缀
    let b = t.as_bytes();
    let mut end = 0usize;
    let mut i = 0usize;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let mut digits = false;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
        digits = true;
        end = i;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
            digits = true;
            end = i;
        }
    }
    if digits && i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        let mut exp_digits = false;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
            exp_digits = true;
        }
        if exp_digits {
            end = j;
        }
    }
    if !digits {
        return None;
    }
    let text = &t[..end];
    if text.chars().all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-')) {
        text.parse::<f64>().ok()
    } else {
        None
    }
}

/// Base64 解码（标准字母表；空白忽略；缺 padding 补齐；非法 → None）。
fn base64_decode(s: &str) -> Option<Vec<u8>> {
    const TBL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lut = [255u8; 256];
    for (i, &c) in TBL.iter().enumerate() {
        lut[c as usize] = i as u8;
    }
    let clean: Vec<u8> = s
        .bytes()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    let mut out = Vec::with_capacity(clean.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0u32;
    let mut pad = 0usize;
    for (i, &c) in clean.iter().enumerate() {
        if c == b'=' {
            pad += 1;
            // '=' 之后只允许 '='
            if clean[i..].iter().any(|&b| b != b'=') {
                return None;
            }
            break;
        }
        if pad > 0 {
            return None;
        }
        let v = lut[c as usize];
        if v == 255 {
            return None;
        }
        buf = (buf << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    // 剩余位非零 → 非法（严格）
    if bits > 0 && buf != 0 {
        return None;
    }
    Some(out)
}

/// percent 解码（decodeURIComponent：%XX UTF-8 字节流；非法 → None）。
fn percent_decode_utf8(s: &str) -> Option<Vec<u8>> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'%' {
            if i + 2 >= bytes.len() {
                return None;
            }
            let h = std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?;
            let v = u8::from_str_radix(h, 16).ok()?;
            out.push(v);
            i += 3;
        } else {
            out.push(b);
            i += 1;
        }
    }
    Some(out)
}

// ---------------------------------------------------------------------------
// 字符串方法（字面量接收者）
// ---------------------------------------------------------------------------

/// UTF-16 单元序列（JS 字符串模型）。
fn utf16_units(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// 单元切片 → String（切在代理对中间 → None，拒绝折叠）。
fn units_to_string(units: &[u16]) -> Option<String> {
    String::from_utf16(units).ok()
}

fn str_method(s: &str, m: &str, args: &[ArgVal]) -> Option<FoldVal> {
    let u = utf16_units(s);
    let int_of = |v: &ArgVal| -> Option<f64> {
        match v {
            ArgVal::Int(i) => Some(*i as f64),
            ArgVal::Num(n) => Some(*n),
            ArgVal::Str(_) => None,
        }
    };
    let clamp_index = |v: f64| -> Option<usize> {
        if v.is_nan() {
            return None; // ToInteger(NaN) = 0 for slice but charAt(NaN)=0……
        }
        let t = v.trunc();
        let i = if t < 0.0 {
            (u.len() as f64 + t).max(0.0) as usize
        } else {
            (t as usize).min(u.len())
        };
        Some(i)
    };
    match m {
        "length" if args.is_empty() => Some(FoldVal::Int(u.len() as i64)),
        "charAt" => {
            let i = if args.is_empty() { 0.0 } else { int_of(&args[0])? };
            let t = i.trunc();
            if t.is_nan() || t < 0.0 || t >= u.len() as f64 {
                return Some(FoldVal::Str(String::new()));
            }
            let unit = u[t as usize];
            Some(FoldVal::Str(String::from_utf16(&[unit]).ok()?))
        }
        "charCodeAt" => {
            let i = if args.is_empty() { 0.0 } else { int_of(&args[0])? };
            let t = i.trunc();
            if t.is_nan() || t < 0.0 || t >= u.len() as f64 {
                return None; // NaN——不折
            }
            Some(FoldVal::Int(u[t as usize] as i64))
        }
        "codePointAt" => {
            let i = if args.is_empty() { 0.0 } else { int_of(&args[0])? };
            let t = i.trunc();
            if t.is_nan() || t < 0.0 || t >= u.len() as f64 {
                return None;
            }
            let idx = t as usize;
            let first = u[idx];
            if (0xD800..0xDC00).contains(&first) && idx + 1 < u.len() {
                let second = u[idx + 1];
                if (0xDC00..0xE000).contains(&second) {
                    let cp = 0x10000 + ((first as u32 - 0xD800) << 10) + (second as u32 - 0xDC00);
                    return Some(FoldVal::Int(cp as i64));
                }
            }
            Some(FoldVal::Int(first as i64))
        }
        "at" => {
            let i = if args.is_empty() { 0.0 } else { int_of(&args[0])? };
            let t = i.trunc();
            if t.is_nan() {
                return Some(FoldVal::Str(String::new()));
            }
            let idx = if t < 0.0 { u.len() as f64 + t } else { t };
            if idx < 0.0 || idx >= u.len() as f64 {
                return None; // undefined——不折
            }
            Some(FoldVal::Str(String::from_utf16(&[u[idx as usize]]).ok()?))
        }
        "slice" => {
            let start = if args.is_empty() { 0.0 } else { int_of(&args[0])? };
            let end = if args.len() < 2 { u.len() as f64 } else { int_of(&args[1])? };
            let s0 = clamp_index(start)?;
            let e0 = clamp_index(end)?;
            let (a, b) = if s0 < e0 { (s0, e0) } else { (e0, e0) };
            units_to_string(&u[a..b]).map(FoldVal::Str)
        }
        "substring" => {
            let start = if args.is_empty() { 0.0 } else { int_of(&args[0])? };
            let end = if args.len() < 2 { u.len() as f64 } else { int_of(&args[1])? };
            // substring 负值 → 0；NaN → 0
            let fix = |v: f64| -> usize {
                if v.is_nan() || v < 0.0 {
                    0
                } else {
                    (v.trunc() as usize).min(u.len())
                }
            };
            let a = fix(start);
            let b = fix(end);
            let (a, b) = if a <= b { (a, b) } else { (b, a) };
            units_to_string(&u[a..b]).map(FoldVal::Str)
        }
        "substr" => {
            let start = if args.is_empty() { 0.0 } else { int_of(&args[0])? };
            let len = if args.len() < 2 { u.len() as f64 } else { int_of(&args[1])? };
            let a = if start.is_nan() {
                0
            } else if start < 0.0 {
                (u.len() as f64 + start.trunc()).max(0.0) as usize
            } else {
                (start.trunc() as usize).min(u.len())
            };
            let n = if len.is_nan() {
                0
            } else {
                len.trunc().max(0.0) as usize
            };
            let b = (a + n).min(u.len());
            units_to_string(&u[a..b]).map(FoldVal::Str)
        }
        "toUpperCase" if args.is_empty() => {
            let out: String = s.to_uppercase();
            (out.len() < 1 << 20).then_some(FoldVal::Str(out))
        }
        "toLowerCase" if args.is_empty() => {
            let out: String = s.to_lowercase();
            (out.len() < 1 << 20).then_some(FoldVal::Str(out))
        }
        "trim" if args.is_empty() => Some(FoldVal::Str(
            s.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}').to_string(),
        )),
        "trimStart" | "trimLeft" if args.is_empty() => Some(FoldVal::Str(
            s.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{feff}').to_string(),
        )),
        "trimEnd" | "trimRight" if args.is_empty() => Some(FoldVal::Str(
            s.trim_end_matches(|c: char| c.is_whitespace() || c == '\u{feff}').to_string(),
        )),
        "indexOf" => {
            let needle = match args.first() {
                Some(ArgVal::Str(t)) => t.clone(),
                _ => return None,
            };
            let from = if args.len() < 2 {
                0usize
            } else {
                let v = int_of(&args[1])?;
                if v.is_nan() {
                    0
                } else if v < 0.0 {
                    0
                } else {
                    (v.trunc() as usize).min(u.len())
                }
            };
            let hay = String::from_utf16(&u).ok()?;
            let sub: String = String::from_utf16(&utf16_units(&hay)).ok()?;
            let _ = sub;
            let from_byte = byte_index_of_unit(&hay, from);
            let hay_tail = &hay[from_byte..];
            match hay_tail.find(&needle) {
                Some(bi) => {
                    // 字节位 → UTF-16 单元位
                    let units_before = hay[..from_byte + bi].encode_utf16().count();
                    Some(FoldVal::Int(units_before as i64))
                }
                None => Some(FoldVal::Int(-1)),
            }
        }
        "lastIndexOf" => {
            let needle = match args.first() {
                Some(ArgVal::Str(t)) => t.clone(),
                _ => return None,
            };
            let hay = String::from_utf16(&u).ok()?;
            match hay.rfind(&needle) {
                Some(bi) => {
                    let units_before = hay[..bi].encode_utf16().count();
                    Some(FoldVal::Int(units_before as i64))
                }
                None => Some(FoldVal::Int(-1)),
            }
        }
        "includes" => {
            let needle = match args.first() {
                Some(ArgVal::Str(t)) => t.encode_utf16().collect::<Vec<u16>>(),
                _ => return None,
            };
            let from = pos_arg(args.get(1), u.len(), 0)?;
            if needle.is_empty() {
                return Some(FoldVal::Bool(true));
            }
            let found = u[from..]
                .windows(needle.len())
                .any(|w| w == needle.as_slice());
            Some(FoldVal::Bool(found))
        }
        "startsWith" => {
            let needle = match args.first() {
                Some(ArgVal::Str(t)) => t.encode_utf16().collect::<Vec<u16>>(),
                _ => return None,
            };
            let pos = pos_arg(args.get(1), u.len(), 0)?;
            let ok = pos + needle.len() <= u.len()
                && u[pos..pos + needle.len()] == needle[..];
            Some(FoldVal::Bool(ok))
        }
        "endsWith" => {
            let needle = match args.first() {
                Some(ArgVal::Str(t)) => t.encode_utf16().collect::<Vec<u16>>(),
                _ => return None,
            };
            let end = pos_arg(args.get(1), u.len(), u.len())?;
            let ok = needle.len() <= end && u[end - needle.len()..end] == needle[..];
            Some(FoldVal::Bool(ok))
        }
        "repeat" => {
            let n = int_of(args.first()?)?;
            if n.is_nan() || n < 0.0 {
                return None;
            }
            let n = n.trunc() as usize;
            let total = s.len().checked_mul(n)?;
            if total > 1 << 20 {
                return None; // 规模上限
            }
            Some(FoldVal::Str(s.repeat(n)))
        }
        "padStart" | "padEnd" => {
            let target = int_of(args.first()?)?;
            if target.is_nan() || target < 0.0 {
                return Some(FoldVal::Str(s.to_string()));
            }
            let target = target.trunc() as usize;
            let fill = match args.get(1) {
                Some(ArgVal::Str(f)) => f.clone(),
                None => " ".to_string(),
                _ => return None,
            };
            if fill.is_empty() || target <= u.len() {
                return Some(FoldVal::Str(s.to_string()));
            }
            let need = target - u.len();
            let fill_units = utf16_units(&fill);
            let mut pad: Vec<u16> = Vec::with_capacity(need);
            for k in 0..need {
                pad.push(fill_units[k % fill_units.len()]);
            }
            let pad_s = units_to_string(&pad)?;
            let out = if m == "padStart" {
                format!("{pad_s}{s}")
            } else {
                format!("{s}{pad_s}")
            };
            (out.len() < 1 << 20).then_some(FoldVal::Str(out))
        }
        "concat" => {
            let mut out = s.to_string();
            for a in args {
                match a {
                    ArgVal::Str(t) => out.push_str(t),
                    ArgVal::Int(i) => out.push_str(&i.to_string()),
                    ArgVal::Num(n) => {
                        let t = format_js_number(*n);
                        if t.contains("Infinity") || t == "NaN" {
                            return None;
                        }
                        out.push_str(&t)
                    }
                }
            }
            (out.len() < 1 << 20).then_some(FoldVal::Str(out))
        }
        _ => None,
    }
}

/// 字节索引 → 至少覆盖到该 UTF-16 单元位之前的字节位。
fn byte_index_of_unit(s: &str, unit_idx: usize) -> usize {
    let mut units = 0usize;
    for (bi, c) in s.char_indices() {
        if units >= unit_idx {
            return bi;
        }
        units += c.len_utf16();
    }
    s.len()
}

// ---------------------------------------------------------------------------
// try 清理
// ---------------------------------------------------------------------------

/// `try{A} finally{}` → A；`try{A}catch(e){throw e} finally{}` → A；
/// `try{A}catch(e){throw e}` → A。
pub struct TsTryCleanup;

impl Rule<TsAst> for TsTryCleanup {
    fn name(&self) -> &'static str {
        "ts_try_cleanup"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Try]
    }
    fn structural(&self) -> bool {
        true
    }
    fn check(&self, ctx: RewriteCtx<'_, TsAst>, id: TsId) -> Option<Edit<TsAst>> {
        let RewriteCtx { lang, .. } = ctx;
        let ch = lang.children(id).to_vec();
        let try_block = *ch.first()?;
        let mut catch = None;
        let mut finally = None;
        for &c in &ch[1..] {
            if lang.kind(c) == NodeKind::Catch {
                catch = Some(c);
            } else {
                finally = Some(c);
            }
        }
        // finally 必须空（或不存在）
        if let Some(f) = finally {
            if lang.kind(f) != NodeKind::Block || !lang.children(f).is_empty() {
                return None;
            }
        }
        // catch：不存在 或 体仅 `throw e`（e 为 catch 参数）
        let catch_ok = match catch {
            None => true,
            Some(c) => {
                let NodeData::Catch { param_raw } = lang.data(c) else {
                    return None;
                };
                let pname = param_raw.as_deref().unwrap_or("");
                let body = lang.children(c).first().copied()?;
                let stmts = lang.children(body).to_vec();
                if stmts.len() != 1 || lang.kind(stmts[0]) != NodeKind::Throw {
                    return None;
                }
                let e = lang.children(stmts[0]).first().copied()?;
                matches!(lang.data(e), NodeData::VarRef { name } if lang.sn(*name) == pname)
            }
        };
        if !catch_ok {
            return None;
        }
        // try 块替换整条（含 catch/finally 的形态）
        Some(Edit::Replace {
            target: id,
            with: try_block,
        })
    }
}

// ---------------------------------------------------------------------------
// else 展平（then 终结后）
// ---------------------------------------------------------------------------

/// `if (c) { …; return } else { X } [之后代码]` 且 else 体不终结 →
/// else 块内容可直接接在 if 之后（then 已终结，else 是唯一可能落空的
/// 分支）。Java 版只处理 else 单 return 的删除；本版处理**else 整体
/// 展平**——语言无关逻辑，JS 的 return/throw 终结性相同。
pub struct TsTrailingElseFlatten;

impl Rule<TsAst> for TsTrailingElseFlatten {
    fn name(&self) -> &'static str {
        "ts_trailing_else_flatten"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::If]
    }
    fn structural(&self) -> bool {
        true
    }
    fn check(&self, ctx: RewriteCtx<'_, TsAst>, id: TsId) -> Option<Edit<TsAst>> {
        let parent = ctx.parent(id)?;
        let idx = ctx.index(id)?;
        let RewriteCtx { lang, .. } = ctx;
        if lang.kind(parent) != NodeKind::Block {
            return None;
        }
        let ch = lang.children(id).to_vec();
        if ch.len() != 3 {
            return None;
        }
        let then = ch[1];
        let els = ch[2];
        // then 必须块终结（return/throw）
        if lang.kind(then) != NodeKind::Block {
            return None;
        }
        let then_stmts = lang.children(then).to_vec();
        if !then_stmts
            .last()
            .is_some_and(|&s| matches!(lang.kind(s), NodeKind::Return | NodeKind::Throw))
        {
            return None;
        }
        // else 必须块
        if lang.kind(els) != NodeKind::Block {
            return None;
        }
        let else_stmts = lang.children(els).to_vec();
        // else 体终结 → 无需展平（语义等价但形态不变；避免与 if_to_ternary 竞争）
        if else_stmts
            .last()
            .is_some_and(|&s| matches!(lang.kind(s), NodeKind::Return | NodeKind::Throw))
        {
            return None;
        }
        // 展平：if 去掉 else 臂 + else 块内容按 Splice 插入父块紧随其后
        Some(Edit::Multi(vec![
            Edit::Splice {
                node: id,
                index: 2,
                remove: 1,
                insert: Vec::new(),
            },
            Edit::Splice {
                node: parent,
                index: idx + 1,
                remove: 0,
                insert: else_stmts,
            },
        ]))
    }
}

// ---------------------------------------------------------------------------
// x = e; return x  →  return e
// ---------------------------------------------------------------------------

/// `x = e; return x;` → `return e;`（x 为局部；写被消费）。
pub struct TsTrailingAssignReturn;

impl Rule<TsAst> for TsTrailingAssignReturn {
    fn name(&self) -> &'static str {
        "ts_trailing_assign_return"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::Assign]
    }
    fn structural(&self) -> bool {
        true
    }
    fn check(&self, ctx: RewriteCtx<'_, TsAst>, id: TsId) -> Option<Edit<TsAst>> {
        // parent 链先取（ctx 借用完成后才能拿 &mut lang）
        let parent = ctx.parent(id)?;
        let grand = ctx.parent(parent);
        let RewriteCtx { lang, .. } = ctx;
        if !matches!(lang.data(id), NodeData::Assign { op: None }) {
            return None;
        }
        let ch = lang.children(id).to_vec();
        if ch.len() != 2 {
            return None;
        }
        let (target, value) = (ch[0], ch[1]);
        if lang.kind(target) != NodeKind::VarRef || !lang.is_local_var(target) {
            return None;
        }
        let name = lang.var_name(target)?.to_string();
        // 语句形态：ExprStmt 包裹或裸 Assign
        let (stmt, block) = if lang.kind(parent) == NodeKind::ExprStmt {
            (parent, grand?)
        } else if lang.kind(parent) == NodeKind::Block {
            (id, parent)
        } else {
            return None;
        };
        if lang.kind(block) != NodeKind::Block {
            return None;
        }
        // 语句在块中的位置（用 walk 的 index——parent 已确认）
        let idx = {
            let siblings = lang.children(block).to_vec();
            siblings.iter().position(|&s| s == stmt)?
        };
        let stmts = lang.children(block).to_vec();
        let next = *stmts.get(idx + 1)?;
        if lang.kind(next) != NodeKind::Return {
            return None;
        }
        let ret_ch = lang.children(next).to_vec();
        if ret_ch.len() != 1 {
            return None;
        }
        if !matches!(lang.data(ret_ch[0]), NodeData::VarRef { .. }) {
            return None;
        }
        if lang.var_name(ret_ch[0]) != Some(name.as_str()) {
            return None;
        }
        if lang.effect(value) > Effect::MayRead {
            return None;
        }
        Some(Edit::Multi(vec![
            Edit::Replace {
                target: ret_ch[0],
                with: value,
            },
            Edit::Delete { node: stmt },
        ]))
    }
}

// ---------------------------------------------------------------------------
// 丢弃结果的纯调用删除
// ---------------------------------------------------------------------------

/// `Math.floor(x);` / `"abc".trim();` / `parseInt(x);` 这类无副作用的
/// 表达式语句删除。白名单：Math.* 静态成员调用 + 字面量接收者的
/// 纯字符串方法 + parseInt/parseFloat/Number/String/atob/
/// decodeURIComponent/isNaN/isFinite。实参副作用 ≤ MayRead。
pub struct TsDeadPureCall;

impl Rule<TsAst> for TsDeadPureCall {
    fn name(&self) -> &'static str {
        "ts_dead_pure_call"
    }
    fn kinds(&self) -> &'static [NodeKind] {
        &[NodeKind::ExprStmt]
    }
    fn check(&self, ctx: RewriteCtx<'_, TsAst>, id: TsId) -> Option<Edit<TsAst>> {
        let RewriteCtx { lang, .. } = ctx;
        let ch = lang.children(id).to_vec();
        if ch.len() != 1 || lang.kind(ch[0]) != NodeKind::Call {
            return None;
        }
        let call = ch[0];
        if matches!(lang.data(call), NodeData::Call { optional: true }) {
            return None;
        }
        let cch = lang.children(call).to_vec();
        let callee = *cch.first()?;
        let args = &cch[1..];
        // 实参必须是**原始字面量**：任何对象实参都会经 ToPrimitive 调用
        // 用户 valueOf/toString（可抛出/带副作用）——`Number(obj1)` 在
        // assert.throws 里被删导致 Test262 return-abrupt 家族失败。
        for &a in args {
            if lang.effect(a) > Effect::MayRead {
                return None;
            }
            if !is_primitive_literal(lang, a) {
                return None;
            }
        }
        let pure = match lang.data(callee) {
            NodeData::Member { .. } => {
                let base = lang.children(callee).first().copied()?;
                let member = lang.member_name(callee)?.to_string();
                match lang.data(base) {
                    NodeData::VarRef { name } => match lang.sn(*name) {
                        "Math" => is_math_pure(&member),
                        _ => false,
                    },
                    NodeData::Literal(JsLit::Str(_)) => {
                        matches!(
                            member.as_str(),
                            // repeat/padStart/padEnd 对负/超限参数抛
                            // RangeError——表达式语句删除会丢异常
                            //（Test262 repeat/count-less-than-zero 抓获）。
                            "toUpperCase" | "toLowerCase" | "trim" | "trimStart"
                                | "trimEnd" | "trimLeft" | "trimRight" | "charAt"
                                | "charCodeAt" | "codePointAt" | "at" | "slice"
                                | "substring" | "substr" | "indexOf" | "lastIndexOf"
                                | "includes" | "startsWith" | "endsWith" | "concat"
                        )
                    }
                    _ => false,
                }
            }
            NodeData::VarRef { name } => matches!(
                lang.sn(*name),
                // atob/btoa/decodeURI*/encodeURI* 对畸形字面量抛
                // URIError/InvalidCharacterError——死调用删除会丢异常
                //（Test262 decodeURI 家族抓获）。
                "parseInt" | "parseFloat" | "Number" | "String" | "isNaN" | "isFinite"
            ),
            _ => false,
        };
        if !pure {
            return None;
        }
        Some(Edit::Delete { node: id })
    }
}

/// 原始值字面量（或其一元 +/-）：实参侧唯一可安全丢弃的形态。
fn is_primitive_literal(lang: &TsAst, n: TsId) -> bool {
    match lang.data(n) {
        // BigInt 在 ToNumber/ToInteger 上下文抛 TypeError；正则/对象
        // 是引用语义——都不算可安全丢弃实参
        NodeData::Literal(JsLit::BigInt(_)) | NodeData::Literal(JsLit::Regex(_)) => false,
        NodeData::Literal(_) => true,
        NodeData::Unary { op: cure_engine::kind::UnOp::Neg | cure_engine::kind::UnOp::Plus } => lang
            .children(n)
            .first()
            .is_some_and(|&c| matches!(lang.data(c), NodeData::Literal(_))),
        _ => false,
    }
}

fn is_math_pure(m: &str) -> bool {
    matches!(
        m,
        "abs" | "ceil" | "floor" | "round" | "trunc" | "sign" | "sqrt" | "cbrt"
            | "exp" | "log" | "log2" | "log10" | "sin" | "cos" | "tan" | "asin"
            | "acos" | "atan" | "atan2" | "pow" | "min" | "max" | "hypot" | "imul"
            | "fround" | "clz32"
    )
}

// ---------------------------------------------------------------------------
// 注册
// ---------------------------------------------------------------------------

/// TS 专属规则列表（在 collect/facade 侧注册）。
pub fn ported_rules() -> Vec<Box<dyn Rule<TsAst>>> {
    vec![
        Box::new(TsLiteralEval),
        Box::new(TsTryCleanup),
        Box::new(TsTrailingElseFlatten),
        Box::new(TsTrailingAssignReturn),
        Box::new(TsDeadPureCall),
    ]
}