//! 差分验证（黄金标准）：对每个测试程序——
//!   原版.java ──javac──▶ 运行 ──▶ stdout/exit
//!   简化版.java ──javac──▶ 运行 ──▶ stdout/exit   ⇒ 两者必须完全一致
//!
//! 本机无 javac 时自动跳过。

use std::fs;
use std::path::Path;
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

fn compile_and_run(dir: &Path, class: &str) -> (String, Option<i32>) {
    let run = Command::new("java")
        .arg("-cp")
        .arg(dir)
        .arg(class)
        .output()
        .expect("run java");
    let out = String::from_utf8_lossy(&run.stdout).to_string();
    let err = String::from_utf8_lossy(&run.stderr).to_string();
    // 异常时的可比对输出：退出码 + stderr 里该异常的类名/消息行
    let err_sig = err
        .lines()
        .filter(|l| l.starts_with("Exception in thread") || l.contains("Caused by"))
        .map(|l| l.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    (format!("{out}{err_sig}"), run.status.code())
}

fn differential(name: &str, src: &str) {
    if !javac_available() {
        eprintln!("skip differential: javac not found");
        return;
    }
    let base = std::env::temp_dir().join(format!("cure_diff_{name}"));
    let _ = fs::remove_dir_all(&base);
    let orig_dir = base.join("orig");
    let simp_dir = base.join("simp");
    fs::create_dir_all(&orig_dir).unwrap();
    fs::create_dir_all(&simp_dir).unwrap();

    // 原版
    let orig_file = orig_dir.join(format!("{name}.java"));
    fs::write(&orig_file, src).unwrap();

    // 简化版
    let mut outcome = parse(src);
    assert!(
        outcome.errors.is_empty(),
        "{name}: 测试源码本身必须能干净解析：{:?}",
        outcome.errors
    );
    let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let simplified = print_unit(&outcome.ast, &outcome.unit);
    let simp_file = simp_dir.join(format!("{name}.java"));
    fs::write(&simp_file, &simplified).unwrap();

    for (d, f) in [(&orig_dir, &orig_file), (&simp_dir, &simp_file)] {
        let out = Command::new("javac")
            .arg("-nowarn")
            .arg("-d")
            .arg(d)
            .arg(f)
            .output()
            .expect("run javac");
        assert!(
            out.status.success(),
            "{name}: javac 失败\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let (a_out, a_code) = compile_and_run(&orig_dir, name);
    let (b_out, b_code) = compile_and_run(&simp_dir, name);
    assert_eq!(
        a_code, b_code,
        "{name}: 退出码不一致（原 {a_code:?} vs 简 {b_code:?}）\n原版输出:\n{a_out}\n简化版输出:\n{b_out}"
    );
    assert_eq!(
        a_out, b_out,
        "{name}: 输出不一致！\n== 原版 ==\n{a_out}\n== 简化版 ==\n{b_out}\n== 简化后源码 ==\n{simplified}"
    );
    eprintln!(
        "differential {name}: OK（{} 次改写，输出 {} 字节一致）",
        report.edits,
        a_out.len()
    );
}

#[test]
fn diff_propagation_and_calls() {
    differential(
        "DiffProp",
        r#"
public class DiffProp {
    static int side = 0;
    static int inc() { side++; return side; }
    public static void main(String[] args) {
        int a = inc();
        int b = a;
        System.out.println(b);
        System.out.println(side);
        int c = inc();
        System.out.println(c + a + b);
    }
}
"#,
    );
}

#[test]
fn diff_short_circuit_effects() {
    differential(
        "DiffShort",
        r#"
public class DiffShort {
    static int n = 0;
    static boolean t() { n += 10; return true; }
    static boolean f() { n += 100; return false; }
    static boolean id(boolean x) { n += 1; return x; }
    public static void main(String[] args) {
        System.out.println(id(false) && t());
        System.out.println(id(true) || f());
        System.out.println(true && t());
        System.out.println(false || f());
        System.out.println(id(true) && true);
        System.out.println(id(false) || false);
        System.out.println(n);
    }
}
"#,
    );
}

#[test]
fn diff_overflow_and_folding() {
    differential(
        "DiffFold",
        r#"
public class DiffFold {
    public static void main(String[] args) {
        System.out.println(1 + 2);
        System.out.println(2 * 21);
        System.out.println(1 << 3);
        System.out.println(1 << 32);
        System.out.println(2147483647 + 1);
        System.out.println(-2147483648 - 1);
        System.out.println(9223372036854775807L + 1L);
        System.out.println(6 & 3);
        System.out.println(6 | 3);
        System.out.println(6 ^ 3);
        System.out.println(100 / 7);
        System.out.println(100 % 7);
        System.out.println(1 + 2 * 3 - 4 / 2);
    }
}
"#,
    );
}

#[test]
fn div_by_zero_exception() {
    differential(
        "DiffDiv",
        r#"
public class DiffDiv {
    public static void main(String[] args) {
        try {
            int x = 10 / 0;
            System.out.println(x);
        } catch (ArithmeticException e) {
            System.out.println("caught: " + e.getMessage());
        }
        try {
            System.out.println(10 % 0);
        } catch (ArithmeticException e) {
            System.out.println("caught: " + e.getMessage());
        }
    }
}
"#,
    );
}

#[test]
fn diff_nan_and_floats() {
    differential(
        "DiffFloat",
        r#"
public class DiffFloat {
    public static void main(String[] args) {
        double nan = Double.NaN;
        System.out.println(0 * nan);
        System.out.println(nan < 1.0);
        System.out.println(!(nan < 1.0));
        System.out.println(nan >= 1.0);
        System.out.println(nan == nan);
        System.out.println(nan != nan);
        double negZero = -0.0;
        System.out.println(1 / negZero == 1 / 0.0);
        System.out.println(negZero == 0.0);
        float f = 0.0f;
        System.out.println(f * 0.0f);
    }
}
"#,
    );
}

#[test]
fn diff_string_builder() {
    differential(
        "DiffSb",
        r#"
public class DiffSb {
    public static void main(String[] args) {
        int a = 5;
        String b = "x";
        Object nil = null;
        System.out.println(new StringBuilder().append(a).append(b).toString());
        System.out.println(new StringBuilder().append(b).append(a).toString());
        System.out.println(new StringBuilder("head").append(1).append("tail").toString());
        System.out.println(new StringBuilder().append(nil).toString());
        System.out.println(new StringBuilder().append('c').append(true).toString());
        System.out.println(new StringBuilder().append(1).append(2).toString());
        StringBuilder keep = new StringBuilder(16);
        keep.append("kept");
        System.out.println(keep.toString());
    }
}
"#,
    );
}

#[test]
fn diff_iterator_loop() {
    differential(
        "DiffIter",
        r#"
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

public class DiffIter {
    public static void main(String[] args) {
        List<String> list = new ArrayList<>();
        list.add("a");
        list.add("bb");
        list.add("ccc");
        int total = 0;
        for (Iterator<String> it = list.iterator(); it.hasNext(); ) {
            String s = it.next();
            total += s.length();
        }
        System.out.println(total);
        int count = 0;
        for (Iterator<String> it = list.iterator(); it.hasNext(); ) {
            String s = it.next();
            count++;
            if (s.equals("bb")) break;
        }
        System.out.println(count);
    }
}
"#,
    );
}

#[test]
fn diff_box_unbox() {
    differential(
        "DiffBox",
        r#"
public class DiffBox {
    public static void main(String[] args) {
        System.out.println(Integer.valueOf(42).intValue());
        System.out.println(Long.valueOf(9L).longValue());
        System.out.println(Boolean.valueOf(true).booleanValue());
        System.out.println(Double.valueOf(2.5).doubleValue());
        System.out.println(Character.valueOf('Z').charValue());
        System.out.println(Integer.valueOf("99").intValue());
    }
}
"#,
    );
}

#[test]
fn diff_boolean_and_ternary() {
    differential(
        "DiffBool",
        r#"
public class DiffBool {
    static boolean flag(int x) { return x > 10; }
    static int pick(boolean c) { return c ? 1 : 0; }
    public static void main(String[] args) {
        int x = 15;
        boolean b = x > 10;
        if (b) {
            System.out.println("big");
        } else {
            System.out.println("small");
        }
        System.out.println(flag(x));
        System.out.println(pick(flag(x)));
        System.out.println(x > 0 ? x : -x);
        System.out.println(!flag(x));
        System.out.println(flag(1) == true);
        System.out.println(flag(1) != false);
        System.out.println(!!flag(x));
        System.out.println(true ? "a" : "b");
    }
}
"#,
    );
}

#[test]
fn diff_dead_store_and_shadow() {
    differential(
        "DiffDead",
        r#"
public class DiffDead {
    static int counter = 0;
    static int bump() { return ++counter; }
    public static void main(String[] args) {
        int x = 1;
        x = 2;
        System.out.println(x);
        int y = bump();
        y = 5;
        System.out.println(y + "," + counter);
        int z = 3;
        {
            int z2 = 4;
            z = z2;
        }
        System.out.println(z);
        int w = 10;
        w = w + 1;
        System.out.println(w);
        int v = 7;
        int v2 = v;
        System.out.println(v + v2);
    }
}
"#,
    );
}

#[test]
fn diff_cast_instanceof() {
    differential(
        "DiffCast",
        r#"
public class DiffCast {
    public static void main(String[] args) {
        Object o = "hello";
        String s = (String) o;
        System.out.println(s.length());
        System.out.println(((String) o).charAt(1));
        System.out.println(o instanceof String);
        Object n = Integer.valueOf(3);
        System.out.println(n instanceof Integer);
        try {
            String bad = (String) n;
            System.out.println(bad);
        } catch (ClassCastException e) {
            System.out.println("CCE");
        }
    }
}
"#,
    );
}

#[test]
fn diff_control_flow_and_try() {
    differential(
        "DiffFlow",
        r#"
public class DiffFlow {
    static String log = "";
    static void a() { log += "a"; }
    static void b() { log += "b"; }
    static void c() { log += "c"; throw new RuntimeException("boom"); }
    public static void main(String[] args) {
        int x = 5;
        if (x > 0) { a(); } else { b(); }
        if (x < 0) { b(); }
        while (x > 0) { c(); x--; }
        try {
            System.out.println("unreachable");
        } catch (RuntimeException e) {
            System.out.println(e.getMessage());
        } finally {
            System.out.println(log);
        }
        System.out.println(x);
    }
}
"#,
    );
}

#[test]
fn diff_assign_propagation_and_copies() {
    differential(
        "DiffCopy",
        r#"
public class DiffCopy {
    static int side = 0;
    static int next() { side += 3; return side; }
    public static void main(String[] args) {
        // 拆分声明 + 拷贝链
        int p;
        p = 11;
        int q;
        q = p;
        int w;
        w = q;
        System.out.println(w);
        // 有副作用的赋值传播（求值顺序敏感：next 先于 println 前的 f）
        int t = 0;
        t = next();
        int u = t;
        System.out.println(u);
        System.out.println(u + w);
        // 死赋值 + 重赋值
        int x = 5;
        x = 7;
        System.out.println(x);
        // valueOf + 拼接
        String v = String.valueOf(42) + "x";
        String m = "a" + v + "b" + "c";
        System.out.println(m);
        System.out.println(side);
    }
}
"#,
    );
}

#[test]
fn diff_string_concat_and_valueof() {
    differential(
        "DiffStr",
        r#"
public class DiffStr {
    public static void main(String[] args) {
        String a = "a";
        String s1 = "x" + a + "y" + "z";
        String s2 = "p" + "q" + a;
        String s3 = String.valueOf(9) + "-";
        String s4 = "-" + String.valueOf(8);
        Object nil = null;
        String s5 = String.valueOf(nil) + "!";
        System.out.println(s1);
        System.out.println(s2);
        System.out.println(s3);
        System.out.println(s4);
        System.out.println(s5);
        int n = 5;
        String s6 = String.valueOf(n) + "u";
        System.out.println(s6);
    }
}
"#,
    );
}

// ---------------------------------------------------------------------------
// 保守性专项：不可证明语义的场景【必须不动】——正确性的一半是"知道何时不该动"
// ---------------------------------------------------------------------------

#[test]
fn conservative_float_nan() {
    // NaN 比较取反在浮点下不成立（!(a<b)=true 但 a>=b=false）
    differential(
        "ConsNaN",
        r#"
public class ConsNaN {
    public static void main(String[] args) {
        double nan = 0.0 / 0.0;
        System.out.println(nan < 1.0);
        System.out.println(!(nan < 1.0));
        System.out.println(nan >= 1.0);
        System.out.println(nan == nan);
        System.out.println(nan != nan);
        double z = 0.0;
        System.out.println(0 * nan);
        System.out.println(z == -z);
        System.out.println(1 / z == Double.POSITIVE_INFINITY);
        float f = 0.0f;
        System.out.println(0 * f);
        System.out.println(-z);
    }
}
"#,
    );
}

#[test]
fn conservative_overflow_and_div() {
    // 整数回绕与除零异常语义
    differential(
        "ConsOvf",
        r#"
public class ConsOvf {
    public static void main(String[] args) {
        System.out.println(Integer.MAX_VALUE + 1);
        System.out.println(Integer.MIN_VALUE - 1);
        System.out.println(Integer.MIN_VALUE / -1);
        System.out.println(-Integer.MIN_VALUE);
        System.out.println(Long.MAX_VALUE + 1L);
        try {
            System.out.println(1 / 0);
        } catch (ArithmeticException e) {
            System.out.println("div0: " + e.getMessage());
        }
        try {
            System.out.println(5 % 0);
        } catch (ArithmeticException e) {
            System.out.println("mod0: " + e.getMessage());
        }
        System.out.println(1 << 32);
        System.out.println(1L << 64);
        int i = 200;
        byte b = (byte) i;
        System.out.println(b);
    }
}
"#,
    );
}

#[test]
fn conservative_string_semantics() {
    // 字符串池化/引用比较/locale 敏感方法
    differential(
        "ConsStr",
        r#"
public class ConsStr {
    public static void main(String[] args) {
        // == 是引用比较（字面量池化后恰好 true，但不可证明等价于 equals）
        String a = "he" + "llo";
        String b = "hello";
        System.out.println(a == b);
        String c = new String("hello");
        System.out.println(c == b);
        System.out.println(c.equals(b));
        // intern 改变引用语义
        System.out.println(c.intern() == b);
        // locale 敏感方法不折
        String s = "abc";
        System.out.println(s.toUpperCase());
        System.out.println("i".toUpperCase());
        String turkish = "I";
        System.out.println(turkish.toLowerCase());
        // hashCode 确定性可折 ✓（JLS 规定）
        System.out.println("hello".hashCode());
    }
}
"#,
    );
}

#[test]
fn conservative_local_array_identity() {
    // 有 string == 时守卫拦截数组字面量下标折叠；传播+打印必须仍产出合法 Java
    differential(
        "ConsArr",
        r#"
public class ConsArr {
    public static void main(String[] args) {
        int i = 1;
        String[] table = {"zero", "one", "two"};
        String f = table[i];
        System.out.println(f);
        String g = "one";
        System.out.println(f == g);
        String h = new String("one");
        System.out.println(h == g);
    }
    static String pick() {
        // 无 == 上下文：数组下标照常折叠
        String[] t2 = {"a", "b"};
        return t2[1];
    }
}
"#,
    );
}

#[test]
fn conservative_string_creation_identity() {
    // 折叠产出「池化常量」 vs 原语义「运行期新建 String」——== 必须保持 false
    differential(
        "ConsStr2",
        r#"
public class ConsStr2 {
    static final String[] T = {"a", "b"};
    public static void main(String[] args) {
        String b = "hello";
        // sb.toString() 常量链：运行期新建 → == false
        String a = new StringBuilder().append("he").append("llo").toString();
        System.out.println(a == b);
        System.out.println(a.equals(b));
        // new String(Base64 解码)：运行期新建 → == false
        String c = new String(java.util.Base64.getDecoder().decode("aGVsbG8="));
        System.out.println(c == b);
        System.out.println(c.equals(b));
        // valueOf 剥壳后常量化：valueOf 调用阻断常量折叠 → == false
        System.out.println(String.valueOf("he") + "llo" == b);
        System.out.println((String.valueOf("he") + "llo").equals(b));
        // toString/valueOf 非常量化折叠：运行期新建 → == false
        System.out.println(Integer.toString(5) == "5");
        System.out.println(String.valueOf('x') == "x");
        // new String(lit)：经典引用身份
        String d = new String("hello");
        System.out.println(d == b);
        // new String()：与 "" 也是不同对象
        System.out.println(new String() == "");
        // 字面量数组的元素本来就是池化引用 → == true（折下标不改变语义）
        System.out.println(T[0] == "a");
        System.out.println(T[1] == "b");
    }
}
"#,
    );
}

#[test]
fn conservative_types_and_boxes() {
    // 装箱身份/窄化/char 溢出
    differential(
        "ConsBox",
        r#"
public class ConsBox {
    public static void main(String[] args) {
        Integer a = 127;
        Integer b = 127;
        System.out.println(a == b);
        Integer c = 128;
        Integer d = 128;
        System.out.println(c == d);
        System.out.println(c.equals(d));
        // Integer.valueOf("99") 是解析——不能与 99 视为同一字面量处理
        System.out.println(Integer.valueOf("99").intValue());
        Object o = "str";
        System.out.println(o instanceof String);
        Object n = Integer.valueOf(3);
        System.out.println(n instanceof String);
        char ch = 'a';
        int i = ch + 1;
        System.out.println((char) i);
        // 混合类型比较
        long l = 5;
        System.out.println(l == 5);
        double dv = 5.0;
        System.out.println(dv == 5);
        System.out.println(0.1 + 0.2 == 0.3);
    }
}
"#,
    );
}
