//! 新增规则 golden 测试：短路/常量折叠/三元/零元素/死赋值/StringBuilder/装箱链/迭代器还原。

use cure_engine::kind::{BinOp, UnOp};
use cure_engine::{Config, Lang};
use cure_java_ast::{JType, JavaAst, JavaId, Lit};
use cure_java_parser::parse;
use cure_java_print::print;
use cure_java_simplify::simplify;

/// 直接从源码走全链路（解析→简化→打印），最接近真实使用。
fn run_src(src: &str) -> String {
    let mut out = parse(src);
    cure_java_simplify::simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
    cure_java_print::print_unit(&out.ast, &out.unit)
}

fn run(build: impl FnOnce(&mut JavaAst) -> JavaId) -> String {
    let mut ast = JavaAst::new();
    let root = build(&mut ast);
    simplify(&mut ast, root, &Config::default());
    print(&ast, root)
}

// ---- 引擎新规则（toy 已覆盖部分，这里验证 Java 形态）----

#[test]
fn short_circuit_folds() {
    assert_eq!(
        run_src("class A{boolean m(boolean x){return false && x;}}").trim_end(),
        "class A {\n    boolean m(boolean x) {\n        return false;\n    }\n}"
    );
    assert!(run_src("class A{boolean m(boolean x){return true || x;}}").contains("return true;"));
    assert!(run_src("class A{boolean m(boolean x){return x && true;}}").contains("return x;"));
    assert!(run_src("class A{boolean m(boolean x){return x || false;}}").contains("return x;"));
    // x 有副作用 → 不折叠
    let out = run_src("class A{boolean m(){return foo() && false;}}");
    assert!(out.contains("foo() && false"), "{out}");
    // 字面量在左：短路语义恒安全
    assert!(run_src("class A{boolean m(boolean x){return false || x;}}").contains("return x;"));
}

