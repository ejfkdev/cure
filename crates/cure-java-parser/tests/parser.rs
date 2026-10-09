//! 解析器测试：golden 打印快照 + 容错恢复。

use cure_java_parser::parse;

fn fmt(src: &str) -> String {
    let out = parse(src);
    cure_java_print::print_unit(&out.ast, &out.unit)
}

fn fmt_nice(src: &str) -> String {
    // 打印后去掉空行差异，便于对比
    fmt(src)
}

#[test]
fn smoke_class() {
    let src = r#"
package com.example;

import java.util.List;
import static java.lang.Math.max;

public final class Foo extends Base implements I1, I2 {
    private static final int MAX = 0x1F;

    private String name = "hello";

    Foo(int a) {
        this.name = String.valueOf(a);
    }

    public int bar(int x) throws Exception {
        return x + 1;
    }

    @Override
    public String toString() {
        return name;
    }
}
"#;
    let out = fmt_nice(src);
    assert!(out.contains("package com.example;"), "{out}");
    assert!(out.contains("import java.util.List;"), "{out}");
    assert!(out.contains("import static java.lang.Math.max;"), "{out}");
    assert!(
        out.contains("public final class Foo extends Base implements I1, I2 {"),
        "{out}"
    );
    assert!(out.contains("private static final int MAX = 0x1F;"), "{out}");
    assert!(out.contains("private String name = \"hello\";"), "{out}");
    assert!(out.contains("Foo(int a) {"), "{out}");
    assert!(out.contains("public int bar(int x) throws Exception {"), "{out}");
    assert!(out.contains("@Override"), "{out}");
}

#[test]
fn statements_coverage() {
    let src = r#"
class A {
    void m(int n, int[] arr, java.util.List<String> list) {
        if (n > 0) {
            return;
        } else if (n < 0) {
            n = -n;
        } else {
            n = 0;
        }
        while (n < 10) {
            n++;
        }
        do {
            n--;
        } while (n > 0);
        for (int i = 0, j = 1; i < n; i++, j--) {
            arr[i] = i * 2;
        }
        for (String s : list) {
            System.out.println(s);
        }
        try (java.io.Closeable c = open(); java.io.Closeable d = open2()) {
            use(c, d);
        } catch (java.io.IOException | RuntimeException e) {
            log(e);
        } finally {
            done();
        }
        synchronized (lock) {
            n++;
        }
        outer: for (int i = 0; i < n; i++) {
            if (bad) break outer;
            if (skip) continue outer;
        }
        assert n >= 0 : "negative";
        switch (n) {
            case 1:
                one();
                break;
            case 2:
            case 3:
                two();
                break;
            default:
                other();
        }
        switch (n) {
            case 1 -> one();
            default -> { other(); }
        }
    }
}
"#;
    let out = fmt_nice(src);
    assert!(out.contains("} else if (n < 0) {"), "{out}");
    assert!(out.contains("for (int i = 0, j = 1; i < n; i++, j--) {"), "{out}");
    assert!(out.contains("for (String s : list) {"), "{out}");
    assert!(
        out.contains("try (java.io.Closeable c = open(); java.io.Closeable d = open2()) {"),
        "{out}"
    );
    assert!(
        out.contains("catch (java.io.IOException | RuntimeException e) {"),
        "{out}"
    );
    assert!(out.contains("synchronized (lock) {"), "{out}");
    assert!(out.contains("outer:"), "{out}");
    assert!(out.contains("break outer;"), "{out}");
    assert!(out.contains("assert n >= 0 : \"negative\";"), "{out}");
    assert!(out.contains("case 2:"), "{out}");
    assert!(out.contains("case 1 -> one();"), "{out}");
    assert!(out.contains("default -> {"), "{out}");
}

