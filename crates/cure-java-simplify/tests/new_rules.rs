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
    // 拆分形态：int x; x = 1; x = 2; → 合并 → 死赋值 → 传播 → return 2;
    assert!(run_src("class A{int m(){int x; x = 1; x = 2; return x;}}").contains("return 2;"));
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

    // toString 结果继续被使用（如 .length()）：SB 折叠 + 字面量长度折叠连锁 → 1
    let out = run_src(r#"
class A {
    int m() {
        return new StringBuilder().append("x").toString().length();
    }
}
"#);
    assert!(out.contains("return 1;"), "{out}");
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
    // if-else 双赋值 → 三元赋值 → r 内联到唯一使用处
    assert!(run_src("class A{int m(boolean c){int r; if (c) {r = 1;} else {r = 2;} return r;}}").contains("return c ? 1 : 2;"));
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

// ---------------------------------------------------------------------------
// 声明-赋值合并 / 赋值传播 / 拼接重结合 / valueOf 剥离
// ---------------------------------------------------------------------------

#[test]
fn decl_assign_merge_rule() {
    // int x; x = 5; return x; → return 5;（合并 + 传播连锁）
    assert!(run_src("class A{int m(){int x; x = 5; return x;}}").contains("return 5;"));
    // 合并 + 传播 + 常量折叠连锁：y = x + 1（x=1 单用途）→ return 2;
    assert!(run_src("class A{int m(){int x; x = 1; int y; y = x + 1; return y;}}").contains("return 2;"));
    // value 引用声明自身（未初始化读）→ 不合并
    let out = run_src("class A{int m(){int y; y = y + 1; return y;}}");
    assert!(out.contains("y = y + 1;"), "{out}");
}

#[test]
fn assign_propagation_rules() {
    // 纯拷贝赋值：x = y; …唯一读 → 内联 y
    let out = run_src("class A{int m(int y){int x = 0; x = y; return x;}}");
    assert!(out.contains("return y;"), "{out}");
    // 有副作用值 + 相邻 VarDecl 使用点（真实 jadx 形态）→ 传播内联到 return
    let out = run_src("class A{int m(){int t = 0; t = foo(); int u = t; return u;}}");
    assert!(out.contains("return foo();"), "{out}");
    assert!(!out.contains("t = foo();"), "{out}");
    assert!(!out.contains("int u"), "{out}");
    // 区间内对 x 再赋值 → 不传播 y；死赋值删除 x=y，z 传播进 return
    let out = run_src("class A{int m(int y, int z){int x = 0; x = y; x = z; return x;}}");
    assert!(out.contains("return z;"), "{out}");
    assert!(!out.contains("x = y;"), "{out}");
    assert!(!out.contains("return y;"), "{out}");
    // value 读到的变量被写 → 不传播
    let out = run_src("class A{int m(int y){int x = 0; x = y; y = 9; return x;}}");
    assert!(out.contains("x = y;") || out.contains("return x;"), "{out}");
    assert!(!out.contains("return y;"), "{out}");
}

#[test]
fn string_concat_reassoc_rules() {
    // "a" + x + "b" + "c" → "a" + x + "bc"
    let out = run_src("class A{String m(String x){return \"a\" + x + \"b\" + \"c\";}}");
    assert!(out.contains(r#"return "a" + x + "bc";"#), "{out}");
    // 三个字面量全折
    let out = run_src("class A{String m(String x){return \"a\" + \"b\" + x;}}");
    assert!(out.contains(r#"return "ab" + x;"#), "{out}");
}

#[test]
fn concat_value_of_drop_rule() {
    let out = run_src("class A{String m(int n){return String.valueOf(n) + \"-\";}}");
    assert!(out.contains("return n + \"-\";"), "{out}");
    // 另一侧不可证 String（数值 + 数值形态不存在 valueOf…这里构造 valueOf+未知变量）
    let out = run_src("class A{String m(int n, Object o){return String.valueOf(n) + o;}}");
    assert!(out.contains("String.valueOf(n)"), "{out}");
}

// ---------------------------------------------------------------------------
// ddc 真实产物规则（A/B/C/D）
// ---------------------------------------------------------------------------

#[test]
fn loop_register_tail_inline() {
    // ddc 循环尾寄存器回拷：int v = x + s; x = v; → x = x + s
    //（使用语句的写发生在求值之后，不算冲突）
    let out = run_src("class A{int m(int x, int s){while (x < 5) { int v = x + s; x = v; } return x;}}");
    assert!(out.contains("x = x + s;"), "{out}");
    // 自赋值场景：self_assign 先删 i = i，传播继续内联 → return 1;
    let out = run_src("class A{int m(){int i = 1; i = i; return i;}}");
    assert!(out.contains("return 1;"), "{out}");
}

#[test]
fn trailing_continue_removed() {
    let out = run_src("class A{int m(java.util.List<String> l){int n = 0; for (String s : l) { n += s.length(); continue; } return n;}}");
    assert!(!out.contains("continue;"), "{out}");
    // 标签指向外层循环的 continue（内层尾部）不删；标签即本循环的可删
    let out = run_src("class A{int m(int a, int b){int n = 0; outer: while (a > 0) { while (b > 0) { n++; continue outer; } a--; } return n;}}");
    assert!(out.contains("continue outer;"), "{out}");
    let out = run_src("class A{int m(){int n = 0; self: while (n < 3) { n++; continue self; } return n;}}");
    assert!(!out.contains("continue"), "{out}");
}

#[test]
fn multi_use_copy_propagation() {
    // v30 = v25; 多处读 → 全部替换为 v25（ddc 寄存器副本）
    let out = run_src("class A{int m(int y){int x = 0; int z = 0; x = y; if (x > 1) { z = x; } return x + z;}}");
    assert!(out.contains("if (y > 1)"), "{out}");
    assert!(out.contains("return y + z;"), "{out}");
    assert!(!out.contains("x = y;"), "{out}");
    // y 被写 → 不传播
    let out = run_src("class A{int m(int y, int w){int x = 0; x = y; y = w; return x;}}");
    assert!(out.contains("return x;"), "{out}");
    assert!(!out.contains("return y;"), "{out}");
}

#[test]
fn string_builder_statement_chain() {
    // ddc 三种形态混合：重赋值 + 新变量 + toString 收尾
    let out = run_src(r#"
class A {
    String m(String x, String y) {
        StringBuilder sb = new StringBuilder().append("h");
        sb = sb.append(x);
        StringBuilder sb2 = sb.append("-");
        sb2 = sb2.append(y);
        String s = sb2.toString();
        return s;
    }
}
"#);
    assert!(out.contains(r#"return "h" + x + "-" + y;"#), "{out}");

    // 链变量有链外使用 → 不折叠
    let out = run_src(r#"
class A {
    String m(String x) {
        StringBuilder sb = new StringBuilder().append("h");
        sb = sb.append(x);
        int len = sb.length();
        String s = sb.toString();
        return s + len;
    }
}
"#);
    assert!(out.contains("new StringBuilder"), "{out}");

    // 空链：new SB().toString() → ""
    let out = run_src("class A{String m(){StringBuilder sb = new StringBuilder(); String s = sb.toString(); return s;}}");
    assert!(out.contains("return \"\";"), "{out}");
}

// ---------------------------------------------------------------------------
// DAD/ASC 伪影规则
// ---------------------------------------------------------------------------

#[test]
fn dad_artifact_class_name_and_trailing_return() {
    // class LDemo; {（Dalvik 描述符泄漏）→ class Demo，构造器可正常解析
    let out = run_src("public class LDemo; {\n    public Demo(int p1) {\n        this.base = p1;\n        return;\n    }\n}\nclass Aux {\n    int x;\n}");
    assert!(out.contains("public class Demo {"), "{out}");
    assert!(out.contains("this.base = p1;"), "{out}");
    assert!(!out.contains("return;"), "{out}"); // 尾部裸 return 删除
}

#[test]
fn trailing_return_rule() {
    // void 方法尾部裸 return → 删除
    let out = run_src("class A{void m(){foo(); return;}}");
    assert!(out.contains("foo();"), "{out}");
    assert!(!out.contains("return;"), "{out}");
    // 带值 return 保留
    let out = run_src("class A{int m(){return 1;}}");
    assert!(out.contains("return 1;"), "{out}");
    // 块中间的提前 return 保留
    let out = run_src("class A{void m(int x){if (x > 0) {return;} foo(); return;}}");
    assert!(out.contains("if (x > 0) {"), "{out}");
    let cnt = out.matches("return;").count();
    assert_eq!(cnt, 1, "{out}"); // 只剩 if 里的那个
}

#[test]
fn iterator_with_cast_and_paren() {
    // DAD：String s = (String) it.next(); → for-each 还原仍工作
    let out = run_src(r#"
class A {
    void m(java.util.List<String> list) {
        java.util.Iterator it = list.iterator();
        while (it.hasNext()) {
            String s = ((String) it.next());
            System.out.println(s);
        }
    }
}
"#);
    assert!(out.contains("for (String s : list)"), "{out}");
}

// ---------------------------------------------------------------------------
// StoreKill（远距死存储/寄存器预声明清理）
// ---------------------------------------------------------------------------

#[test]
fn store_kill_rules() {
    // 字面量提升 + 传播接力：int v = 0; foo(); v = 1; return v; → foo(); return 1;
    let out = run_src("class A{int m(){int v = 0; foo(); v = 1; return v;}}");
    assert!(out.contains("foo();"), "{out}");
    assert!(out.contains("return 1;"), "{out}");
    assert!(!out.contains("int v"), "{out}");
    // 剥除 init + 传播接力：int v = 0; foo(); v = y; return v; → foo(); return y;
    let out = run_src("class A{int m(int y){int v = 0; foo(); v = y; return v;}}");
    assert!(out.contains("foo();"), "{out}");
    assert!(out.contains("return y;"), "{out}");
    // 赋值形态远距死存储 + 传播接力：v = 0; …; v = y; return v; → return y;
    let out = run_src("class A{int m(int y){int v = 0; v = 0; foo(); v = y; return v;}}");
    assert!(!out.contains("v = 0;"), "{out}");
    assert!(out.contains("return y;"), "{out}");
}

#[test]
fn store_kill_safety_cases() {
    // 副作用 init：int v = bump(); v = 5; → 不动（调用不能丢）
    let out = run_src("class A{int m(){int v = bump(); v = 5; return v;}}");
    assert!(out.contains("int v = bump();"), "{out}");
    // 击杀写在条件分支内 → 不动（definite assignment + 分支可能不执行）
    let out = run_src("class A{int m(int c){int v = 0; if (c > 0) { v = 1; } return v;}}");
    assert!(out.contains("int v = 0;"), "{out}");
    // 击杀写在循环体内 → 不动（可能零次执行）
    let out = run_src("class A{int m(){int v = 0; while (v < 3) { v = 9; } return v;}}");
    assert!(out.contains("int v = 0;") || out.contains("int v = 0;\n"), "{out}");
    // 中间语句的 init 里读 v → 不动（读事件阻断）
    let out = run_src("class A{String m(){String v = \"\"; String w = v + \"x\"; v = \"y\"; return w;}}");
    assert!(out.contains("v = \"y\";"), "{out}");
}

// ---------------------------------------------------------------------------
// CFF 嵌套条件（分支到达同一后续条件 case）
// ---------------------------------------------------------------------------

#[test]
fn cff_nested_conditional() {
    let src = r#"
public class NestC {
    public static void main(String[] args) {
        int x = 8;
        String r = null;
        int s = 0;
        while (true) {
            switch (s) {
                case 0: {
                    if (x > 0) {
                        s = 1;
                    } else {
                        s = 5;
                    }
                    break;
                }
                case 1: {
                    if (x > 10) {
                        s = 2;
                    } else {
                        s = 3;
                    }
                    break;
                }
                case 2: {
                    r = "big";
                    s = 4;
                    break;
                }
                case 3: {
                    r = "mid";
                    s = 4;
                    break;
                }
                case 5: {
                    r = "neg";
                    s = 4;
                    break;
                }
            }
            if (s == 4) {
                break;
            }
        }
        System.out.println("r=" + r);
    }
}
"#;
    let mut outcome = parse(src);
    assert!(outcome.errors.is_empty());
    let report = cure_java_simplify::simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let cleaned = cure_java_print::print_unit(&outcome.ast, &outcome.unit);
    assert!(report.by_rule.contains_key("cff_recover"), "CFF 未触发：\n{cleaned}");
    assert!(!cleaned.contains("switch"), "{cleaned}");
    assert!(cleaned.contains("x > 0"), "{cleaned}");
    assert!(cleaned.contains("x > 10"), "{cleaned}");
    // 嵌套条件还原为 if/else 链后，三元归并接力折成嵌套三元
    assert!(
        cleaned.contains(r#"x > 0 ? x > 10 ? "big" : "mid" : "neg""#),
        "{cleaned}"
    );
}
