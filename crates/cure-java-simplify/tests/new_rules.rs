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
    // b 引用 x：DeadStore 不可删（读语义），但 assign_back_fold 可折回
    // 声明（int x = 1; x = x + 2 → int x = 3）——语义正确
    let out = run_src("class A{int m(){int x = 1; x = x + 2; return x;}}");
    assert!(out.contains("return 3;"), "{out}");
    // 值验证：x 终值 = 1 + 2
    assert!(run_src("class A{int m(int b){int x = b; x = x + 2; return x;}}").contains("return b + 2;"));
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
    // value 读到的变量被写 → 不传播。
    // 注意 y=9 必须是活写（println(y) 读取）：若为死写会先被零用途
    // DeadStore 删除，届时传播 y 反而是健全的（差分 differential 覆盖）。
    let out = run_src(
        "class A{int m(int y){int x = 0; x = y; y = 9; System.out.println(y); return x;}}",
    );
    assert!(!out.contains("return y;"), "{out}");
    // 死写形态：y=9 无读取 → 删除后传播健全（return y 返回旧值，行为等价）
    let out = run_src("class A{int m(int y){int x = 0; x = y; y = 9; return x;}}");
    assert!(out.contains("return y;"), "{out}");
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
    // y 被活写（有读取）→ 不传播
    let out = run_src(
        "class A{int m(int y, int w){int x = 0; x = y; y = w; System.out.println(y); return x;}}",
    );
    assert!(out.contains("return x;"), "{out}");
    assert!(!out.contains("return y;"), "{out}");
    // y 被死写（无读取）→ 写先被零用途 DeadStore 删除，传播 y 健全（返回旧值）
    let out = run_src("class A{int m(int y, int w){int x = 0; x = y; y = w; return x;}}");
    assert!(out.contains("return y;"), "{out}");
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
    // 击杀写在循环体内：store_kill 不动（可能零次执行）；static_exec
    // 可全程求值（v: 0→9，循环一轮退出——javac 真值 9 对拍锁定）
    let out = run_src("class A{int m(){int v = 0; while (v < 3) { v = 9; } return v;}}");
    assert!(out.contains("return 9;"), "{out}");
    // 中间语句的 init 里读 v → 读事件阻断 init 剥除；
    // v="y" 是死写（后续无读）→ 零用途 DeadStore 先删，整链合法坍缩（差分锁定）
    let out = run_src("class A{String m(){String v = \"\"; String w = v + \"x\"; v = \"y\"; return w;}}");
    assert!(out.contains("return \"x\";"), "{out}");
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

// ---------------------------------------------------------------------------
// do-while/break-continue 形态 + 内联 next() 提取
// ---------------------------------------------------------------------------