#[test]
fn expressions_and_precedence() {
    let src = r#"
class E {
    int m(int a, int b, boolean flag) {
        int r1 = 1 + 2 * 3;
        int r2 = (1 + 2) * 3;
        int r3 = a << 2 & 7 | 3 ^ 1;
        int r4 = a > 0 ? a : -a;
        boolean r5 = flag && a > 0 || b < 1;
        int r6 = a += 5;
        Object o = (Object) a;
        int r7 = (a) + b;
        boolean r8 = a instanceof Integer;
        Runnable rn = () -> doIt();
        java.util.function.IntUnaryOperator f = x -> x + 1;
        Runnable block = () -> {
            doIt();
            doMore();
        };
        java.util.Comparator<String> c = String::compareToIgnoreCase;
        int[] xs = new int[]{1, 2, 3};
        int[][] grid = new int[3][4];
        Object lst = new java.util.ArrayList<String>(10);
        return r1 + r2 + r3 + r4 + r6 + r7;
    }
}
"#;
    let out = fmt_nice(src);
    assert!(out.contains("int r1 = 1 + 2 * 3;"), "{out}");
    assert!(out.contains("int r2 = (1 + 2) * 3;"), "{out}");
    assert!(out.contains("int r3 = a << 2 & 7 | 3 ^ 1;"), "{out}"); // Java 优先级下无需括号
    assert!(out.contains("int r4 = a > 0 ? a : -a;"), "{out}");
    assert!(out.contains("boolean r5 = flag && a > 0 || b < 1;"), "{out}"); // && 优先于 ||
    assert!(out.contains("int r6 = a += 5;"), "{out}");
    assert!(out.contains("Object o = (Object) a;"), "{out}");
    assert!(out.contains("int r7 = a + b;"), "{out}"); // 冗余括号被去除
    assert!(out.contains("boolean r8 = a instanceof Integer;"), "{out}");
    assert!(out.contains("Runnable rn = () -> doIt();"), "{out}");
    assert!(out.contains("java.util.function.IntUnaryOperator f = (x) -> x + 1;"), "{out}");
    assert!(out.contains("java.util.Comparator<String> c = String::compareToIgnoreCase;"), "{out}");
    // 声明处 new T[]{…} 规范化为惯用短形态 {…}（语义等价，JLS）
    assert!(out.contains("int[] xs = {1, 2, 3};"), "{out}");
    assert!(out.contains("int[][] grid = new int[3][4];"), "{out}");
    assert!(out.contains("Object lst = new java.util.ArrayList<String>(10);"), "{out}");
}

#[test]
fn generics_split_token() {
    // Map<K, List<V>> 的 ">>" 切分是词法级经典坑
    let src = r#"
class G {
    java.util.Map<String, java.util.List<Integer>> m = new java.util.HashMap<>();
}
"#;
    let out = fmt_nice(src);
    assert!(
        out.contains("java.util.Map<String, java.util.List<Integer>> m = new java.util.HashMap<>();"),
        "{out}"
    );
}

#[test]
fn interface_enum_nested_varargs() {
    let src = r#"
public interface Api {
    int CONST = 42;

    String name();

    default void log(String msg) {
        System.out.println(msg);
    }

    enum Color {
        RED, GREEN(2), BLUE {
            @Override
            public int code() {
                return 3;
            }
        };

        public int code() {
            return 1;
        }
    }

    class Impl implements Api {
        public String name() {
            return "impl";
        }
    }

    static <T> T pick(T a, T b, T... rest) {
        return rest.length > 0 ? rest[0] : a;
    }
}
"#;
    let out = fmt_nice(src);
    assert!(out.contains("public interface Api {"), "{out}");
    assert!(out.contains("int CONST = 42;"), "{out}");
    assert!(out.contains("default void log(String msg) {"), "{out}");
    assert!(out.contains("enum Color {"), "{out}");
    assert!(out.contains("RED, GREEN(2), BLUE {"), "{out}");
    assert!(out.contains("class Impl implements Api {"), "{out}");
    assert!(out.contains("static <T> T pick(T a, T b, T... rest) {"), "{out}");
}

// ---------------------------------------------------------------------------
// 容错恢复
// ---------------------------------------------------------------------------

