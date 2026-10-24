//! 差分验证（黄金标准，node 对拍）：对每个 JS 程序——
//!   原版.js ──node──▶ stdout/exit
//!   简化版.js ──node──▶ stdout/exit   ⇒ 两者必须完全一致
//!
//! 覆盖 JS 语义雷区（每条都是某类错误简化的「陷阱弹」）：
//!   - **-0 / NaN / Infinity**：打印与 1/x 符号敏感
//!   - **字符串胁迫**：`"5" * 1` ≠ `"5"`——is_exact_int=false 的理由
//!   - **var 提升**：块外引用读到 undefined
//!   - **闭包捕获**：lambda 里的读时机
//!   - **真值语义**：`""`/`"0"`/`NaN` 均为假值但互不相等
//!   - **ASI**：换行隐式分号
//!   - **比较语义**：`==` vs `===` 的胁迫差异
//!   - **逗号序列 / 副作用顺序**
//!
//! 本机无 node 时自动跳过。

use std::fs;
use std::path::Path;
use std::process::Command;

use cure_engine::Config;
use cure_ts_parser::parse;
use cure_ts_print::print_module;
use cure_ts_simplify::simplify_unit;

fn node_bin() -> Option<&'static str> {
    for c in ["node", "/Users/e/.local/share/vite-plus/bin/node"] {
        if Command::new(c).arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
            return Some(c);
        }
    }
    None
}

fn run_js(bin: &str, file: &Path) -> (String, Option<i32>) {
    let out = Command::new(bin).arg(file).output().expect("run node");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        out.status.code(),
    )
}

/// 差分：原版 vs 简化版运行输出逐字节一致。
fn differential(name: &str, src: &str) {
    differential_cfg(name, src, Config::default());
}

fn differential_cfg(name: &str, src: &str, cfg: Config) {
    let Some(node) = node_bin() else {
        eprintln!("skip differential: node not found");
        return;
    };
    let base = std::env::temp_dir().join(format!("cure_ts_diff_{name}"));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();

    let orig_file = base.join("orig.js");
    fs::write(&orig_file, src).unwrap();

    let mut outcome = parse(src);
    assert!(
        outcome.errors.is_empty(),
        "{name}: 测试源码必须干净解析：{:?}",
        outcome.errors
    );
    let _report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &cfg);
    let simplified = print_module(&outcome.ast, &outcome.unit);
    let simp_file = base.join("simp.js");
    fs::write(&simp_file, &simplified).unwrap();

    let (orig_out, orig_code) = run_js(node, &orig_file);
    let (simp_out, simp_code) = run_js(node, &simp_file);
    assert_eq!(
        orig_code, simp_code,
        "{name}: 退出码改变\n== 原版 ==\n{orig_out}\n== 简化 ==\n{simp_out}\n== 简化源 ==\n{simplified}"
    );
    assert_eq!(
        orig_out, simp_out,
        "{name}: 语义改变！\n== 原版 ==\n{orig_out}\n== 简化 ==\n{simp_out}\n== 简化源 ==\n{simplified}"
    );
}

/// 差分 + 断言发生了改写（防「简化=复读机」的假绿）。
fn differential_edits(name: &str, src: &str, min_edits: usize) {
    let Some(node) = node_bin() else {
        eprintln!("skip differential: node not found");
        return;
    };
    let base = std::env::temp_dir().join(format!("cure_ts_diff_{name}"));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let orig_file = base.join("orig.js");
    fs::write(&orig_file, src).unwrap();
    let mut outcome = parse(src);
    assert!(outcome.errors.is_empty(), "{name}: {:#?}", outcome.errors);
    let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    assert!(
        report.edits >= min_edits,
        "{name}: 预期 ≥{min_edits} 次改写，实际 {} 次",
        report.edits
    );
    let simplified = print_module(&outcome.ast, &outcome.unit);
    let simp_file = base.join("simp.js");
    fs::write(&simp_file, &simplified).unwrap();
    let (orig_out, orig_code) = run_js(node, &orig_file);
    let (simp_out, simp_code) = run_js(node, &simp_file);
    assert_eq!(orig_code, simp_code, "{name}: 退出码改变\n{simplified}");
    assert_eq!(orig_out, simp_out, "{name}: 语义改变！\n== 简化源 ==\n{simplified}");
}

