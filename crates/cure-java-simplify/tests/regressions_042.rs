//! 0.4.2 Java 后端缺陷回归（用户报告的 6 个最小用例）。
//!
//! 每条都做两件事：断言净化输出保持合法/等价形态；javac 可用时编译
//! 原始与净化两份源码，防止再次产出不可编译代码。

use std::fs;
use std::process::Command;

use cure_engine::Config;
use cure_java_parser::parse;
use cure_java_print::print_unit;
use cure_java_simplify::simplify_unit;

fn javac_available() -> bool {
    Command::new("javac")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn process(name: &str, src: &str) -> String {
    let mut outcome = parse(src);
    assert!(
        outcome.errors.is_empty(),
        "{name}: 源码解析失败：{:?}",
        outcome.errors
    );
    let _ = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let cleaned = print_unit(&outcome.ast, &outcome.unit);

    if javac_available() {
        let base = std::env::temp_dir().join(format!("cure_java_042_{name}"));
        let _ = fs::remove_dir_all(&base);
        let orig_dir = base.join("orig");
        let clean_dir = base.join("clean");
        fs::create_dir_all(&orig_dir).unwrap();
        fs::create_dir_all(&clean_dir).unwrap();
        let file = format!("{name}.java");
        fs::write(orig_dir.join(&file), src).unwrap();
        fs::write(clean_dir.join(&file), &cleaned).unwrap();
        for d in [&orig_dir, &clean_dir] {
            let out = Command::new("javac")
                .arg("-encoding")
                .arg("UTF-8")
                .arg("-nowarn")
                .arg("-d")
                .arg(d)
                .arg(d.join(&file))
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{name}: javac 失败\n{}\n== 净化输出 ==\n{cleaned}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
    cleaned
}

#[test]
fn nested_if_false_does_not_leave_parent_without_body() {
    let src = r#"
class Case1 {
    void f(boolean a) {
        if (a) if (false) System.out.println(1);
    }
}
"#;
    let out = process("Case1", src);
    assert!(!out.contains("if (false)"), "{out}");
    // 条件求值必须保留（a 为参数时整句可删；带副作用的形态单独由
    // case1b 验证）
    assert!(out.contains("void f(boolean a)"), "{out}");
}

#[test]
fn nested_if_false_keeps_side_effecting_condition() {
    let src = r#"
class Case1b {
    boolean side() { System.out.println("side"); return true; }
    void f() {
        if (side()) if (false) System.out.println(1);
    }
}
"#;
    let out = process("Case1b", src);
    assert!(out.contains("if (side())"), "{out}");
}

#[test]
fn inline_never_substitutes_into_incdec_or_assignment_target() {
    let src = r#"
class Case2 {
    static boolean test(int a) {
        return !((a++) != a);
    }
    public static void main(String[] args) {
        System.out.println(test(3));
    }
}
"#;
    let out = process("Case2", src);
    assert!(out.contains("test(3)"), "{out}");
    assert!(!out.contains("3++"), "{out}");
}

#[test]
fn null_is_not_propagated_into_synchronized_monitor() {
    let src = r#"
class Case3 {
    void f() {
        Object x = null;
        synchronized (x) {
            System.out.println(1);
        }
    }
}
"#;
    let out = process("Case3", src);
    assert!(out.contains("synchronized (x)"), "{out}");
    assert!(!out.contains("synchronized (null)"), "{out}");
}

#[test]
fn const_field_propagation_respects_lambda_and_local_shadowing() {
    let src = r##"
import java.util.List;
class Case4 {
    static final int s = 4;
    void f(List<String> xs) {
        xs.stream().filter(s -> !s.startsWith("#")).forEach(t -> System.out.println(t));
    }
}
"##;
    let out = process("Case4", src);
    assert!(out.contains("!s.startsWith(\"#\")"), "{out}");
    assert!(!out.contains("4.startsWith"), "{out}");

    // 局部变量 / catch / for-each 同名遮蔽同样不能替换
    let src = r#"
class Case4b {
    static final int n = 7;
    int f(java.util.List<Integer> xs) {
        int n = 3;
        for (int v : xs) {
            n += v;
        }
        try {
            return n;
        } catch (RuntimeException n2) {
            return n + n2.hashCode();
        }
    }
}
"#;
    let out = process("Case4b", src);
    assert!(out.contains("int n = 3;"), "{out}");
    assert!(!out.contains("int 7 ="), "{out}");
}

#[test]
fn box_unbox_chain_stays_a_statement_expression() {
    let src = r#"
class Case5 {
    void f(double d) {
        Integer.valueOf((int) d).intValue();
    }
}
"#;
    let out = process("Case5", src);
    assert!(
        out.contains("Integer.valueOf((int) d).intValue();"),
        "{out}"
    );
    assert!(!out.contains("(int) d;"), "{out}");
}

#[test]
fn annotation_string_arguments_keep_unicode_escapes() {
    // Rust 源码里写成 \\u0010，Java 源文本才是 \u0010 六字符转义。
    let src = "@SuppressWarnings(value = {\"a\\u0010b\", \"c\\0d\", \"e\\nf\"})\nclass Case6 {\n}\n";
    let out = process("Case6", src);
    assert!(out.contains("\\u0010"), "{out}");
    assert!(!out.chars().any(|c| c == '\u{10}'), "{out}");
    assert!(out.contains("\\0"), "{out}");
    assert!(out.contains("\\n"), "{out}");
}
#[test]
fn for_init_self_assign_is_kept() {
    let src = r#"
class Case7 {
    void f(int i, int n) {
        for (i = i; i < n; i++) {
            System.out.println(i);
        }
    }
}
"#;
    let out = process("Case7", src);
    assert!(out.contains("for (i = i; i < n; i++)"), "{out}");
}

#[test]
fn iterator_loop_to_for_each_requires_iterable_type() {
    let src = r#"
class Case8 {
    static class Bag {
        java.util.Iterator<String> iterator() {
            return java.util.List.of("x").iterator();
        }
    }
    void f(Bag bag) {
        for (java.util.Iterator<String> it = bag.iterator(); it.hasNext();) {
            String s = it.next();
            System.out.println(s);
        }
    }
}
"#;
    let out = process("Case8", src);
    assert!(out.contains("bag.iterator()"), "{out}");
    assert!(!out.contains("for (String s : bag)"), "{out}");
}

#[test]
fn propagation_does_not_narrow_assignment_target_base() {
    let src = r#"
class Case9 {
    void f(Object[] a, Object x) {
        Object[] result = a;
        result[0] = x;
    }
}
"#;
    let out = process("Case9", src);
    assert!(out.contains("Object[] result = a;"), "{out}");
    assert!(out.contains("result[0] = x;"), "{out}");
}

#[test]
fn const_field_not_propagated_into_switch_case_label() {
    let src = r#"
class Case10 {
    enum E { A, B }
    static final int A = 1;
    int f(E e) {
        switch (e) {
            case A: return 2;
            case B: return 3;
            default: return 0;
        }
    }
}
"#;
    let out = process("Case10", src);
    assert!(out.contains("case A:"), "{out}");
    assert!(!out.contains("case 1:"), "{out}");
}

#[test]
fn generic_call_not_propagated_into_receiver_when_typing_depends_on_target() {
    let src = r#"
import java.util.List;
import java.util.stream.Stream;
class Case11 {
    void f(List<String> fini) {
        Stream<Class<?>> cstream = fini.stream().map(String::getClass);
        Class<?> t = cstream.findFirst().orElse(void.class);
        System.out.println(t);
    }
}
"#;
    let out = process("Case11", src);
    assert!(out.contains("Stream<Class<?>> cstream"), "{out}");
    assert!(out.contains("cstream.findFirst()"), "{out}");
}

#[test]
fn generic_array_initializer_keeps_new() {
    let src = r#"
class Case12 {
    static class A<T> {}
    void f() {
        A<String>[] xs = new A[] {new A<String>()};
        System.out.println(xs.length);
    }
}
"#;
    let out = process("Case12", src);
    assert!(out.contains("new A[]"), "{out}");
}

#[test]
fn folded_int_literal_stays_compilable() {
    // `x -= 0x80000000` 合并进初始化/返回表达式后，字面量是 i32::MIN
    // （0x80000000），十进制 2147483648 不是合法 int 字面量。
    let src = r#"
class Case13 {
    int f(byte[] b) {
        int x = b[0];
        x -= 0x80000000;
        return x;
    }
}
"#;
    let out = process("Case13", src);
    assert!(!out.contains("2147483648"), "{out}");
    assert!(out.contains("0x80000000"), "{out}");
}

#[test]
fn flattened_else_keeps_declaration_scope() {
    let src = r#"
class Case14 {
    int f(int n) {
        if (n == 0) {
            return 0;
        } else {
            int seq = 1;
            System.out.println(seq);
            System.out.println(seq);
        }
        for (int i = 0; i < n; i++) {
            int seq = i;
            System.out.println(seq);
            System.out.println(seq);
        }
        return n;
    }
}
"#;
    let out = process("Case14", src);
    assert!(out.contains("int seq = 1;"), "{out}");
    assert!(out.contains("int seq = i;"), "{out}");
}

#[test]
fn try_resource_keeps_effectively_final_variable() {
    let src = r#"
import java.io.IOException;
import java.io.InputStream;
import java.util.Scanner;
import java.util.Set;
class Case15 {
    static Set<String> f(String module) throws IOException {
        InputStream filesIn = null;
        try {
            filesIn = open(module);
        } catch (IOException e) {
            throw new AssertionError("io");
        }
        if (filesIn == null) {
            return Set.of();
        }
        final InputStream in = filesIn;
        try (in; Scanner scanner = new Scanner(filesIn)) {
            return Set.of(scanner.nextLine());
        }
    }
    static InputStream open(String m) throws IOException {
        return null;
    }
}
"#;
    let out = process("Case15", src);
    assert!(out.contains("final InputStream in = filesIn;"), "{out}");
}

#[test]
fn diamond_not_moved_into_ternary_branch() {
    let src = r#"
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
class Case16 {
    List<String> f(Object[] array) {
        if (array == null) {
            return new ArrayList<>(0);
        }
        List<?> list = Arrays.asList(array);
        return (List<String>) list;
    }
}
"#;
    process("Case16", src);
}

#[test]
fn narrow_return_constant_expression_not_merged_into_ternary() {
    let src = r#"
class Case17 {
    static final short A = 1;
    static final short B = 2;
    short f(boolean c) {
        if (c) {
            return A | B;
        } else {
            return A;
        }
    }
}
"#;
    let out = process("Case17", src);
    assert!(out.contains("if (c)"), "{out}");
}

#[test]
fn while_iterator_nested_generic_keeps_element_type() {
    let src = r##"
import java.util.Iterator;
import java.util.Map;
import java.util.Set;
class Case18 {
    boolean f(Map<String, String> a, Map<String, String> b) {
        Set<Map.Entry<String, String>> entries = a.entrySet();
        Iterator<Map.Entry<String, String>> iterator = entries.iterator();
        Map.Entry<String, String> entry = null;
        while (iterator.hasNext()) {
            entry = iterator.next();
            if (!entry.getValue().equals(b.get(entry.getKey()))) {
                return false;
            }
        }
        return true;
    }
}
"##;
    let out = process("Case18", src);
    assert!(!out.contains("for (Object e"), "{out}");
}

#[test]
fn inline_does_not_rebind_this() {
    let src = r##"
class Case19 {
    boolean flag = true;
    String escape(String s) {
        return this.flag ? s : "";
    }
    static class Inner {
        String f(Case19 o) {
            return o.escape("x");
        }
    }
}
"##;
    let out = process("Case19", src);
    assert!(out.contains("o.escape(\"x\")"), "{out}");
}

#[test]
fn inline_null_argument_not_into_member_receiver() {
    let src = r#"
class Case20 {
    static String id(Object o) {
        return o == null ? "ID_NULL" : o.toString();
    }
    String f() {
        return id(null);
    }
}
"#;
    let out = process("Case20", src);
    assert!(out.contains("id(null)"), "{out}");
}

#[test]
fn const_true_throw_does_not_make_following_break_unreachable() {
    let src = r#"
class Case21 {
    void f(int n) {
        switch (n) {
            case 1:
                if (true) throw new RuntimeException();
                break;
            default:
                break;
        }
    }
}
"#;
    let out = process("Case21", src);
    assert!(out.contains("if (true)"), "{out}");
}