#[test]
fn recovery_broken_method_preserved_good_method_optimized() {
    // 一个坏方法 + 一个好方法：坏方法原文保真，好方法照常解析
    let src = r#"
class A {
    void broken() {
        int x = ;
        foo(= dead);
    }

    int good() {
        int a = foo();
        int b = a;
        return b;
    }
}
"#;
    let out = parse(src);
    assert!(!out.errors.is_empty(), "should report errors");
    let printed = cure_java_print::print_unit(&out.ast, &out.unit);
    assert!(printed.contains("void broken() {"), "{printed}");
    // 坏语句原文保真
    assert!(printed.contains("int x = ;"), "{printed}");
    assert!(printed.contains("foo(= dead);"), "{printed}");
    // good() 结构完好（优化断言见 cure-java-simplify 的端到端测试）
    assert!(printed.contains("int b = a;"), "{printed}");
}

#[test]
fn recovery_broken_statement_inside_method() {
    let src = r#"
class A {
    int m(boolean flag) {
        int a = 1;
        bar(= broken);
        if (flag) {
            return true;
        } else {
            return false;
        }
    }
}
"#;
    let out = parse(src);
    let printed = cure_java_print::print_unit(&out.ast, &out.unit);
    // 坏语句被保留
    assert!(printed.contains("bar(= broken);"), "{printed}");
    // if 结构完好（优化断言见 cure-java-simplify 端到端测试）
    assert!(printed.contains("return true;"), "{printed}");
}

#[test]
fn recovery_missing_semicolon() {
    // 缺分号：恢复应停在 return 前，而不是吞掉 return
    let src = r#"
class A {
    int m() {
        int x = 5
        return x;
    }
}
"#;
    let out = parse(src);
    let printed = cure_java_print::print_unit(&out.ast, &out.unit);
    assert!(printed.contains("return x;"), "{printed}");
}

#[test]
fn recovery_unterminated_string() {
    let src = r#"
class A {
    String m() {
        return "unterminated...
    }
}
"#;
    let out = parse(src);
    assert!(!out.errors.is_empty());
    // 不崩溃即成功；错误有行号
    assert!(out.errors.iter().any(|e| e.line >= 3));
}

#[test]
fn recovery_total_garbage() {
    let src = "}}{{ 完全不是 Java !!! (((";
    let out = parse(src);
    // 不 panic、有错误报告
    assert!(!out.errors.is_empty());
}

#[test]
fn no_errors_on_valid_input() {
    let src = r#"
package p;
class B {
    int f() {
        return 1;
    }
}
"#;
    let out = parse(src);
    assert!(
        out.errors.is_empty(),
        "unexpected errors: {:?}",
        out.errors
    );
}

// ---------------------------------------------------------------------------
// 显式类型实参（type witness）与 JSR 308 类型使用位置注解——保往返
// ---------------------------------------------------------------------------

#[test]
fn type_witness_roundtrip() {
    // 泛型方法调用 witness 是 javac 推断必需实参（Stream.<Path>empty()
    // 曾丢后 CAP#1 推断失败不可编译——PathUtils 抓获）
    let src = r#"
import java.util.*;
import java.util.stream.*;
class W {
    List<String> a = Collections.<String>emptyList();
    java.util.List<String> b() {
        return java.util.Collections.<String>unmodifiableList(new ArrayList<String>());
    }
    Stream<String> c() {
        return Stream.<String>empty();
    }
    Object d() {
        return this.<String>gen();
    }
    <T> T gen() {
        return null;
    }
}
"#;
    let out = fmt(src);
    assert!(out.contains("Collections.<String>emptyList()"), "{out}");
    assert!(out.contains("java.util.Collections.<String>unmodifiableList("), "{out}");
    assert!(out.contains("Stream.<String>empty()"), "{out}");
    assert!(out.contains("this.<String>gen()"), "{out}");
}