// ---------------------------------------------------------------------------
// 差分用例（黄金集）
// ---------------------------------------------------------------------------

#[test]
fn diff_register_noise() {
    // 混淆器寄存器噪声的经典形态：拷贝链 + 死存储 + 常量折叠
    differential_edits(
        "register_noise",
        r#"
function f(x) {
    var a = 2;
    var b = 3;
    var c = a * b;
    var d = c;
    var e = 0;
    e = d + x;
    var g = e * 1;
    return g;
}
console.log(f(4), f(-1));
"#,
        3,
    );
}

#[test]
fn diff_dead_store_chain() {
    differential_edits(
        "dead_store_chain",
        r#"
function f() {
    var a = 1;
    a = 2;
    a = 3;
    var b = a;
    var c = 0;
    c = b + 4;
    return c;
}
console.log(f());
"#,
        2,
    );
}

#[test]
fn diff_negative_zero() {
    // -0：1/x 的符号敏感（打印必须保留 "-0"）
    differential(
        "negative_zero",
        r#"
function f(x) { return 1 / x; }
console.log(f(0), f(-0), 1 / -0, -0 === 0);
var z = -0;
console.log(1 / z, `${z}`, String(z), z + 0);
"#,
    );
}

#[test]
fn diff_nan_semantics() {
    // NaN：比较恒假（不得当普通值折叠）
    differential(
        "nan",
        r#"
var n = NaN;
console.log(n === n, n == n, isNaN(n), n ? 1 : 2);
function g(x) { return x !== x; }
console.log(g(NaN), g(1));
"#,
    );
}

#[test]
fn diff_string_coercion() {
    // 字符串数值胁迫：is_exact_int=false 的存在理由
    //（`"5" * 1 === 5` ≠ "5"；`x * 1` 不得折叠为 x）
    differential(
        "string_coercion",
        r#"
function mul(x) { return x * 1; }
function sub(x) { return x - 0; }
console.log(mul("5"), mul(5), sub("5"), sub(5));
console.log("5" == 5, "5" === 5);
console.log("a" + 1, 1 + "a", "" + 1, `${1}${2}`);
"#,
    );
}

#[test]
fn diff_var_hoisting() {
    // var 提升：块外引用读到 undefined（删声明 → ReferenceError）
    differential(
        "var_hoisting",
        r#"
function f(c) {
    var r = "before";
    if (c) { var r = "in"; }
    return r;
}
console.log(f(true), f(false));
function g() {
    try { console.log(x); return; } finally { }
    var x = 1;
}
g();
"#,
    );
}

#[test]
fn diff_closure_capture() {
    // 闭包捕获时机：值传播不得改变捕获语义
    differential(
        "closure",
        r#"
function counter() {
    var n = 0;
    return function () { n = n + 1; return n; };
}
var c1 = counter();
var c2 = counter();
console.log(c1(), c1(), c2());
function late() {
    var x = 1;
    var f = function () { return x; };
    x = 2;
    return f();
}
console.log(late());
"#,
    );
}

#[test]
fn diff_truthiness() {
    // 真值语义：""/"0"/NaN 都是假值；!! 不得折叠为恒等
    differential(
        "truthiness",
        r#"
function t(x) { if (x) { return "truthy"; } return "falsy"; }
console.log(t(""), t("0"), t(0), t(NaN), t(null), t(undefined), t([]), t({}));
function d(x) { return !!x; }
console.log(d(0), d("x"), d(null));
"#,
    );
}

#[test]
fn diff_loose_vs_strict() {
    differential(
        "eq_semantics",
        r#"
console.log(0 == "", 0 === "", null == undefined, null === undefined);
console.log([1] == 1, "1" == true);
function f(a, b) { return a != b && a !== b; }
console.log(f(1, 2), f("1", 1), f(null, undefined));
"#,
    );
}

#[test]
fn diff_asi() {
    // ASI：return 换行；++ 后缀换行
    differential(
        "asi",
        r#"
function f() {
    return
    1;
}
function g(x) {
    var a = x
    ++x
    return a + x
}
console.log(f(), g(5));
"#,
    );
}