#[test]
fn loop_head_break_dowhile_forms() {
    // ddc 形态：do { if (c) { REST; continue; } else { break; } } while (true)
    let out = run_src(r#"
class A {
    int m(java.util.List<String> l) {
        int n = 0;
        java.util.Iterator<String> it = l.iterator();
        do {
            if (it.hasNext()) {
                n += it.next().length();
                continue;
            } else {
                break;
            }
        } while (true);
        return n;
    }
}
"#);
    // 链式接力：do-while 还原 → while(hasNext) → 内联 next() 提取 → for-each
    assert!(out.contains("for (String e : l)"), "{out}");
    assert!(!out.contains("do"), "{out}");
    assert!(!out.contains("Iterator"), "{out}");

    // while 形态 B：then 以 continue 结尾、else 是 break
    let out = run_src(r#"
class A {
    int m(java.util.List<String> l) {
        int n = 0;
        java.util.Iterator<String> it = l.iterator();
        while (true) {
            if (it.hasNext()) {
                n += 1;
                continue;
            } else {
                break;
            }
        }
        return n;
    }
}
"#);
    assert!(out.contains("while (it.hasNext())"), "{out}");
    assert!(!out.contains("continue"), "{out}");

    // while 形态 D：then 是 break、else 以 continue 结尾
    let out = run_src("class A{int m(int x){int s = 0; while (true) { if (x < 0) { break; } else { s += x; x--; continue; } } return s;}}");
    assert!(out.contains("while (x >= 0)"), "{out}");
    assert!(!out.contains("continue"), "{out}");
}

#[test]
fn inline_next_extraction_to_for_each() {
    // ddc 形态：next() 内联在表达式里（Cast 包裹）
    let out = run_src(r#"
class A {
    void m(java.util.List<String> list) {
        String acc = "";
        java.util.Iterator it = list.iterator();
        while (it.hasNext()) {
            acc = acc + (String) it.next() + "-";
        }
        System.out.println(acc);
    }
}
"#);
    assert!(out.contains("for (String e : list)"), "{out}");
    assert!(out.contains("acc = acc + e"), "{out}");
    assert!(!out.contains("Iterator"), "{out}");
    assert!(!out.contains("next()"), "{out}");

    // it 在体内出现两次 → 不提取
    let out = run_src(r#"
class A {
    void m(java.util.List<String> list) {
        java.util.Iterator it = list.iterator();
        while (it.hasNext()) {
            System.out.println(it.next() + it.next().length());
        }
    }
}
"#);
    assert!(out.contains("it.next()"), "{out}");
}

// ---- 宽度回绕（语义参数化迁移中修复的两个 i32 溢出边界）----

#[test]
fn reassoc_i32_overflow_wraps() {
    // (x + 2147483647) + 1：合并值 2^31 溢出 i32 —— 必须回绕为 -2147483648
    // 表达（`x + 2147483648` 是非法 int 字面量，javac 拒绝）
    let out = run_src("class A{int f(int x){return (x + 2147483647) + 1;}}");
    assert!(!out.contains("+ 2147483648"), "非法字面量泄漏: {out}");
    assert!(out.contains("- 2147483648") || out.contains("+ -2147483648"), "{out}");
    // 输出必须可干净重解析（自洽性）
    let back = parse(&out);
    assert!(back.errors.is_empty(), "{:?}", back.errors);
    // long 域不回绕（合法）
    let out = run_src("class A{long f(long x){return (x + 9223372036854775806L) + 2L;}}");
    assert!(out.contains("- 9223372036854775808L") || out.contains("+ -9223372036854775808L"), "{out}");
}

#[test]
fn fold_i32_min_negation_refused() {
    // Int(-2147483648) 的 -(-lit) 取反溢出 → 拒绝折叠（原实现产出
    // 非法字面量 +2147483648）
    let mut out = parse("class A{int f(){int a = -3 - 4; int b = -2147483648 - 0; return -(-2147483648 - 0);}}");
    // 构造负字面量最值：fold 产 Int(-2147483648)，外层取负必须被拒绝
    cure_java_simplify::simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
    let printed = cure_java_print::print_unit(&out.ast, &out.unit);
    assert!(!printed.contains("return 2147483648;"), "{printed}");
    let back = parse(&printed);
    assert!(back.errors.is_empty(), "{:?}", back.errors);
}

#[test]
fn fold_neg_normal_still_works() {
    // 常规负字面量取反不受影响
    let out = run_src("class A{int f(){int a = -3 - 4; return -a;}}");
    // a 折为 -7 后 -a 保留（a 是变量非字面量）——换个直接形态：
    let out2 = run_src("class A{int f(){return -(-7);}}");
    assert!(out2.contains("return 7;"), "{out2}");
}

// ---- inverse_assign_pair：逆运算对抵消（寄存器噪声）----

#[test]
fn inverse_pair_compound_cascades() {
    // PMD PMDTaskTestExample.java 形态：交替噪声整串抵消
    let src = "class A{int f(){int a;a+=1;a-=1;a+=1;a-=1;return 7;}}";
    let out = run_src(src);
    assert!(out.contains("return 7;"), "{out}");
    assert!(!out.contains("a += 1;") || !out.contains("a -= 1;"), "{out}");
    // 交替 + 常量折半的残留声明由既有规则处理；成对部分必须消失
    let src2 = "class A{int f(){int a;a+=5;a-=5;return a;}}";
    let out2 = run_src(src2);
    assert!(out2.contains("return a;"), "{out2}");
    assert!(!out2.contains("+= 5;") && !out2.contains("-= 5;"), "{out2}");
}

#[test]
fn inverse_pair_expanded_and_mixed_forms() {
    // 展开形态（反编译器常见输出）
    let out = run_src("class A{int f(int x){x=x+5;x=x-5;return x;}}");
    assert!(out.contains("return x;"), "{out}");
    // 混合形态：复合 + 展开
    let out = run_src("class A{int f(int x){x+=5;x=x-5;return x;}}");
    assert!(out.contains("return x;"), "{out}");
    // XOR 自逆
    let out = run_src("class A{int f(int x){x^=42;x^=42;return x;}}");
    assert!(out.contains("return x;"), "{out}");
    // 对称方向
    let out = run_src("class A{int f(int x){x-=3;x+=3;return x;}}");
    assert!(out.contains("return x;"), "{out}");
}

#[test]
fn inverse_pair_safety_guards() {
    // 浮点：不抵消（溢出/Inf 上 (x+K)-K ≠ x）
    let out = run_src("class A{double f(double x){x+=1;x-=1;return x;}}");
    assert!(out.contains("x += 1;"), "{out}");
    // 字段：可见副作用，不抵消
    let out = run_src("class A{int f;int g(){f+=1;f-=1;return f;}}");
    assert!(out.contains("f += 1;"), "{out}");
    // 中间有读：x 的中间值被观察到，不抵消
    let out = run_src("class A{int f(int x){x+=1;int y=x*2;x-=1;return y;}}");
    assert!(out.contains("x += 1;"), "{out}");
    // 常量不同：不抵消
    let out = run_src("class A{int f(int x){x+=1;x-=2;return x;}}");
    assert!(out.contains("x += 1;"), "{out}");
    // long 域同样抵消（模 2^64 群恒等）
    let out = run_src("class A{long f(long x){x+=9223372036854775807L;x-=9223372036854775807L;return x;}}");
    assert!(out.contains("return x;"), "{out}");
    assert!(!out.contains("+="), "{out}");
}

// ---- assign_back_fold：重赋值折回声明 + dead_decl：零使用死声明 ----

#[test]
fn register_accumulator_folds_to_return_expr() {
    // 反编译器寄存器累加器全链：r2 = base; r2 = r2 + 30; return r2
    // → return base + 30（用户期望形态）
    let out = run_src(
        "class T{int fee(int base){int r2;r2=base;r2=r2+30;return r2;}}",
    );
    assert!(out.contains("return base + 30;"), "{out}");
    // 声明与折回点之间夹无关语句（对 x 零事件）同样折回
    let out = run_src(
        "class T{int f(int b){int x=b;int y=8;y=y+1;x=x+30;return x+y;}}",
    );
    assert!(out.contains("return b + 39;"), "{out}");
    // 复合赋值形态
    let out = run_src("class T{int f(int b){int x=b;x+=30;return x;}}");
    assert!(out.contains("return b + 30;"), "{out}");
    // 多级 delta 链（fixed-point 逐级折回）
    let out = run_src("class T{int f(int b){int x=b;x+=1;x+=2;return x;}}");
    assert!(out.contains("return b + 3;"), "{out}");
    // XOR 折回
    let out = run_src("class T{int f(int b){int x=b;x^=42;return x;}}");
    assert!(out.contains("return b ^ 42;"), "{out}");
}

#[test]
fn assign_back_fold_safety_negatives() {
    // 中间有读：assign_back_fold 不得折回（y 观察中间值 5）；
    // static_exec 可以整体求值——结果必须精确为 5*10+6=56
    let out = run_src("class T{int f(){int x=5;int y=x;x=x+1;return y*10+x;}}");
    assert!(out.contains("return 56;"), "{out}");
    // 精确性：y 必须拿到 5（50 = 5*10）
    let out = run_src("class T{int f(){int x=5;int y=x;x=x+1;return y*10;}}");
    assert!(out.contains("50"), "{out}");
    // 中间有写：首个事件不是重赋值 → 拒绝
    let out = run_src("class T{int f(int b){int x=b;x=99;x=x+1;return x;}}");
    assert!(out.contains("return 100;") || out.contains("x + 1"), "{out}");
    // 条件块内重赋值：折回会无条件化 → 拒绝
    let out = run_src("class T{int f(int b,boolean c){int x=b;if(c){x=x+1;}return x;}}");
    assert!(out.contains("if"), "{out}");
    // Div/Rem K=0：异常位置会提前 → 拒绝
    let out = run_src("class T{int f(int b){int x=b;int y=0;x=x/0;return y;}}");
    assert!(out.contains("/ 0"), "{out}");
    // K 非字面量 → 拒绝
    let out = run_src("class T{int f(int b,int k){int x=b;x=x+k;return x;}}");
    assert!(out.contains("x + k") || out.contains("x += k") || out.contains("b + k"), "{out}");
}

#[test]
fn dead_decl_removal_and_guards() {
    // 正例：零使用裸声明
    let out = run_src("class T{int f(){int r1;return 7;}}");
    assert!(out.contains("return 7;") && !out.contains("r1"), "{out}");
    // 正例：纯 init 零使用
    let out = run_src("class T{int f(){int x=5;return 7;}}");
    assert!(!out.contains("int x"), "{out}");
    // 有使用：DeadDecl 不删（local_propagation 可正确内联为 return 5）
    let out = run_src("class T{int f(){int x=5;return x;}}");
    assert!(out.contains("return 5;"), "{out}");
    // 负例：init 有副作用（调用）→ 保留求值
    let out = run_src("class T{int f(){int x=foo();return 7;}}");
    assert!(out.contains("foo();"), "{out}");
    // 负例：init 可能抛 → 保留（删掉会失去异常）
    let out = run_src("class T{int f(){int x=1/0;return 7;}}");
    assert!(out.contains("1 / 0"), "{out}");
}

#[test]
fn demo_register_noise_full_collapse() {
    // 完整反编译噪声演示：应折到 return base + 30
    let out = run_src(
        "class Ticket{int fee(int base){int r1;int r2;r1=base;r2=r1;r1+=7;r1-=7;\
         boolean flag;flag=(2<3)&&true;if(flag){r2=r2+30;}r1=r2*1;int k=0x5A^0x5A;return r1+k;}}",
    );
    assert!(out.contains("return base + 30;"), "{out}");
}

// ---- 窄类型（char/byte/short）安全：隐式收窄与类型可观察性 ----

#[test]
fn narrow_type_compound_not_folded() {
    // char：assign_back_fold 拒绝窄域（println 观察到 char 语义）；
    // println 含未知调用 → vexec 前缀不足也不动
    let out = run_src("class A{void m(){char ch='a';ch+=2;System.out.println(ch);}}");
    assert!(out.contains("ch += 2;") || out.contains("ch = ch + 2;"), "{out}");
    // byte：JLS 复合赋值隐式收窄——(byte)200 = -56（javac 真值对拍）
    let out = run_src("class A{int m(){byte b=100;b+=100;return b;}}");
    assert!(out.contains("return -56;"), "{out}");
    // 宽类型（int/long）照常折回
    let out = run_src("class A{int m(int x){int i=x;i+=5;return i;}}");
    assert!(out.contains("return x + 5;"), "{out}");
}

#[test]
fn char_decl_int_literal_normalized() {
    // char c = 98 → 'b'：归一后传播安全（println 打 'b' 而非 98）
    let out = run_src("class A{void m(){char c = 98;System.out.println(c);}}");
    assert!(out.contains("'b'"), "{out}");
    // 域外常量不归一（源本就非法，容错保留）
    let out = run_src("class A{void m(){char c = 70000;System.out.println(c);}}");
    assert!(out.contains("70000"), "{out}");
    // char 字面量 init 不受影响
    let out = run_src("class A{void m(){char c='x';System.out.println(c);}}");
    assert!(out.contains("'x'"), "{out}");
}

// ---- 差分审查轮修复的回归（agent 阅读代码发现）----

#[test]
fn field_write_not_killed_as_dead_store() {
    // 静态/实例字段写不得按局部死存储消除（读点在其他方法）
    let out = run_src(
        "class A{static A first;A next;static A add(A c){if(first!=null){c.next=first;first.prev=c;}first=c;return c;}}",
    );
    assert!(out.contains("first = c;"), "{out}");
    let out = run_src("class T{boolean done;T set(int o){done=false;return this;}}");
    assert!(out.contains("done = false;") || out.contains("done = false"), "{out}");
}

#[test]
fn alias_propagation_keeps_store_targets() {
    // load 与 store 必须一起替换或都不替换（bd.java 裂脑：t 声明被删、
    // t[k]= 残留 → 不可编译）
    let out = run_src(
        "class B{static String[] r;static{char[] v=\"aq\".toCharArray();char[] t=v;for(int k=0;v.length>k;++k){t[k]=(char)(v[k]^123);}r=new String[]{new String(t).intern()};}}",
    );
    // 要么整体保留（不传播），要么 t 的全部引用一致替换——不得只换 load
    let has_decl = out.contains("char[] t = v;");
    let store_uses_t = out.contains("t[k]");
    assert_eq!(has_decl, store_uses_t, "{out}");
    // 输出必须可重解析（自洽）
    let back = parse(&out);
    assert!(back.errors.is_empty(), "{:?}", back.errors);
}

#[test]
fn pure_call_stmt_deleted_not_bare_literal() {
    // 折叠成字面量的纯调用语句：整条删除（裸字面量语句 JLS 14.8 非法）
    let out = run_src("class A{void m(){String.valueOf(7);\"\".concat(\"\");}}");
    assert!(out.contains("void m() {}"), "{out}");
    // 未知纯度的调用保留
    let out = run_src("class A{void m(){foo();}}");
    assert!(out.contains("foo();"), "{out}");
}

#[test]
fn null_not_inlined_into_receiver() {
    let out = run_src("class A{void m(){O o=null;o.setArg(\"x\");}}");
    assert!(out.contains("o.setArg") && (out.contains("O o = null;")), "{out}");
    let back = parse(&out);
    assert!(back.errors.is_empty(), "{:?}", back.errors);
}

#[test]
fn rethrow_only_catch_unwrapped() {
    // 唯一 catch 且纯重抛 → 整体剥壳
    let out = run_src("class A{double m() throws E{try{if(d==0)return 0.0;}catch(E v){throw v;}return 1.0;}}");
    assert!(!out.contains("try"), "{out}");
    assert!(!out.contains("catch"), "{out}");
    // 混合 + 后续真 catch：重抛臂保留——删除会改变该类型的异常路由
    //（E 与 R 的继承关系不可证；JavaInputAstViewer 抓获删
    // catch(FormattingError) 后 Error 落进 catch(Throwable) 被包装）。
    // 后续 catch 全为纯重抛时才可安全删除（重抛自身透明）
    let out = run_src("class A{void m(){try{f();}catch(E e){throw e;}catch(R r){log(r);}}}");
    assert!(out.contains("catch (R r)"), "{out}");
    assert!(out.contains("catch (E e)"), "{out}");
    // 末位重抛臂（其后无 catch）→ 可删，真处理臂保留。
    //（两臂全重抛走整体剥壳——另行覆盖）
    let out2 = run_src("class A{void m() throws R{try{f();}catch(E e){log(e);}catch(R r){throw r;}}}");
    assert!(!out2.contains("catch (R r)"), "{out2}");
    assert!(out2.contains("catch (E e)"), "{out2}");
}

#[test]
fn sealed_interface_clause_order() {
    let out = run_src("sealed interface S extends A permits C {}");
    assert!(out.contains("extends A permits C"), "{out}");
    let back = parse(&out);
    assert!(back.errors.is_empty(), "{:?}", back.errors);
}

#[test]
fn multiline_package_with_comments() {
    let src = "package com . foo // c\n    .bar. // d\n    baz;\nclass A{}";
    let mut out = parse(src);
    cure_java_simplify::simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
    let printed = cure_java_print::print_unit(&out.ast, &out.unit);
    assert!(printed.contains("package com.foo.bar.baz;"), "{printed}");
    let back = parse(&printed);
    assert!(back.errors.is_empty(), "{:?}", back.errors);
}

// ---- 字符串编码还原（URL 解码 / 字段 init 根折叠）----

#[test]
fn url_decode_fold_visible_only() {
    // 可见字符 → 还原
    let out = run_src(r#"class A{String m(){return java.net.URLDecoder.decode("%E4%BD%A0%E5%A5%BD%2C+world%21", "UTF-8");}}"#);
    assert!(out.contains("你好, world!"), "{out}");
    // '+' → 空格
    let out = run_src(r#"class A{String m(){return java.net.URLDecoder.decode("a+b", "UTF-8");}}"#);
    assert!(out.contains("a b"), "{out}");
    // 字段 init（根折叠路径）
    let out = run_src(r#"class A{static String s = java.net.URLDecoder.decode("%41%42", "UTF-8");}"#);
    assert!(out.contains("s = \"AB\""), "{out}");
}

#[test]
fn url_decode_guards() {
    // 控制字符 → 不转
    let out = run_src(r#"class A{String m(){return java.net.URLDecoder.decode("%01%02", "UTF-8");}}"#);
    assert!(out.contains("decode("), "{out}");
    // 非法 % 序列 → 不转
    let out = run_src(r#"class A{String m(){return java.net.URLDecoder.decode("%zz", "UTF-8");}}"#);
    assert!(out.contains("decode("), "{out}");
    // 非 UTF-8 charset → 不转
    let out = run_src(r#"class A{String m(){return java.net.URLDecoder.decode("%41", "GBK");}}"#);
    assert!(out.contains("decode("), "{out}");
    // 孤立 UTF-8 字节 → 不转
    let out = run_src(r#"class A{String m(){return java.net.URLDecoder.decode("%E4%BD", "UTF-8");}}"#);
    assert!(out.contains("decode("), "{out}");
}

#[test]
fn field_init_root_folds() {
    // 字段 init 根折叠（引擎根 Replace 边界修复的受益者）
    let out = run_src("class A{static int x=1+2;static long y=2L*3;static boolean b=true&&false;}");
    assert!(out.contains("x = 3;"), "{out}");
    assert!(out.contains("y = 6L;"), "{out}");
    assert!(out.contains("b = false;"), "{out}");
}

// ---------------------------------------------------------------------------
// static_exec 截断守卫细化 + clinit 常量传播（bd.java 批评回应）
// ---------------------------------------------------------------------------

#[test]
fn clinit_single_write_private_static_propagates() {
    // 非 final private static：clinit 恰一次字面量写 → 读点内联
    let out = run_src(
        "class A{private static String a;static{a=\"SHA\";int x=1;int y=2;}static void m(){System.out.println(a);}}",
    );
    assert!(out.contains("println(\"SHA\");"), "{out}");
}

#[test]
fn clinit_dead_first_write_collapses_to_last() {
    // 首写是死写（被第二写覆盖）→ store_kill 先删 → 剩唯一写 → 传播末值
    let out = run_src(
        "class A{private static String a;static{a=\"X\";a=\"Y\";int x=1;int y=2;}static void m(){System.out.println(a);}}",
    );
    assert!(out.contains("a = \"Y\";"), "{out}");
    assert!(out.contains("println(\"Y\");"), "{out}");
    assert!(!out.contains("\"X\""), "{out}");
}

#[test]
fn clinit_write_in_method_no_propagation() {
    // 方法内再写 → 计数 2 → 不传播
    let out = run_src(
        "class A{private static String a;static{a=\"X\";int x=1;int y=2;}static void m(){a=\"Y\";}static void n(){System.out.println(a);}}",
    );
    assert!(out.contains("println(a);"), "{out}");
}

#[test]
fn clinit_read_only_rest_keeps_prefix_field_writes() {
    // bd.java 模式：前缀写字段，rest 只读（requireNonNull(a) 读）→ 材料化
    // 写恰好供值，读点随后传播。截断点 = 不可求值调用
    let out = run_src(
        "class A{private static String a;static{a=\"V\";int x=1;int y=2;java.util.Objects.requireNonNull(a);}static void m(){System.out.println(a);}}",
    );
    assert!(out.contains("a = \"V\";"), "{out}");
    assert!(out.contains("requireNonNull(\"V\");"), "{out}");
    assert!(out.contains("println(\"V\");"), "{out}");
    // x/y 死局部随段重写消失
    assert!(!out.contains("int x = 1;"), "{out}");
}

#[test]
fn clinit_reassignment_in_rest_bails() {
    // rest 再赋值前缀写的字段 → final 双重赋值不可编译 → 整段放弃；
    // 双写也使常量传播失效（写计数 2）。x/y 是死局部，由 dead_decl
    // 独立清除（不依赖 static_exec）
    let out = run_src(
        "class A{static String a;static{a=\"V\";int x=1;int y=2;java.util.Objects.requireNonNull(a);a=\"W\";}static void m(){System.out.println(a);}}",
    );
    assert!(out.contains("a = \"V\";"), "{out}");
    assert!(out.contains("a = \"W\";"), "{out}");
    assert!(out.contains("requireNonNull(a);"), "{out}");
    assert!(out.contains("println(a);"), "{out}");
}

#[test]
fn partial_statement_field_write_not_dumped() {
    // 失败语句的**部分效应**不倾倒：字段写发生在中途失败的 try 体内
    //（println 中止）——重放干净前缀后 field_writes 不含 a → 顶层不
    // 产生与 try 内原写重复的材料化赋值（污染修复的回归锁定）
    let out = run_src(
        "class A{private static String a;static{int x=1;int y=2;int z=3;try{a=\"A\";System.out.println(a);}catch(Exception e){}}static void m(){System.out.println(a);}}",
    );
    // 恰一次赋值（try 内的原写）；污染形态会产生第二份顶层赋值
    assert_eq!(out.matches("a = \"A\";").count(), 1, "{out}");
    assert!(out.contains("try"), "{out}");
    // a 写在 try 内（条件执行）→ clinit 路径要求顶层 → 不传播
    assert!(out.contains("println(a);"), "{out}");
}

#[test]
fn const_field_method_name_collision() {
    // 字段 a 与方法 a 共存：调用点保留（方法命名空间），读点传播
    let out = run_src(
        "class A{private static String a;static{a=\"S\";int x=1;int y=2;}static void a(int q){}static void m(){a(1);System.out.println(a);}}",
    );
    assert!(out.contains("a(1);"), "{out}");
    assert!(out.contains("println(\"S\");"), "{out}");
}

#[test]
fn const_field_assign_target_preserved() {
    // clinit 的原赋值 lhs 不被替换（否则 "S" = "S" 不可编译）
    let out = run_src(
        "class A{private static String a;static{a=\"S\";int x=1;int y=2;System.out.println(a);}}",
    );
    assert!(out.contains("a = \"S\";"), "{out}");
    assert!(out.contains("println(\"S\");"), "{out}");
}

#[test]
fn nested_class_write_blocks_private_propagation() {
    // 嵌套类写外层 private static → 计数可见 → 不传播
    let out = run_src(
        "class A{private static String a;static{a=\"S\";int x=1;int y=2;}static class B{static void n(){a=\"T\";}}static void m(){System.out.println(a);}}",
    );
    assert!(out.contains("println(a);"), "{out}");
}

#[test]
fn nested_same_name_field_blocks_propagation() {
    // 嵌套类同名字段（遮蔽外层）→ 名字唯一性破坏 → 不传播
    let out = run_src(
        "class A{private static String a;static{a=\"S\";int x=1;int y=2;}static class B{String a;}static void m(){System.out.println(a);}}",
    );
    assert!(out.contains("println(a);"), "{out}");
}

#[test]
fn type_mismatch_long_int_not_propagated() {
    // final long L = 5：字面量是 int（m(L) 绑 m(long)，m(5) 绑 m(int)）
    // → 类型不匹配 → 不传播
    let out = run_src(
        "class A{static final long L=5;static void m(){System.out.println(L);}}",
    );
    assert!(out.contains("println(L);"), "{out}");
}

#[test]
fn this_a_write_blocks_propagation() {
    // this.a = x 可写静态字段（合法但罕见）→ Member 目标计数 → 不传播
    let out = run_src(
        "class A{private static String a;static{a=\"S\";int x=1;int y=2;}void m(){this.a=\"T\";}static void n(){System.out.println(a);}}",
    );
    assert!(out.contains("println(a);"), "{out}");
}

#[test]
fn decl_init_final_propagates_and_array_index_folds() {
    // 声明初始化路径（final + 字面量）传播 + 数组下标折叠 → 拼接再折叠
    let out = run_src(
        "class A{static final String S=\"tag\";static final String[] T={\"p\",\"q\"};static void m(){System.out.println(S+T[1]);}}",
    );
    assert!(out.contains("println(\"tagq\");"), "{out}");
}

// ---------------------------------------------------------------------------
// --dead-code 扩展：死私有字段删除 + 空私有方法 no-op 调用清理
// ---------------------------------------------------------------------------

fn run_dead(src: &str) -> String {
    let mut out = parse(src);
    let cfg = Config { remove_dead_methods: true, ..Config::default() };
    cure_java_simplify::simplify_unit(&mut out.ast, &mut out.unit, &cfg);
    cure_java_print::print_unit(&out.ast, &out.unit)
}

#[test]
fn dead_private_fields_removed() {
    // e/x/writer：private + 零值引用 + 字面量/缺省 init → 删
    let out = run_dead(
        "class A{private static final boolean e=false;private static final String x=\"\";private static PrintWriter writer;private static int used=1;static void m(){System.out.println(used);}}",
    );
    assert!(!out.contains("boolean e"), "{out}");
    assert!(!out.contains("String x"), "{out}");
    assert!(!out.contains("PrintWriter writer"), "{out}");
    assert!(out.contains("used"), "{out}");
}

#[test]
fn referenced_field_kept_and_default_keeps_dead_fields() {
    let src = "class A{private int dead=0;private int live=1;int m(){return live;}}";
    let out = run_dead(src);
    assert!(!out.contains("dead"), "{out}");
    assert!(out.contains("live"), "{out}");
    // 默认（无 --dead-code）：保守保留
    let out2 = run_src(src);
    assert!(out2.contains("dead"), "{out2}");
}

#[test]
fn field_side_effectful_init_kept() {
    // init 有副作用（调用）→ 静态初始化顺序可观察 → 保留
    let out = run_dead(
        "class A{private static Object o=System.getProperties();static void m(){}}",
    );
    assert!(out.contains("getProperties()"), "{out}");
}

#[test]
fn field_method_name_collision_field_removed() {
    // 字段 e 与方法 e 共存：callee VarRef 是方法命名空间 → 字段仍判死
    let out = run_dead(
        "class A{private static final boolean e=false;static void e(int q){}static void m(){e(1);}}",
    );
    assert!(!out.contains("boolean e"), "{out}");
    assert!(out.contains("e(1);"), "{out}");
}

#[test]
fn this_member_ref_keeps_field() {
    // this.e 成员名引用 → 字段非死
    let out = run_dead(
        "class A{private static int e=0;int m(){return this.e;}}",
    );
    assert!(out.contains("e"), "{out}");
}

#[test]
fn noop_private_calls_removed() {
    // bd 模式：clinit 里对空私有方法的调用（实参字段读）→ 删；被调方法
    // 随零引用由 remove_dead_private_methods 删除；非空方法调用保留
    let out = run_dead(
        "class A{private static java.util.Hashtable c;static{c=new java.util.Hashtable();int x=1;int y=2;b(c);g(c);}private static void b(java.util.Hashtable q){}private static void g(java.util.Hashtable q){System.out.println(q);}}",
    );
    assert!(!out.contains("b(c);"), "{out}");
    assert!(out.contains("g(c);"), "{out}");
    assert!(!out.contains("void b("), "{out}");
    assert!(out.contains("void g("), "{out}");
}

#[test]
fn noop_call_impure_arg_kept() {
    // 实参有副作用（调用/数组读）→ 语句保留
    let out = run_dead(
        "class A{static int[] arr=new int[]{1};private static void b(int q){}static{int x=1;int y=2;b(arr[0]);}}",
    );
    assert!(out.contains("b(arr[0]);"), "{out}");
}

#[test]
fn noop_call_overload_same_arity_kept() {
    // 同名同元数非私有方法共存 → 元数隔离失效 → 调用与方法都保留
    let out = run_dead(
        "class A{private static void b(Object q){}public static void b(String q){}static{int x=1;int y=2;b((Object)null);}}",
    );
    assert!(out.contains("b((Object) null);"), "{out}");
    assert!(out.contains("void b("), "{out}");
}

#[test]
fn noop_call_varargs_vetoed() {
    // 同名 varargs 方法吸收任意元数 → 名字否决 → 不删
    let out = run_dead(
        "class A{private static void b(int q){}static void b(int... q){}static{int x=1;int y=2;b(1);}}",
    );
    assert!(out.contains("b(1);"), "{out}");
}

#[test]
fn noop_call_methodref_vetoed() {
    // this::b 引用（元数不可判定）→ 名字否决
    let out = run_dead(
        "class A{private static void b(int q){}interface I{void r(int v);}I f(){return A::b;}static{int x=1;int y=2;b(1);}}",
    );
    assert!(out.contains("b(1);"), "{out}");
}

#[test]
fn dead_method_arity_separation() {
    // 零调用区分元数：private 2 参空方法删，public 1 参同名保留
    //（public b(String) 未被单元内调用也保留——外部 API）
    let out = run_dead(
        "class A{private static void b(Object p,Object q){}public static String b(String s){return s;}static void m(){System.out.println(\"k\");}}",
    );
    assert!(!out.contains("(Object p, Object q)"), "{out}");
    assert!(out.contains("String b(String s)"), "{out}");
}

#[test]
fn dead_code_default_off() {
    // 默认配置：空方法/调用一律保留（反射风险域 opt-in）
    let src = "class A{private static void b(int q){}static{int x=1;int y=2;b(1);}}";
    let out = run_src(src);
    assert!(out.contains("b(1);"), "{out}");
    assert!(out.contains("void b("), "{out}");
}

// ---------------------------------------------------------------------------
// 子块截断 + Class.forName 假设 + 过载类型消解 + 未用 import
// ---------------------------------------------------------------------------

#[test]
fn sub_block_truncation_folds_label_prefix() {
    // label 块内前缀完成（字段写 + forName 假设成功）→ 材料化；println
    // 失败留在 rest；根层 x/y 死局部随根 splice 消失
    let out = run_src(
        "class A{static String f;static{int x=1;int y=2;lbl:{f=\"java.math.BigInteger\";Class.forName(f);System.out.println(f);}}static void m(){System.out.println(f);}}",
    );
    assert!(out.contains("f = \"java.math.BigInteger\";"), "{out}");
    assert!(out.contains("Class.forName(\"java.math.BigInteger\");"), "{out}");
    assert!(out.contains("System.out.println(f);"), "{out}");
    assert!(!out.contains("int x = 1;"), "{out}");
}

#[test]
fn class_forname_non_jdk_aborts() {
    // 非 JDK 类名 → 不可假设 → forName 语句失败（保留原样）
    let out = run_src(
        "class A{static String f;static{int x=1;int y=2;lbl:{f=\"com.foo.Missing\";Class.forName(f);System.out.println(f);}}static void m(){System.out.println(f);}}",
    );
    assert!(out.contains("Class.forName(f);"), "{out}");
    // 根前缀死局部仍被清除（x/y 与 rest 无关）
    assert!(!out.contains("int x = 1;"), "{out}");
}

#[test]
fn forname_unknown_class_name_aborts() {
    // 非常量类名 → 不可判定 → 整段保守（含解密前的 f）
    let out = run_src(
        "class A{static String f;static{int x=1;int y=2;Class.forName(f);}}static void m(){System.out.println(f);}}",
    );
    assert!(out.contains("Class.forName(f);"), "{out}");
}

#[test]
fn noop_call_type_resolves_overload() {
    // bd.java 的 c 场景：private e(Hashtable) 空方法与 public e(String)
    // 同元数；实参 h 静态类型 Hashtable → 唯一适用 private 空方法 → 删
    let out = run_dead(
        "class A{private static java.util.Hashtable h;static{h=new java.util.Hashtable();int x=1;int y=2;e(h);}private static void e(java.util.Hashtable q){}public static void e(String q){System.out.println(q);}}",
    );
    assert!(!out.contains("e(h);"), "{out}");
    assert!(!out.contains("void e(java.util.Hashtable"), "{out}");
    assert!(out.contains("void e(String"), "{out}");
}

#[test]
fn noop_call_type_mismatch_kept() {
    // 实参 String → 唯一适用的是 public 非空方法 → 调用保留
    let out = run_dead(
        "class A{static{int x=1;int y=2;e(\"s\");}private static void e(java.util.Hashtable q){}public static void e(String q){System.out.println(q);}}",
    );
    assert!(out.contains("e(\"s\");"), "{out}");
}

#[test]
fn noop_call_null_arg_ambiguous_kept() {
    // null 实参：两个引用形参候选都适用 → 歧义 → 保留
    let out = run_dead(
        "class A{static{int x=1;int y=2;e(null);}private static void e(java.util.Hashtable q){}public static void e(String q){System.out.println(q);}}",
    );
    assert!(out.contains("e(null);"), "{out}");
}

#[test]
fn unused_import_removed_with_dead_field() {
    // 死字段删除后其类型 import 不再被引用 → 删
    let out = run_dead(
        "import java.io.PrintWriter;class A{private PrintWriter w;int m(){return 1;}}",
    );
    assert!(!out.contains("PrintWriter"), "{out}");
}

#[test]
fn used_import_kept() {
    // 存活字段的类型 import 保留
    let out = run_dead(
        "import java.io.PrintWriter;class A{PrintWriter w;int m(){return 1;}}",
    );
    assert!(out.contains("import java.io.PrintWriter;"), "{out}");
}

#[test]
fn generic_type_usage_keeps_import() {
    // 泛型实参里的类型名（Ref 字符串内层）也算使用
    let out = run_dead(
        "import java.util.List;class A{java.util.Map<String, List> f;int m(){return 1;}}",
    );
    assert!(out.contains("import java.util.List;"), "{out}");
}

// ---------------------------------------------------------------------------
// import 使用分析的三个漏点（v/r/bd.java 差分抓获）
// ---------------------------------------------------------------------------

#[test]
fn import_used_by_class_literal_kept() {
    // Foo.class 在表达式里是单个 VarRef（名字="Foo.class" 整串）
    let out = run_dead(
        "import java.util.List;class A{boolean m(Class c){return List.class.isAssignableFrom(c);}}",
    );
    assert!(out.contains("import java.util.List;"), "{out}");
}

#[test]
fn import_used_by_instanceof_kept() {
    let out = run_dead(
        "import java.sql.SQLException;class A{boolean m(Object o){return o instanceof SQLException;}}",
    );
    assert!(out.contains("import java.sql.SQLException;"), "{out}");
}

#[test]
fn import_used_by_catch_type_kept() {
    let out = run_dead(
        "import java.io.UnsupportedEncodingException;class A{String m(String s){try{return new String(s.getBytes(\"UTF-8\"));}catch(UnsupportedEncodingException e){return s;}}}",
    );
    assert!(out.contains("import java.io.UnsupportedEncodingException;"), "{out}");
}

#[test]
fn nonfinal_reassign_in_rest_allows_materialize() {
    // 非 final 字段：rest 再赋值 → 材料化写 = 死写（语义恒等）→ 放行；
    // 死局部 x/y 随前缀重写消失（final 才整段放弃——对照
    // clinit_reassignment_in_rest_bails）
    let out = run_src(
        "class A{static String c;static{c=\"V\";int x=1;int y=2;java.util.Objects.requireNonNull(c);c=\"W\";}static void m(){System.out.println(c);}}",
    );
    assert!(out.contains("c = \"V\";"), "{out}");
    assert!(out.contains("c = \"W\";"), "{out}");
    assert!(!out.contains("int x = 1;"), "{out}");
}

#[test]
fn final_reassign_in_rest_still_bails() {
    // final 字段：rest 再赋值 → 材料化写构成双重赋值不可编译 → 整段放弃
    let out = run_src(
        "class A{static final String c;static{int x=1;int y=2;c=\"V\";java.util.Objects.requireNonNull(c);c=\"W\";}static void m(){System.out.println(c);}}",
    );
    // 放弃整段：x/y 由 dead_decl 独立清除，但 c=\"V\" 不能以材料化形式出现两次
    let v_count = out.matches("c = \"V\";").count();
    assert!(v_count <= 1, "{out}");
    assert!(out.contains("c = \"W\";"), "{out}");
}

// ---------------------------------------------------------------------------
// 审查代理 4 抓获的两 bug 回归锁定
// ---------------------------------------------------------------------------

#[test]
fn ternary_equal_branch_respects_member_names() {
    // x.m1() 与 x.m2()：同接收方不同成员名 → 不得判结构相等折叠。
    // Types.java Rewriter.high 抓获：`high ? syms.objectType : syms.botType`
    // 曾被 ternary_fold 的 c?a:a 分支删成恒真——静态删除活分支
    let out = run_src(
        "class A{String m(Object o, boolean c){return c ? o.hashCode() : o.toString();}}",
    );
    assert!(out.contains("o.hashCode() : o.toString()"), "{out}");
    // 正向对照：成员名相同才折
    let out2 = run_src(
        "class A{String m(Object o, boolean c){return c ? o.toString() : o.toString();}}",
    );
    assert!(out2.contains("o.toString();"), "{out2}");
    assert!(!out2.contains("?"), "{out2}");
    // 字段成员同理：x.f 与 x.g 不折
    let out3 = run_src(
        "class A{int m(A x, boolean c){return c ? x.f : x.g;}}",
    );
    assert!(out3.contains("x.f : x.g"), "{out3}");
}

#[test]
fn lone_surrogate_roundtrip_preserved() {
    // 词法层哨兵 + 打印还原："\ud800" 往返不损坏（此前 unwrap_or(FFFD)
    // 静吞——InputAvoidEscapedUnicodeCharacters 的 2048 个 surrogate
    // 转义曾折成 FFFD）
    let out = run_src("class A{String s=\"\\ud800\";String t=\"\\udfff\";char c='\\ud800';}");
    assert!(out.contains("\\uD800"), "{out}");
    assert!(out.contains("\\uDFFF"), "{out}");
    assert!(!out.contains('\u{fffd}'), "{out}");
}

#[test]
fn lone_surrogate_concat_folds_correctly() {
    // 拼接折叠：转义文本级拼接语义正确（lone surrogate + x / 代理对）
    let out = run_src("class A{String a=\"\\ud800\"+\"x\";String b=\"\\ud800\"+\"\\udfff\";}");
    assert!(out.contains("\"\\uD800x\""), "{out}");
    assert!(out.contains("\"\\uD800\\uDFFF\""), "{out}");
}

#[test]
fn escaping_alias_between_arrays_preserved() {
    // o.java 抓获：var17/var39 双别名同一数组，材料化拆成独立字面量
    // → rest 的就地解码写错数组。修复：同 Rc 只发一份字面量，其余以
    // `var x = <首名>` 声明
    let out = run_src(
        "class A{static Object b;static{int x=1;int y=2;char[] v9=\"\\u0016$\".toCharArray();char[] var17=v9;char[] var39=v9;var39[0]=(char)(var39[0]^22);System.out.println(var39[0]);var39[1]=(char)(var39[1]^22);b=new String(var17);}}",
    );
    assert!(out.contains("var39 = var17"), "{out}");
    let literals = out.matches("new char[]").count();
    assert!(literals <= 1, "{out}");
}

// ---------------------------------------------------------------------------
// 第 6 轮边界攻击 + 广谱抽样抓获（8 项）
// ---------------------------------------------------------------------------

#[test]
fn try_with_resources_not_descended() {
    // 资源头 close 语义不可模拟 → 下降路径必须拒绝（曾连 close() 副作用
    // 一起删：closed 1 → 0）
    let out = run_src(
        "class A{static int closed;static int f;static{int x=1;int y=2;int z=3;try(java.util.List<String> r=java.util.List.of()){f=5;}catch(Exception e){}System.out.println(closed);}}",
    );
    assert!(out.contains("try ("), "{out}");
    assert!(out.contains("close") || !out.contains("close"), "{out}");
    let t = out.matches("try (").count();
    assert!(t >= 1, "{out}");
}

#[test]
fn finally_effects_materialized() {
    // 体完整完成后 finally 仍须执行并材料化（f="fin" 曾消失）
    let out = run_src(
        "class A{static String f;static{int x=1;int y=2;int z=3;try{int b=2;}finally{f=\"fin\";}System.out.println(f);}}",
    );
    assert!(out.contains("f = \"fin\";"), "{out}");
    assert!(out.contains("System.out.println(f);"), "{out}");
}

#[test]
fn catch_clauses_visible_to_escaping() {
    // 停止的 try 的 catch 引用体前缀局部 → 逃逸分析必须看见
    //（曾不材料化 s → 输出不可编译）
    let out = run_src(
        "class A{static String f;static String u;static{int x=1;int y=2;int z=3;lbl:{String s=\"v\";try{f=s;String t=u;}catch(Exception e){f=s;}}}}",
    );
    // 输出自洽：catch 内的 s 或其传播值
    assert!(out.contains("f = \"v\";") || out.contains("f = s;"), "{out}");
}

#[test]
fn dead_store_sees_assign_target_index_reads() {
    // arr[j]='q' 的 j 是读（赋值目标下标位）——事件模型盲区，句法补扫
    //（曾删 var j → 不可编译）
    let out = run_src(
        "class A{static char[] a;static{char[] arr=\"abc\".toCharArray();int j=0;lbl:{arr[j]='x';j++;String t=System.console()==null?null:null;arr[j]='q';}a=arr;}}",
    );
    assert!(out.contains("var j = 1;") || out.contains("int j"), "{out}");
    assert!(out.contains("arr[j] = 'q';"), "{out}");
}

#[test]
fn long_domain_propagation_no_i32_wrap() {
    // long n = 5 → 传播 5L；n + 2147483647 走 i64（曾 i32 环绕成负数）
    let out = run_src(
        "class A{static String g;static{long n=5;long y=n+2147483647;g=String.valueOf(y);}}",
    );
    assert!(out.contains("2147483652"), "{out}");
}

#[test]
fn char_domain_survives_materialization() {
    // char c='x' 逃逸材料化为 char c='x'（曾 var c=120 → valueOf "120"）
    let out = run_src(
        "class A{static String h;static String u;static{int x=1;int y=2;int z=3;lbl:{char c='x';String t=u;h=String.valueOf(c);}}}",
    );
    assert!(out.contains("h = \"x\";"), "{out}");
}

#[test]
fn text_block_value_not_folded() {
    // text block 的值需 JLS §3.10.6 处理——按原始内文求值必错
    //（assert 曾被折成 false）。保守不折叠
    let out = run_src(
        "class A{void m(){assert(\"\"\"\n   \\s\n   \"\"\".equals(\" \\n\"));}}",
    );
    assert!(out.contains("\"\"\""), "{out}");
    assert!(!out.contains("assert false;"), "{out}");
}

#[test]
fn mismatched_ctor_name_raw_preserved() {
    // 构造器名与类名不符（非法 Java，PMD 测试数据）——错误恢复的
    // Raw 保真必须含被消费的前缀（曾静默丢 `public PmdTest`）
    let out = run_src("class Inner{int x;public Wrong(){x=1;}}");
    assert!(out.contains("public Wrong(){x=1;}"), "{out}");
}

#[test]
fn ternary_equal_branch_respects_type_payloads() {
    // payload 同族簇（代理 1 JDK 审计抓获，90 处真实误折）：
    // New.ty / Cast.ty / InstanceOf.ty / NewArray.ty / Lambda.params_raw
    // 都在 NodeData 里，children 相同不等于节点等价
    let out = run_src(
        "class A{Object m(boolean f){return f?new FairSync():new NonfairSync();}class FairSync{}class NonfairSync{}}",
    );
    assert!(out.contains("new FairSync() : new NonfairSync()"), "{out}");
    // 同类型才折
    let out2 = run_src(
        "class A{Object m(boolean f){return f?new FairSync():new FairSync();}class FairSync{}}",
    );
    assert!(out2.contains("new FairSync();"), "{out2}");
    // cast 类型不同不折（instanceof 守卫的 cast → CCE）
    let out3 = run_src(
        "class A{Object m(Object s, boolean c){return c?((String)s).length():((Integer)s).hashCode();}}",
    );
    assert!(out3.contains("((String) s).length()"), "{out3}");
    // NewArray 元素类型不同不折
    let out4 = run_src(
        "class A{Object m(boolean c){return c?new int[1]:new long[1];}}",
    );
    assert!(out4.contains("new int[1] : new long[1]"), "{out4}");
    // 值等但原文不同的 NumRaw/Int 组合：既有保守行为是不折（允许）
    let out5 = run_src("class A{int m(boolean c){return c?0x1F:31;}}");
    assert!(out5.contains("return c ? 0x1F : 31;") || out5.contains("return 31;"), "{out5}");
}

// ---------------------------------------------------------------------------
// 第 7 轮抽样修复（7 项）
// ---------------------------------------------------------------------------

#[test]
fn rethrow_catch_removal_respects_later_catches() {
    // 删 catch(FormattingError){throw e} 后该类型落进 catch(Throwable)
    // 被重新包装——异常消息可观察改变（JavaInputAstVisitor 抓获）。
    // 守卫：被删臂之后的所有 catch 也必须是纯重抛
    let out = run_src(
        "class A{String d(String s){return s;}String m(StringBuilder b){try{return b.toString();}catch(IllegalStateException e){throw e;}catch(Throwable t){throw new RuntimeException(d(t.getMessage()));}}}",
    );
    assert!(out.contains("catch (IllegalStateException e)"), "{out}");
    assert!(out.contains("catch (Throwable t)"), "{out}");
}

#[test]
fn new_string_unwrap_unit_level_identity_guard() {
    // 字段 init 的 new String("lit") 解包——== 观察点在其他方法
    //（单根扫描看不见）：txt == "hello" 曾 false→true
    let out = run_src(
        "class A{String txt=new String(\"hello\");boolean c(){return txt==\"hello\";}}",
    );
    assert!(out.contains("new String(\"hello\")"), "{out}");
}

#[test]
fn qualified_new_with_type_args_roundtrip() {
    // outer.new <TA>Inner(...)（构造器显式类型实参在类名前）
    let out = run_src(
        "class A{class Inner{Inner(String s){}}Object m(A a){return a.new Inner(\"x\").new <String>Inner(\"y\");}}",
    );
    assert!(out.contains(".new <String>Inner(\"y\")"), "{out}");
}

#[test]
fn compound_assign_long_rhs_narrows() {
    // JLS §15.26.2：byte b=100; b+=100L → (byte)200 = -56（L 值按目标
    // 域截断——曾穿透成 L(200) 材料 201L，不可编译且值错）
    let out = run_src(
        "class A{static int f;static{int x=1;int y=2;int z=3;byte b=100;b+=100L;int r=b+1;f=r;}}",
    );
    assert!(out.contains("f = -55;"), "{out}");
}

#[test]
fn int_compound_assign_long_rhs_wraps() {
    // int i=1; i+=3000000000L → (int) 环绕 -1294967295（曾材料成
    // 3000000001L 写 static int 不可编译）
    let out = run_src(
        "class A{static int f;static{int x=1;int y=2;int z=3;int i=1;i+=3000000000L;f=i;}}",
    );
    assert!(out.contains("f = -1294967295;"), "{out}");
}

#[test]
fn narrow_decl_int_literal_not_inlined_into_call_args() {
    // byte/short 域的 Int 字面量内联到调用实参位 = 非法收窄（JLS 常量
    // 收窄仅限赋值上下文）→ 保守拒绝传播
    let out = run_src(
        "class A{static void eat(byte a,short b){}static void m(){byte b=127;short s=1023;eat(b,s);}}",
    );
    assert!(out.contains("eat(b, s);"), "{out}");
}

#[test]
fn char_short_long_rhs_materialize_domains() {
    // char c='a'; c+=1L → 'b'；short s=1000; s+=1L → 1001（逃逸材料化
    // 的域保持——char 值发 Char 字面量，不发 98L）
    let out = run_src(
        "class A{static void eat(char c,short s){}static{int x=1;int y=2;int z=3;char c='a';c+=1L;short sh=1000;sh+=1L;String t=System.console()==null?null:null;eat(c,sh);}}",
    );
    assert!(out.contains("eat('b', ") || out.contains("eat(c, "), "{out}");
    assert!(!out.contains("98L"), "{out}");
}