#[test]
fn type_use_annotations_roundtrip() {
    // JSR 308：返回位置/类型参数 bound/cast/instanceof 模式的纯 TYPE_USE
    // 注解保留（曾丢弃）；new 数组维度注解跳过（parse_type 维度策略）
    let src = r#"
class T {
    @interface U { }
    @U String ret() {
        return null;
    }
    <G extends @U Object> @U G generic(G g) {
        return g;
    }
    Object c(Object o) {
        return (@U String) o;
    }
    boolean i(Object o) {
        return o instanceof @U String s && s.length() > 0;
    }
    int[] n() {
        return new int @U [3];
    }
}
"#;
    let out = fmt(src);
    assert!(out.contains("@U String ret()"), "{out}");
    assert!(out.contains("<G extends @U Object> @U G generic(G g)"), "{out}");
    assert!(out.contains("(@U String) o"), "{out}");
    assert!(out.contains("instanceof @U String s"), "{out}");
    // 维度注解跳过但表达式必须可解析（曾 "bad new expression" 区域跳过）
    assert!(out.contains("new int[3]"), "{out}");
}

// ---------------------------------------------------------------------------
// 第 11 轮代理修复回归（for-init 维度/文本块首行/一元+/catch final/
// 限定段注解/失控循环/语句注解）
// ---------------------------------------------------------------------------

#[test]
fn for_init_multi_declarator_cstyle_dims() {
    // ExoticJava 抓获：j 的 [] 曾丢失；k 的 [] 曾并入共享类型感染 l
    let out = fmt("class A{void m(){for (int i = 10, j[] = {20}; i < 5; i++, j[0]++){}for (int k[] = new int[2], l = 0; l < 2; l++){}for (int a = 1, b = 2, c[] = {3}; a < 1; a++){}for (int s[] = new int[1]; s[0] < 3; s[0]++){}}}");
    assert!(out.contains("for (int i = 10, j[] = {20}; i < 5; i++, j[0]++)"), "{out}");
    assert!(out.contains("for (int k[] = new int[2], l = 0; l < 2; l++)"), "{out}");
    assert!(out.contains("for (int a = 1, b = 2, c[] = {3}; a < 1; a++)"), "{out}");
    assert!(out.contains("for (int[] s = new int[1]; s[0] < 3; s[0]++)"), "{out}");
}

#[test]
fn text_block_first_line_discard() {
    // lombok TextBlocks 抓获：开行尾随空白曾进入内容——打印再插 \n 值漂移
    // （javac len=0 → cure 往返 len=1）。JLS 3.10.6：开定界符到首个行终止
    // 符的内容整体丢弃
    let src = "class A{String ex4 = \"\"\"   \n\t\t\"\"\";String std = \"\"\"\nabc\n\"\"\";}";
    let out = fmt(src);
    assert!(out.contains("\"\"\"\n\t\t\"\"\""), "{out}");
    assert!(out.contains("\"\"\"\nabc\n\"\"\""), "{out}");
}

#[test]
fn unary_plus_retained() {
    // 数值提升即语义：Object o = +c 装箱 Integer 而非 Character（第 11 轮
    // 代理抓获曾直接丢弃）
    let out = fmt("class A{Object o = +'c';int i = +5;int j = + +5;int q(int x){return +x;}}");
    assert!(out.contains("+'c'"), "{out}");
    assert!(out.contains("+5"), "{out}");
    assert!(out.contains("+(+5)"), "{out}");
    assert!(out.contains("return +x;"), "{out}");
}

#[test]
fn catch_param_final_retained() {
    // mockito 5 + lombok 21 文件抓获：catch 形参 final 曾丢弃（方法形参
    // /局部 final 保留——仅 catch 位失守）
    let out = fmt("class A{void m(){try{x();}catch (final Exception e){}}void x(){}}");
    assert!(out.contains("catch (final Exception e)"), "{out}");
}

#[test]
fn qualified_segment_type_annotation_retained() {
    // Outer.@NonNull Inner——JSR 308 段级注解曾丢弃
    let out = fmt("class A{Outer.@NonNull Inner f;class Outer{class Inner{}}@interface NonNull{}}");
    assert!(out.contains("Outer.@NonNull Inner f;"), "{out}");
}

#[test]
fn qualified_statement_annotation_no_false_error() {
    // @lombok.Cleanup 语句级限定注解：曾假报 unexpected token（内容保真
    // 但错误计数污染）
    let out = parse("class A{void m(){@lombok.Cleanup java.io.Writer w = null;}}");
    assert!(
        out.errors.is_empty(),
        "unexpected errors: {:?}",
        out.errors
    );
}