#[test]
fn diff_sequence_and_side_effects() {
    // 逗号序列副作用顺序
    differential(
        "seq",
        r#"
var log = [];
function f(x) { log.push(x); return x * 2; }
var r = (f(1), f(2), f(3));
console.log(r, log.join(","));
var i = 0;
var j = (i++, i++ + 2, i);
console.log(i, j);
"#,
    );
}

#[test]
fn diff_switch_loops() {
    differential(
        "switch_loops",
        r#"
function f(n) {
    var s = 0;
    var i = 0;
    while (i < n) {
        s = s + i;
        i = i + 1;
    }
    switch (s) {
        case 0: return "zero";
        case 1: return "one";
        default: return "many:" + s;
    }
}
console.log(f(0), f(1), f(5));
"#,
    );
}

#[test]
fn diff_dead_code_after_return() {
    // return 之后的语句（引擎保守不动）
    differential(
        "dead_code",
        r#"
function f() {
    var a = 1;
    return a;
    var b = 2;
    a = 3;
}
console.log(f());
"#,
    );
}

#[test]
fn diff_template_literals() {
    differential(
        "templates",
        r#"
var name = "world";
var n = 3;
console.log(`hello ${name} ${n + 1}`);
console.log(`${`${n}`}`);
console.log(`a\nb${"x"}c`);
"#,
    );
}

#[test]
fn diff_number_printing() {
    // ECMA Number→String 的指数阈值
    differential(
        "num_print",
        r#"
console.log(0.1, 100, 123.456, 1e21, 1e-7, 5e-7, 9007199254740992);
console.log(0.5, -2.5, 1 / 3, 2 ** 53, 2 ** 53 + 1 === 2 ** 53);
console.log((0.1 + 0.2) === 0.3, 0.1 + 0.2);
"#,
    );
}

#[test]
fn diff_optional_chain_and_nullish() {
    differential(
        "opt_nullish",
        r#"
var o = { a: { b: 1 } };
console.log(o?.a?.b, o?.x?.y, o.x?.y);
var v = null;
console.log(v ?? "dflt", 0 ?? "not", "" ?? "not", undefined ?? "dflt");
var w = undefined;
w ??= 5;
console.log(w);
var q = 0;
q ||= 10;
console.log(q);
var p = 1;
p &&= 2;
console.log(p);
"#,
    );
}

#[test]
fn diff_try_catch_finally() {
    // finally 观察中间值（store_kill 异常窗口守卫的存在理由）
    differential(
        "try_finally",
        r#"
function f(x) {
    var v = true;
    try {
        if (x) { throw new Error("boom"); }
        v = false;
        return "ok";
    } catch (e) {
        return "err";
    } finally {
        console.log("finally:" + v);
    }
}
console.log(f(0), f(1));
"#,
    );
}

#[test]
fn diff_logical_short_circuit() {
    // && || 短路：右侧不求值（副作用不得搬移）
    differential(
        "short_circuit",
        r#"
var calls = 0;
function side(x) { calls = calls + 1; return x; }
console.log(false && side(1), true || side(2), calls);
console.log(true && side(3), false || side(4), calls);
var a = 0;
var b = a || "dflt";
console.log(b);
"#,
    );
}

#[test]
fn diff_class_and_arrow() {
    // 注：`var t = this.n + by; this.n = t` 的 t 传播被求值顺序守卫
    // 拒绝（赋值前缀 this.n 读在 t 之前——代理接收者不可证明无副作用），
    // 0 编辑是正确保守行为——本用例验证语义保持。
    differential(
        "class_arrow",
        r#"
class Counter {
    constructor(start) {
        this.n = start;
    }
    inc(by) {
        var t = this.n + by;
        this.n = t;
        return this.n;
    }
    get() { return this.n; }
}
var c = new Counter(10);
console.log(c.inc(1), c.inc(2), c.get());
var add = (a, b) => a + b;
var mul = (a, b) => { return a * b; };
console.log(add(2, 3), mul(4, 5));
"#,
    );
}

#[test]
fn diff_destructuring() {
    differential(
        "destructuring",
        r#"
var [a, b = 5, ...rest] = [1, undefined, 3, 4];
console.log(a, b, rest.join(","));
var { x, y: yy, z = 7 } = { x: 1, y: 2 };
console.log(x, yy, z);
function f({ p, q }) { return p + q; }
console.log(f({ p: 1, q: 2 }));
"#,
    );
}