#[test]
fn const_fold() {
    assert!(run_src("class A{int m(){return 1 + 2;}}").contains("return 3;"));
    assert!(run_src("class A{int m(){return 2 * 3;}}").contains("return 6;"));
    assert!(run_src("class A{int m(){return 10 - 4;}}").contains("return 6;"));
    assert!(run_src("class A{int m(){return 1 << 3;}}").contains("return 8;"));
    assert!(run_src("class A{int m(){return 6 & 3;}}").contains("return 2;"));
    // 位移距离掩码（Java 语义：<<32 == <<0）
    assert!(run_src("class A{int m(){return 1 << 32;}}").contains("return 1;"));
    // Int 溢出回绕（与 javac 常量折叠一致）
    assert!(run_src("class A{int m(){return 2147483647 + 1;}}").contains("return -2147483648;"));
    // long 传播
    assert!(run_src("class A{long m(){return 2L * 3L;}}").contains("return 6L;"));
    // 除零不折叠
    let out = run_src("class A{int m(){return 1 / 0;}}");
    assert!(out.contains("1 / 0"), "{out}");
    // 字符串拼接
    assert!(run_src("class A{String m(){return \"ab\" + \"cd\";}}").contains(r#"return "abcd";"#));
    // 字面量原文保真（十六进制不折叠掉进制）
    let out = run_src("class A{int m(){return 0x1F;}}");
    assert!(out.contains("0x1F"), "{out}");
}

#[test]
fn not_compare() {
    assert!(run_src("class A{boolean m(int a, int b){return !(a == b);}}").contains("return a != b;"));
    assert!(run_src("class A{boolean m(int a, int b){return !(a < b);}}").contains("return a >= b;"));
    assert!(run_src("class A{boolean m(int a, int b){return !(a <= b);}}").contains("return a > b;"));
    // 浮点不做次序取反（NaN 语义）
    let out = run_src("class A{boolean m(double a, double b){return !(a < b);}}");
    assert!(out.contains("!(a < b)"), "{out}");
    // 引用/对象相等取反恒安全
    assert!(run_src("class A{boolean m(Object a, Object b){return !(a == b);}}").contains("return a != b;"));
}

#[test]
fn ternary_folds() {
    assert!(run_src("class A{int m(int x){return true ? 1 : x;}}").contains("return 1;"));
    assert!(run_src("class A{int m(int x){return false ? x : 2;}}").contains("return 2;"));
    assert!(run_src("class A{int m(boolean c, int x){return c ? x : x;}}").contains("return x;"));
    // c 有副作用 → 不折叠
    let out = run_src("class A{int m(){return call() ? 1 : 1;}}");
    assert!(out.contains("call() ? 1 : 1"), "{out}");
    assert!(run_src("class A{boolean m(boolean c){return c ? true : false;}}").contains("return c;"));
    assert!(run_src("class A{boolean m(boolean c){return c ? false : true;}}").contains("return !c;"));
}

#[test]
fn arith_zero() {
    // 0 * x → 0（x 纯）
    assert!(run_src("class A{int m(int x){return 0 * x;}}").contains("return 0;"));
    assert!(run_src("class A{int m(int x){return x * 0;}}").contains("return 0;"));
    // x 有副作用不折叠
    let out = run_src("class A{int m(){return 0 * foo();}}");
    assert!(out.contains("0 * foo()"), "{out}");
    // 浮点不折叠（0.0 * NaN = NaN）
    let out = run_src("class A{double m(double x){return 0 * x;}}");
    assert!(out.contains("0 * x"), "{out}");
    // x - x → 0
    assert!(run_src("class A{int m(int x){return x - x;}}").contains("return 0;"));
    // 移位 0 恒等
    assert!(run_src("class A{int m(int x){return x << 0;}}").contains("return x;"));
    assert!(run_src("class A{int m(int x){return x >> 0;}}").contains("return x;"));
}

#[test]
fn dead_store() {
    // int x = 1; x = 2; → int x = 2; → 传播继续 → return 2;
    assert!(run_src("class A{int m(){int x = 1; x = 2; return x;}}").contains("return 2;"));
    let out = run_src("class A{int m(){int x; x = 1; x = 2; return x;}}");
    assert!(out.contains("x = 2;"), "{out}");
    assert!(!out.contains("x = 1;"), "{out}");
    // a 有副作用 → 保留
    let out = run_src("class A{int m(){int x; x = foo(); x = 2; return x;}}");
    assert!(out.contains("x = foo();"), "{out}");
    // b 引用 x → 不可删
    let out = run_src("class A{int m(){int x = 1; x = x + 2; return x;}}");
    assert!(out.contains("x = x + 2;"), "{out}");
}

// ---- Java 特有新规则 ----

#[test]
fn string_builder_fold() {
    let out = run_src(r#"
class A {
    String m(int a, String b) {
        return new StringBuilder().append(a).append(b).toString();
    }
}
"#);
    assert!(out.contains("return \"\" + a + b;"), "{out}");

    // 首参为 String → 无需 ""
    let out = run_src(r#"
class A {
    String m(String a, String b) {
        return new StringBuilder(a).append(b).toString();
    }
}
"#);
    assert!(out.contains("return a + b;"), "{out}");

    // 全字面量
    let out = run_src(r#"
class A {
    String m() {
        return new StringBuilder().append("x").append("y").toString();
    }
}
"#);
    assert!(out.contains(r#"return "xy";"#), "{out}");

    // 容量构造不折叠
    let out = run_src(r#"
class A {
    StringBuilder m() {
        return new StringBuilder(16);
    }
}
"#);
    assert!(out.contains("new StringBuilder(16)"), "{out}");

    // toString 结果继续被使用（如 .length()）：折叠仍语义安全（String 值等价）
    let out = run_src(r#"
class A {
    int m() {
        return new StringBuilder().append("x").toString().length();
    }
}
"#);
    assert!(out.contains(r#""x".length()"#), "{out}");
}

#[test]
fn box_unbox_chain() {
    let out = run_src(r#"
class A {
    int m() {
        return Integer.valueOf(42).intValue();
    }
}
"#);
    assert!(out.contains("return 42;"), "{out}");

    let out = run_src(r#"
class A {
    boolean m(boolean x) {
        return Boolean.valueOf(x).booleanValue();
    }
}
"#);
    assert!(out.contains("return x;"), "{out}");

    // valueOf(String) 是解析 → 不折叠
    let out = run_src(r#"
class A {
    int m() {
        return Integer.valueOf("42").intValue();
    }
}
"#);
    assert!(out.contains("Integer.valueOf(\"42\")"), "{out}");
}

#[test]
fn iterator_to_for_each() {
    let out = run_src(r#"
class A {
    int m(java.util.List<String> list) {
        int n = 0;
        for (java.util.Iterator<String> it = list.iterator(); it.hasNext(); ) {
            String s = it.next();
            n += s.length();
        }
        return n;
    }
}
"#);
    assert!(out.contains("for (String s : list) {"), "{out}");
    assert!(!out.contains("Iterator"), "{out}");

    // it 在后续被引用 → 不还原
    let out = run_src(r#"
class A {
    int m(java.util.List<String> list) {
        int n = 0;
        for (java.util.Iterator<String> it = list.iterator(); it.hasNext(); ) {
            String s = it.next();
            n += s.length() + it.hashCode();
        }
        return n;
    }
}
"#);
    assert!(out.contains("it.hashCode()"), "{out}");
}

#[test]
fn unreachable_opt_in() {
    // 默认不删（DCE 选配）
    let out = run_src("class A{int m(){return 1; call();}}");
    assert!(out.contains("call();"), "{out}");

    // all_java_rules 启用后删除
    let src = "class A{int m(){return 1; call();}}";
    let mut o = parse(src);
    let rules = cure_java_simplify::all_java_rules();
    if let cure_java_ast::Member::Method { body: Some(b), .. } =
        &mut o.unit.types[0].members[0].clone()
    {
        let report = cure_engine::simplify(&mut o.ast, *b, &rules, &Config::default());
        assert!(report.edits >= 1);
        let printed = cure_java_print::print(&o.ast, *b);
        assert!(printed.contains("return 1;"));
        assert!(!printed.contains("call();"), "{printed}");
    }
}

// ---------------------------------------------------------------------------
// 反混淆新规则
// ---------------------------------------------------------------------------

#[test]
fn cmp_const_fold() {
    assert!(run_src("class A{boolean m(){return 1 < 2;}}").contains("return true;"));
    assert!(run_src("class A{boolean m(){return 2 == 3;}}").contains("return false;"));
    assert!(run_src("class A{boolean m(){return 5L >= 5L;}}").contains("return true;"));
    // 浮点不折叠
    let out = run_src("class A{boolean m(){return 1.0 < 2.0;}}");
    assert!(out.contains("1.0 < 2.0"), "{out}");
    // 不透明谓词整链击穿：boolean always = 2 > 1; if (always) {...} else {junk}
    let out = run_src(r#"
class A {
    String m(int x) {
        boolean always = 2 > 1;
        if (always) {
            return "live";
        } else {
            return junk();
        }
    }
}
"#);
    assert!(out.contains("return \"live\";"), "{out}");
    assert!(!out.contains("junk()"), "{out}");
    assert!(!out.contains("always"), "{out}");
}

#[test]
fn bit_identity_rules() {
    assert!(run_src("class A{int m(int x){return x ^ 0;}}").contains("return x;"));
    assert!(run_src("class A{int m(int x){return 0 | x;}}").contains("return x;"));
    assert!(run_src("class A{int m(int x){return x & -1;}}").contains("return x;"));
    assert!(run_src("class A{int m(int x){return x | -1;}}").contains("return -1;"));
    assert!(run_src("class A{int m(int x){return x & 0;}}").contains("return 0;"));
    assert!(run_src("class A{int m(int x){return x ^ x;}}").contains("return 0;"));
    // x 有副作用不折
    let out = run_src("class A{int m(){return foo() & 0;}}");
    assert!(out.contains("& 0"), "{out}");
}

#[test]
fn arith_reassoc_rules() {
    // (x ^ 84) ^ 84 → x（混淆器经典双异或）
    assert!(run_src("class A{int m(int x){return (x ^ 84) ^ 84;}}").contains("return x;"));
    assert!(run_src("class A{int m(int x){return (x + 5) - 5;}}").contains("return x;"));
    assert!(run_src("class A{int m(int x){return (x - 3) - 2;}}").contains("return x - 5;"));
    assert!(run_src("class A{int m(int x){return (x + 1) + 2;}}").contains("return x + 3;"));
    let out = run_src("class A{long m(long x){return (x + 5L) - 5L;}}");
    assert!(out.contains("return x;"), "{out}");
}

#[test]
fn if_to_ternary_rules() {
    // if-else 双 return → 三元
    assert!(run_src("class A{int m(boolean c){if (c) {return 1;} else {return 2;}}}").contains("return c ? 1 : 2;"));
    // if-then + 收尾 return → 三元
    assert!(run_src("class A{int m(boolean c){if (c) {return 1;} return 2;}}").contains("return c ? 1 : 2;"));
    // 布尔特例 → 直接 return c
    assert!(run_src("class A{boolean m(boolean c){if (c) {return true;} return false;}}").contains("return c;"));
    // if-else 双赋值 → 三元赋值
    assert!(run_src("class A{int m(boolean c){int r; if (c) {r = 1;} else {r = 2;} return r;}}").contains("r = c ? 1 : 2;"));
    // 副作用条件照常保留
    let out = run_src("class A{int m(){if (check()) {return 1;} return 2;}}");
    assert!(out.contains("check() ? 1 : 2"), "{out}");
}

#[test]
fn new_string_fold_rule() {
    assert!(run_src("class A{String m(){return new String(\"lit\");}}").contains(r#"return "lit";"#));
    // 非 String() 形态不折
    let out = run_src("class A{String m(byte[] b){return new String(b);}}");
    assert!(out.contains("new String(b)"), "{out}");
}

// ---------------------------------------------------------------------------
// 真实反编译形态规则（jadx/jcdc 产物）
// ---------------------------------------------------------------------------

#[test]
fn loop_head_break_rules() {
    // while(true){if(c)break;REST} → while(!c){REST}（取反由 not_compare 继续折叠）
    let out = run_src("class A{int m(int n){int s=0; while (true) { if (n >= 5) { break; } s += n; n++; } return s;}}");
    assert!(out.contains("while (n < 5)"), "{out}");
    // else 形态
    let out = run_src("class A{int m(int n){int s=0; while (true) { if (n >= 5) break; else { s += n; } n++; } return s;}}");
    assert!(out.contains("while (n < 5)"), "{out}");
    // 带 continue 的循环体
    let out = run_src("class A{int m(int n){int s=0; while (true) { if (n > 100) { break; } if (n % 2 == 0) { n++; continue; } s += n; n++; } return s;}}");
    assert!(out.contains("while (n <= 100)"), "{out}");
    assert!(out.contains("continue;"), "{out}");
    // 有标签 break（目标是外层）→ 不动
    let out = run_src("class A{int m(){int s=0; outer: while (true) { while (true) { if (ok()) { break outer; } s++; } } }}");
    assert!(out.contains("break outer;"), "{out}");
}

#[test]
fn while_iterator_to_for_each_rules() {
    let out = run_src(r#"
class A {
    int m(java.util.List<String> list) {
        int n = 0;
        java.util.Iterator<String> it = list.iterator();
        while (it.hasNext()) {
            String s = it.next();
            n += s.length();
        }
        return n;
    }
}
"#);
    assert!(out.contains("for (String s : list)"), "{out}");
    assert!(!out.contains("Iterator"), "{out}");
    // it 在后续被引用 → 不动
    let out = run_src(r#"
class A {
    int m(java.util.List<String> list) {
        int n = 0;
        java.util.Iterator<String> it = list.iterator();
        while (it.hasNext()) {
            String s = it.next();
            n += s.length() + it.hashCode();
        }
        return n;
    }
}
"#);
    assert!(out.contains("it.hashCode()"), "{out}");
}