#[test]
fn no_runaway_error_loop_on_malformed_for() {
    // lombok after-ecj 残骸抓获：for 双分号曾 4M 错误/529MB 原位自旋
    //（sync_stmt 停在闭括号不消费 + 块循环无停滞守卫）
    let out = parse("class A{void m(){for (int i=0;; (i<10); i++) {System.out.println(i);}}}");
    assert!(
        out.errors.len() < 10,
        "runaway: {} errors",
        out.errors.len()
    );
    // 输出可重解析（幂等基础）：同样有界错误
    let printed = cure_java_print::print_unit(&out.ast, &out.unit);
    let reparsed = parse(&printed);
    assert!(
        reparsed.errors.len() < 10,
        "reparse runaway: {} errors",
        reparsed.errors.len()
    );
}

// ---------------------------------------------------------------------------
// 第 12 轮修复回归
// ---------------------------------------------------------------------------

#[test]
fn for_init_declarator_dims_add() {
    // A5e 抓获：维度相加（JLS 14.14/14.4：声明类型 dims + 声明符自带）
    let out = fmt("class A{void m(){for (String[] s, t[][];;){break;}}}");
    assert!(out.contains("t[][][]"), "{out}");
    // 局部声明同法
    let out2 = fmt("class A{void m(){int[] p = null, q[][] = null;int r[] = p;}}");
    assert!(out2.contains("int[] p = null;"), "{out2}");
    assert!(out2.contains("int[][][] q = null;"), "{out2}");
}

#[test]
fn member_level_stall_guard() {
    // A8mem 抓获：顶层残骸 `}` 曾使成员循环 2M 自旋（2M 错误/147MB）
    let out = parse("class C { void m() {} }\nvoid n() {} }\n");
    assert!(out.errors.len() < 10, "runaway: {} errors", out.errors.len());
    let printed = cure_java_print::print_unit(&out.ast, &out.unit);
    let reparsed = parse(&printed);
    assert!(
        reparsed.errors.len() < 10,
        "reparse runaway: {}",
        reparsed.errors.len()
    );
}

#[test]
fn catch_param_annotation_before_final() {
    // A9g 抓获：`catch (@A final X e)` 注解在 final 前——JLS 14.20
    // VariableModifier 任意序；曾 parse_type 失败输出结构破坏且非幂等
    let out = fmt("class A{@interface B{ }void m(){try{x();}catch (@B final Exception e){}catch (final @B Exception e){}}void x(){}}");
    assert!(out.contains("catch (@B final Exception e)"), "{out}");
    assert!(out.contains("catch (final @B Exception e)"), "{out}");
}

#[test]
fn local_record_interleaved_modifiers_verbatim() {
    // F2 抓获：`final @Deprecated static record R(…) {}` 的 static 在注解后
    // ——修饰/注解交错曾使局部 record 退化残句丢 {} 不可编译；修饰符
    // 原文保真（@Deprecated 有运行时可观察性）
    let out = fmt("class A{void m(){final @Deprecated static record R(String a) {}record Ok(String a) {}}}");
    assert!(out.contains("final @Deprecated static record R(String a) {}"), "{out}");
    assert!(out.contains("record Ok(String a) {}"), "{out}");
}

#[test]
fn empty_case_statement_dropped_for_idempotency() {
    // F3 抓获：`case X:;` 的裸 ; 打印空行后重解析消失——非幂等
    let src = "class A{void m(int x){switch (x){case 1:;default:case 2:;}}}";
    let p1 = fmt(src);
    let p2 = fmt(&p1);
    assert_eq!(p1, p2, "not idempotent");
}

// ---------------------------------------------------------------------------
// 第 13 轮终审修复回归（R13c 破损 for 头 / P1 空体）
// ---------------------------------------------------------------------------