#[test]
fn diff_regexp() {
    differential(
        "regexp",
        r#"
var re = /a+c/g;
console.log(re.source, re.test("aaac"), "aaac".replace(/a/g, "b"));
console.log("x".split(/x/).length);
"#,
    );
}

#[test]
fn diff_in_instanceof_typeof() {
    differential(
        "in_typeof",
        r#"
console.log("a" in { a: 1 }, "b" in { a: 1 }, 0 in [10], 0 in []);
console.log([] instanceof Array, [] instanceof Object, 1 instanceof Number);
console.log(typeof undefined, typeof x === "undefined", typeof null, typeof 1, typeof "s");
"#,
    );
}

#[test]
fn diff_ternary_chains() {
    differential_edits(
        "ternary",
        r#"
function grade(s) {
    var r = 0;
    r = s > 90 ? "A" : s > 80 ? "B" : s > 70 ? "C" : "F";
    return r;
}
console.log(grade(95), grade(85), grade(75), grade(50));
"#,
        1,
    );
}

#[test]
fn diff_void_and_await_like() {
    differential(
        "void_op",
        r#"
var u = void 0;
console.log(u, u === undefined, void "x");
var x;
console.log(x);
"#,
    );
}

#[test]
fn diff_label_break() {
    differential(
        "labels",
        r#"
outer:
for (var i = 0; i < 3; i++) {
    for (var j = 0; j < 3; j++) {
        if (j === 1) { continue outer; }
        if (i === 2) { break outer; }
        console.log(i, j);
    }
}
"#,
    );
}

#[test]
fn diff_string_methods() {
    differential(
        "strings",
        r#"
var s = "Hello";
console.log(s.length, s.toLowerCase(), s.indexOf("l"), s.slice(1, 3));
console.log(s.charAt(1), s[1], "a,b,c".split(",").join("|"));
console.log("  x  ".trim(), "x".repeat(3), "ab".concat("cd"));
"#,
    );
}

#[test]
fn diff_sorting_object() {
    // 对象/数组同一性：不得值复制传播
    differential(
        "obj_identity",
        r#"
var arr = [3, 1, 2];
var copy = arr;
copy.sort();
console.log(arr.join(","), copy === arr);
var o1 = { v: 1 };
var o2 = o1;
o2.v = 2;
console.log(o1.v, o1 === o2);
"#,
    );
}

// ---------------------------------------------------------------------------
// 幂等：cure(cure(x)) 输出稳定（打印一致性）
// ---------------------------------------------------------------------------

#[test]
fn idempotent_printing() {
    let srcs = [
        "var x = 1;\nfunction f(a) { return a + x; }\n",
        "function f(n) { var s = 0; for (var i = 0; i < n; i++) { s += i; } return s; }\nconsole.log(f(10));\n",
        "const g = (a, b) => a * (b + 1);\nconst h = a => { return a ? 1 : -1; };\n",
        "class A { constructor(x) { this.x = x; } m() { return this.x * 2; } }\n",
        "try { f(); } catch (e) { console.log(e); } finally { done(); }\n",
        "switch (x) { case 1: a(); break; case 2: b(); default: c(); }\n",
        "var t = `pre${x}mid${y}post`;\nvar u = { a: 1, b, m() { return 2; } };\n",
        "var r = a ? b : c;\nvar p = x ?? y;\nvar q = (m, n) => m + n;\n",
        "do { x(); } while (cond);\nwhile (true) { break; }\n",
        "for (const k of [1, 2]) { console.log(k); }\nfor (const k in { a: 1 }) { console.log(k); }\n",
    ];
    for (i, src) in srcs.iter().enumerate() {
        let mut out1 = parse(src);
        assert!(out1.errors.is_empty(), "#{i}: {:#?}", out1.errors);
        let _ = simplify_unit(&mut out1.ast, &mut out1.unit, &Config::default());
        let once = print_module(&out1.ast, &out1.unit);
        let mut out2 = parse(&once);
        assert!(
            out2.errors.is_empty(),
            "#{i}: 打印输出不可再解析：{:#?}\n== once ==\n{once}",
            out2.errors
        );
        let _ = simplify_unit(&mut out2.ast, &mut out2.unit, &Config::default());
        let twice = print_module(&out2.ast, &out2.unit);
        assert_eq!(once, twice, "#{i}:\n== once ==\n{once}\n== twice ==\n{twice}");
    }
}

