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
    assert!(out.contains("int[] xs = new int[] {1, 2, 3};"), "{out}");
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