#[test]
fn malformed_for_header_recovery_bounded() {
    // R13 P0-1（after-ecj 三半分号 for）：错误有界 + 幂等 + 类结构不破坏
    //（成员不移位、循环体语句不丢）
    let src = "class C{int t=1;C(int t){this.t=t;}static int f(java.util.List<String> k){java.util.Map<String,String> m=new java.util.LinkedHashMap<String,String>();for(int $i=0;;($i<k.size());$i++)m.put(k.get($i),k.get($i));m=java.util.Collections.unmodifiableMap(m);return m.size();}}";
    let p = parse(src);
    assert!(p.errors.len() < 10, "runaway: {} errors", p.errors.len());
    let printed = cure_java_print::print_unit(&p.ast, &p.unit);
    assert!(printed.contains("C(int t)"), "member displaced: {printed}");
    let reparsed = parse(&printed);
    assert!(reparsed.errors.len() < 10, "reparse: {}", reparsed.errors.len());
    assert_eq!(
        printed,
        cure_java_print::print_unit(&reparsed.ast, &reparsed.unit),
        "not idempotent"
    );
}

#[test]
fn all_blank_block_prints_single_line() {
    // R13 P1（EmptyStatementComments）：const_condition 把 if(true); 换成
    // Empty 残留——全空块多行形态重解析丢孩子 → 非幂等
    let src = "class A{void m1(){if(true);if(true);}}";
    let p1 = fmt(src);
    let p2 = fmt(&p1);
    assert_eq!(p1, p2, "not idempotent");
}

#[test]
fn for_semicolon_form_keeps_keyword_idempotent() {
    // R14 族 D：`for ;` 的 expect("(") 失败路径曾从 ; 起取 Raw 丢 for
    // → 孤儿 ; 二轮消失
    let p1 = fmt("class D{void m(){for ;}}");
    let p2 = fmt(&p1);
    assert_eq!(p1, p2, "not idempotent: {p1}");
}

#[test]
fn case_region_open_paren_garbage_bounded() {
    // R14 族 E：case 区 `foo(` 开括号垃圾——界内同步（到下一 case/
    // default/`}` 止），不吞后续 case 与 switch 闭括号（括号平衡保持
    // 输入原状）；`return r` 留在方法内
    let src = "class P{int m(int x){int r=0;switch(x){foo(case 1:r=1;break;default:r=2;}return r;}}";
    let p = parse(src);
    let printed = cure_java_print::print_unit(&p.ast, &p.unit);
    assert!(printed.contains("return r;"), "{printed}");
    let o = printed.matches('(').count();
    let c = printed.matches(')').count();
    let so = src.matches('(').count();
    let sc = src.matches(')').count();
    assert!(o - c == so - sc, "paren balance changed: {o}/{c} vs {so}/{sc}");
}


#[test]
fn local_decl_raw_with_array_initializer() {
    // R19 P0-1：局部声明整句 Raw 遇数组初始化器被 sync_stmt 的 depth-0
    // `{` 拦腰截断（其后重解析成块——括号消解、尾分号被吞）。
    // raw_local_decl：`=` 之后的 `{` 是初始化器 → 平衡吞
    let out = fmt(
        "class M{@interface A{}void m(){int @A [] a={1,2};int b @A []=new int[]{3};int c,d @A []={4};}}",
    );
    assert!(out.contains("int @A [] a={1,2};"), "{out}");
    assert!(out.contains("int b @A []=new int[]{3};"), "{out}");
    assert!(out.contains("int c,d @A []={4};"), "{out}");
}

#[test]
fn raw_local_decl_stops_at_block_close() {
    // R20 P1：残缺 init（int x = new A 直撞块尾）的 depth-0 `}` 是方法/
    // 块闭括号——raw_local_decl 曾吞进 Raw span → 打印器再补闭括号 →
    // 每轮 +2 括号无界发散（Sample.java 幂等 0→1 实锺）
    let src = "class MR{void m(){int x=new A}}";
    let p1 = fmt(src);
    let p2 = fmt(&p1);
    assert_eq!(p1, p2, "not idempotent: {p1}");
    let opens = p1.matches('{').count();
    let closes = p1.matches('}').count();
    assert_eq!(opens, closes, "unbalanced: {p1}");
}