/// 解析→打印（不简化）的纯往返保真：输出再解析无错误。
#[test]
fn parse_print_roundtrip_clean() {
    let srcs = [
        "function f() { return -x ** 2; }\n",
        "var a = (1 + 2) * 3;\nvar b = 1 + 2 * 3;\nvar c = (2 ** 3) ** 2;\nvar d = 2 ** 3 ** 2;\n",
        "var e = a - -b;\nvar f1 = typeof x;\nvar g1 = void 0;\nvar h1 = delete o.p;\n",
        "var i = a?.b?.[c]?.(1);\nvar j = tag`t${x}`;\n",
        "var k = x as unknown as string;\n",
        "label1: for (;;) { break label1; }\n",
        "var m = { 'str key': 1, [comp]: 2, nest: { deep: [1, [2]] } };\n",
        "if (a) b(); else if (c) d(); else e();\n",
        "var n = 0x1F + 0b101 + 0o17 + 1_000 + 12n;\n",
    ];
    for (i, src) in srcs.iter().enumerate() {
        let mut out = parse(src);
        assert!(out.errors.is_empty(), "#{i} 源解析失败: {:#?}", out.errors);
        let printed = print_module(&out.ast, &out.unit);
        let re = parse(&printed);
        assert!(
            re.errors.is_empty(),
            "#{i} 打印输出不可再解析：{:#?}\n== printed ==\n{printed}",
            re.errors
        );
    }
}

// ---------------------------------------------------------------------------
// 混淆器对抗差分（obfuscator.io 风格——常量表/内联/折叠/死项全链路）
// ---------------------------------------------------------------------------

#[test]
fn obf_string_array_chain() {
    // 字符串数组 + 偏移访问器 + hex 拷贝链 + 真值噪声 + 引号成员
    //（module 标记启用死项清除）
    differential_edits(
        "obf_string_array",
        r#"
var _0x3f2a = ['\x68\x65\x6c\x6c\x6f', '\x77\x6f\x72\x6c\x64'];
var _0xoff = 0x1 + 0x1;
function _0xacc(i) { return _0x3f2a[i - _0xoff]; }
export function greet() {
    var _0xf = !![];
    var _0xg = ![];
    if (_0xf) { return _0xacc(0x2) + ' ' + _0xacc(0x3) + (_0xg === false ? '!' : '?'); }
    return '';
}
console.log(greet());
"#,
        5,
    );
}

#[test]
fn obf_hex_chain_and_bool() {
    differential_edits(
        "obf_hex_chain",
        r#"
var _0xk = 0x0a;
var _0xm = _0xk * 0x03 + 0x02;
function calc(x) {
    var _0xt = _0xm - 0x1c;
    var _0xu = _0xt + x;
    return _0xu;
}
export function run() { return [calc(1), calc(2), ![], !![]]; }
console.log(run().join(','));
"#,
        3,
    );
}

#[test]
fn obf_sandbox_safety() {
    // 不可折的形态保持原样：NaN 比较、字符串数值胁迫、动态下标
    differential(
        "obf_safety",
        r#"
var _0xs = '5';
var _0xdyn = 1;
function keep(x) {
    var a = x * 1;
    var b = x - 0;
    var arr = [10, 20, 30];
    return [a, b, arr[_0xdyn], x !== x, x == 5, x === 5];
}
export function run() { return keep(_0xs).join('|'); }
console.log(run());
"#,
    );
}

#[test]
fn obf_rotation_call_chain() {
    // 多调用点内联 + 数组折叠 + 控制流噪声
    differential_edits(
        "obf_rotation",
        r#"
var _0xtbl = ['a', 'b', 'c', 'd'];
var _0xrot = 0x0;
function _0xpick(i) { return _0xtbl[(i + _0xrot) % 0x4]; }
export function run() {
    var s = '';
    for (var i = 0x0; i < 0x4; i++) { s = s + _0xpick(i); }
    var flag = !![];
    if (!!flag) { return s + '-' + _0xpick(0x0); }
    return 'nope';
}
console.log(run());
"#,
        3,
    );
}

#[test]
fn diff_paren_array_identity() {
    // 括号包裹的数组常量不是标量：传播会复制数组同一性
    // （`([1,2]).push(3)` 不会改变原常量 A）。
    differential(
        "paren_array_identity",
        r#"
const A = ([1, 2]);
A.push(3);
console.log(A.length, A.join(","));
const B = ({ v: 1 });
B.v = 2;
console.log(B.v);
"#,
    );
}

#[test]
fn diff_await_identifier_in_nested_scope() {
    // 非 async 函数里 `await(...)` 是名为 await 的普通函数调用。
    differential(
        "await_identifier",
        r#"
var await = function (x) { return x + 1; };
function f() { return await(null); }
function g() { function h() { return await(1); } return h(); }
console.log(f(), g());
"#,
    );
}

#[test]
fn export_group_keeps_all_exports() {
    let src = "export var a = 1, b = 2;\n";
    let mut out = cure_ts_parser::parse_module(src);
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    let _ = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
    let printed = print_module(&out.ast, &out.unit);
    assert!(
        printed.contains("export var a") && printed.contains("export var b"),
        "multi-declarator export lost binding export:\n{printed}"
    );
}

#[test]
fn decorators_are_preserved() {
    for src in [
        "class C { @dec m() {} }\n",
        "class C { @ns.$ static m() {} }\n",
        "@dec class C { m() {} }\n",
    ] {
        let mut out = parse(src);
        assert!(out.errors.is_empty(), "{src}: {:?}", out.errors);
        let _ = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
        let once = print_module(&out.ast, &out.unit);
        assert!(once.contains("@dec") || once.contains("@ns.$"), "{src}:\n{once}");
        let mut out2 = parse(&once);
        assert!(out2.errors.is_empty(), "{once}: {:?}", out2.errors);
        let _ = simplify_unit(&mut out2.ast, &mut out2.unit, &Config::default());
        let twice = print_module(&out2.ast, &out2.unit);
        assert_eq!(once, twice, "decorator output not idempotent:\n{once}\n---\n{twice}");
    }
}

#[test]
fn module_top_level_await_keeps_operator() {
    let src = "await (g());\n";
    let mut out = cure_ts_parser::parse_module(src);
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    let _ = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
    let once = print_module(&out.ast, &out.unit);
    assert!(once.contains("await g()") || once.contains("await (g())"), "{once}");
    let mut out2 = cure_ts_parser::parse_module(&once);
    assert!(out2.errors.is_empty(), "{once}: {:?}", out2.errors);
    let _ = simplify_unit(&mut out2.ast, &mut out2.unit, &Config::default());
    assert_eq!(once, print_module(&out2.ast, &out2.unit));
}

#[test]
fn deep_nesting_falls_back_to_verbatim_source() {
    let src = format!(
        "var x = {}1{};\n",
        "(".repeat(2000),
        ")".repeat(2000)
    );
    let mut out = parse(&src);
    assert!(!out.errors.is_empty(), "depth guard should mark fallback");
    let _ = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
    let printed = print_module(&out.ast, &out.unit);
    assert_eq!(printed, src, "deep input must be preserved verbatim");
    let mut out2 = parse(&printed);
    let _ = simplify_unit(&mut out2.ast, &mut out2.unit, &Config::default());
    assert_eq!(print_module(&out2.ast, &out2.unit), printed);
}

#[test]
fn diff_assign_to_compound() {
    differential(
        "assign_to_compound",
        r#"
let a = 1;
let b = 2;
a = a + b;
let s = "x";
s = s + "y";
let c = 5;
function bump() { c = 10; return 2; }
c = c + bump();
console.log(a, s, c);
"#,
    );
}

#[test]
fn assign_to_compound_is_binding_aware() {
    let src = "let a = 1, b = 2; a = a + b;\nconst o = { x: 1 }; o.x = o.x + 1;\n";
    let mut out = parse(src);
    let _ = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
    let printed = print_module(&out.ast, &out.unit);
    assert!(printed.contains("a += "), "{printed}");
    assert!(printed.contains("o.x = o.x + 1"), "member target must stay: {printed}");
}

// ---------------------------------------------------------------------------
// 随机小程序差分（确定性 PRNG）：原版/cure 版 node 输出比较 + 二次幂等
// ---------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.max(1))
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn gen_expr(rng: &mut Rng, vars: &[String], funcs: &[String], depth: usize) -> String {
    let leaf = depth == 0 || rng.below(3) == 0;
    if leaf {
        if !vars.is_empty() && rng.below(4) != 0 {
            return vars[rng.below(vars.len())].clone();
        }
        return match rng.below(7) {
            0 => "0".into(),
            1 => "1".into(),
            2 => "2".into(),
            3 => "-1".into(),
            4 => format!("{}", rng.below(20) as i64 - 10),
            5 => "true".into(),
            6 => "false".into(),
            _ => format!("'{}'", ["", "a", "0", "x y"][rng.below(4)]),
        };
    }
    match rng.below(8) {
        0 => {
            let op = ["+", "-", "*", "%", "&", "|", "^", "<<", ">>", ">>>"][rng.below(10)];
            format!(
                "({} {} {})",
                gen_expr(rng, vars, funcs, depth - 1),
                op,
                gen_expr(rng, vars, funcs, depth - 1)
            )
        }
        1 => {
            let op = ["===", "!==", "<", ">", "<=", ">="][rng.below(6)];
            format!(
                "({} {} {})",
                gen_expr(rng, vars, funcs, depth - 1),
                op,
                gen_expr(rng, vars, funcs, depth - 1)
            )
        }
        2 => {
            let op = ["&&", "||", "??"][rng.below(3)];
            format!(
                "({} {} {})",
                gen_expr(rng, vars, funcs, depth - 1),
                op,
                gen_expr(rng, vars, funcs, depth - 1)
            )
        }
        3 => {
            let op = ["!", "~", "-", "+", "typeof ", "void "][rng.below(6)];
            format!("({op}{})", gen_expr(rng, vars, funcs, depth - 1))
        }
        4 => format!(
            "({} ? {} : {})",
            gen_expr(rng, vars, funcs, depth - 1),
            gen_expr(rng, vars, funcs, depth - 1),
            gen_expr(rng, vars, funcs, depth - 1)
        ),
        5 if !funcs.is_empty() => format!(
            "{}({}, {})",
            funcs[rng.below(funcs.len())],
            gen_expr(rng, vars, funcs, depth - 1),
            gen_expr(rng, vars, funcs, depth - 1)
        ),
        6 => format!(
            "`{}${{{}}}`",
            ["p", "", "x"][rng.below(3)],
            gen_expr(rng, vars, funcs, depth - 1)
        ),
        _ => format!(
            "(({}, {}))",
            gen_expr(rng, vars, funcs, depth - 1),
            gen_expr(rng, vars, funcs, depth - 1)
        ),
    }
}

fn gen_block(
    rng: &mut Rng,
    vars: &mut Vec<String>,
    assignable: &mut Vec<String>,
    funcs: &mut Vec<String>,
    out: &mut String,
    n: usize,
    indent: &str,
) {
    for _ in 0..n {
        match rng.below(6) {
            0 | 1 => {
                let name = format!("v{}", vars.len() + assignable.len());
                let expr = gen_expr(rng, vars, funcs, 3);
                let kind = ["var", "let", "const"][rng.below(3)];
                out.push_str(&format!("{indent}{kind} {name} = {expr};\n"));
                if kind != "const" {
                    assignable.push(name.clone());
                }
                vars.push(name);
            }
            2 if !assignable.is_empty() => {
                let name = assignable[rng.below(assignable.len())].clone();
                let expr = gen_expr(rng, vars, funcs, 3);
                let op = ["=", "+=", "-=", "^=", "*="][rng.below(5)];
                out.push_str(&format!("{indent}{name} {op} {expr};\n"));
            }
            3 => {
                let expr = gen_expr(rng, vars, funcs, 2);
                let a = gen_expr(rng, vars, funcs, 2);
                let b = gen_expr(rng, vars, funcs, 2);
                out.push_str(&format!("{indent}if ({expr}) {{\n"));
                let mut inner = vars.clone();
                let mut inner_as = assignable.clone();
                gen_block(rng, &mut inner, &mut inner_as, funcs, out, 2, &format!("{indent}  "));
                out.push_str(&format!("{indent}}} else {{\n"));
                let mut inner2 = vars.clone();
                let mut inner2_as = assignable.clone();
                gen_block(rng, &mut inner2, &mut inner2_as, funcs, out, 2, &format!("{indent}  "));
                out.push_str(&format!("{indent}}}\n"));
                // 保留 else 分支里对既有变量的赋值可见性（新声明不外泄）
                let _ = (a, b);
            }
            4 => {
                let bound = 1 + rng.below(3);
                let body = gen_expr(rng, vars, funcs, 2);
                let target = if assignable.is_empty() {
                    "v".to_string()
                } else {
                    assignable[rng.below(assignable.len())].clone()
                };
                if assignable.is_empty() {
                    out.push_str(&format!("{indent}for (let i = 0; i < {bound}; i++) {{ void {body}; }}\n"));
                } else {
                    out.push_str(&format!(
                        "{indent}for (let i = 0; i < {bound}; i++) {{ {target} = {body}; }}\n"
                    ));
                }
            }
            5 => {
                let fname = format!("f{}", funcs.len());
                let p0 = "a";
                let p1 = "b";
                let body = gen_expr(rng, vars, funcs, 2);
                out.push_str(&format!(
                    "{indent}function {fname}({p0}, {p1}) {{ return {body}; }}\n"
                ));
                funcs.push(fname);
            }
            _ => {
                // 内层同名遮蔽
                if let Some(name) = vars.first().cloned() {
                    let expr = gen_expr(rng, vars, funcs, 2);
                    out.push_str(&format!("{indent}{{\n{indent}  let {name} = {expr};\n"));
                    let mut inner = vars.clone();
                    let mut inner_as = assignable.clone();
                    inner_as.retain(|v| v != &name);
                    gen_block(rng, &mut inner, &mut inner_as, funcs, out, 1, &format!("{indent}  "));
                    out.push_str(&format!("{indent}}}\n"));
                }
            }
        }
    }
}

fn gen_program(seed: u64) -> String {
    let mut rng = Rng::new(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15));
    let mut vars = Vec::new();
    let mut assignable = Vec::new();
    let mut funcs = Vec::new();
    let mut out = String::new();
    gen_block(&mut rng, &mut vars, &mut assignable, &mut funcs, &mut out, 6, "");
    let args = if vars.is_empty() { "'empty'".to_string() } else { vars.join(", ") };
    out.push_str(&format!("console.log({args});\n"));
    out
}

#[test]
fn randomized_program_differential() {
    if node_bin().is_none() {
        eprintln!("skip randomized differential: node not found");
        return;
    }
    let cases: u64 = std::env::var("CURE_FUZZ_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80);
    for seed in 1..=cases {
        let src = gen_program(seed);
        eprintln!("--- random case {seed} ---\n{src}");
        differential(&format!("rand_{seed}"), &src);
        // 二次 cure 幂等（同一输入下输出必须逐字节一致）
        let mut out = parse(&src);
        assert!(out.errors.is_empty(), "case {seed} parse: {:?}\n{src}", out.errors);
        let _ = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
        let once = print_module(&out.ast, &out.unit);
        let mut out2 = parse(&once);
        assert!(out2.errors.is_empty(), "case {seed} reparse: {:?}\n{once}", out2.errors);
        let _ = simplify_unit(&mut out2.ast, &mut out2.unit, &Config::default());
        let twice = print_module(&out2.ast, &out2.unit);
        assert_eq!(once, twice, "case {seed} not idempotent:\n{src}\n--- once ---\n{once}\n--- twice ---\n{twice}");
    }
}

#[test]
fn diff_ternary_bool_op_requires_boolean_condition() {
    // c ? true : b ≡ c || b 仅当 c 是布尔；数值 1 时前者 true、后者 1。
    differential(
        "ternary_bool_numeric_cond",
        r#"
let v0 = (true & true);
let v2 = ((v0 ? (false ? 1 : true) : (true < v0)) || v0);
console.log(v0, v2);
"#,
    );
}

#[test]
fn diff_double_neg_only_for_numbers() {
    // -(-true) = 1，不是 true；-(-("1")) = -1，不是 "1"。
    differential(
        "double_neg_non_number",
        r#"
let v0 = (-(-((false, true))));
let v1 = (-(-("1")));
console.log(v0, v1);
"#,
    );
}
