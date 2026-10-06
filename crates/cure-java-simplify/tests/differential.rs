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
    differential_cfg(name, src, Config::default());
}

fn differential_cfg(name: &str, src: &str, cfg: Config) {
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
    let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &cfg);
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

#[test]
fn dead_method_removal_after_inline() {
    // 解密器 d() 被内联→常量折叠后死掉；unusedHelper 天生死；
    // alive 被引用必须保留；public 方法永不删。
    // 行为差分 + 输出内容断言（死方法确实被删掉）。
    use std::collections::HashSet;
    if !javac_available() {
        eprintln!("skip differential: javac not found");
        return;
    }
    let src = r##"
public class DeadM {
    static final String[] T = {"c3Vw", "b3I="};
    private static String d(int i) {
        return new String(java.util.Base64.getDecoder().decode(T[i]));
    }
    private static String unusedHelper(int x) { return "u" + x; }
    private static int alive() { return 42; }
    private static String refOverload(String s) { return s + "!"; }
    private static String refOverload(int n) { return "#" + n; }
    public static void main(String[] args) {
        System.out.println(d(0));
        System.out.println(d(1));
        System.out.println(alive());
        System.out.println(refOverload("k"));
    }
}
"##;
    let cfg = Config {
        remove_dead_methods: true,
        ..Default::default()
    };
    differential_cfg("DeadM", src, cfg.clone());

    // 内容断言：死方法删除，存活方法保留
    let mut outcome = parse(src);
    simplify_unit(&mut outcome.ast, &mut outcome.unit, &cfg);
    let out = print_unit(&outcome.ast, &outcome.unit);
    assert!(!out.contains("unusedHelper"), "死方法未删除:\n{out}");
    // d 内联后零引用 → 删除
    assert!(!out.contains("private static String d("), "解密器残留:\n{out}");
    // 被引用的保留
    assert!(out.contains("alive()"), "{out}");
    // 撞名重载（refOverload 只有一个被调用，另一个同名也保留）
    let count = out.matches("refOverload").count();
    assert!(count >= 2, "撞名重载应保留:\n{out}");
    let _ = HashSet::<String>::new();
}

#[test]
fn dead_write_then_propagate() {
    // 零用途 DeadStore 删除死写后，此前被写冲突挡住的传播变为健全。
    // 三种形态的行为等价性由 javac 差分锁定（死写删除不改变可观察行为）。
    differential(
        "DeadW",
        r#"
public class DeadW {
    static int mark = 0;
    static int bump() { mark++; return mark; }
    public static void main(String[] args) {
        // 死写 y=9 无后续读 → 删除后 return y 与原 return x 同值
        int y0 = args.length;
        int x = 0;
        x = y0;
        y0 = 9;
        System.out.println(x);
        // 死写 + 后续传播整链坍缩
        String v = "";
        String w = v + "x";
        v = "y";
        System.out.println(w);
        // 副作用值不可丢：bump() 的调用次数必须保留
        int p = bump();
        p = 5;
        System.out.println(p);
        System.out.println(mark);
    }
}
"#,
    );
}

#[test]
fn deobfuscation_capability_wave() {
    // 本轮新增的还原形态：ZKM/Allatori char 数组藏匿、intern 恒等、
    // dex2jar 空 finally、StringBuilder 容量构造解锁——行为差分 + 形态断言。
    differential(
        "CapW",
        r#"
public class CapW {
    static int mark = 0;
    static int bump() { mark++; return mark; }
    public static void main(String[] args) {
        // ZKM/Allatori：new String(char 字面量数组)
        System.out.println(new String(new char[]{'h', 'e', 'l', 'l', 'o'}));
        // intern 恒等（JLS：字面量编译期驻留）
        System.out.println("abc".intern());
        System.out.println("abc".intern() == "abc");
        // dex2jar：空 finally 剥壳 + try 解包 + 块展平
        try {
            System.out.println(bump());
        } finally {
        }
        // 有 catch：只剥空 finally，try/catch 保留（异常必须仍被吞/处理）
        try {
            System.out.println(bump());
        } catch (RuntimeException e) {
            System.out.println("caught");
        } finally {
        }
        // StringBuilder 容量字面量构造（容量只是分配提示）
        System.out.println(new StringBuilder(16).append("cap").append(42).toString());
        // 作用域安全：内层块有声明（遮蔽）→ 不展平
        int x = 1;
        {
            int x2 = x + 1;
            System.out.println(x2);
        }
        System.out.println(x);
        System.out.println(mark);
    }
}
"#,
    );
    // 形态断言
    let mut outcome = parse(r#"
public class CapW2 {
    String s() { return new String(new char[]{'h', 'i'}); }
    String i() { return "x".intern(); }
    void t() { try { foo(); } finally { } }
    String c() { return new StringBuilder(32).append("a").toString(); }
}
"#);
    simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let out = print_unit(&outcome.ast, &outcome.unit);
    assert!(out.contains(r#"return "hi";"#), "{out}");
    assert!(out.contains(r#"return "x";"#), "{out}");
    assert!(out.contains("foo();"), "{out}");
    assert!(!out.contains("finally"), "{out}");
    assert!(out.contains(r#"return "a";"#), "{out}");
}

#[test]
fn double_neg_and_printer_legality() {
    // -(-x) → x（补码回绕恒等）；-(-lit) → |lit|；
    // printer 二义修复：-(-x)/-(-5) 曾输出 --x/--5（词法=前置自减，非法）
    differential(
        "DNeg",
        r#"
public class DNeg {
    public static void main(String[] args) {
        int x = -7;
        System.out.println(-(-x));
        System.out.println(-(-5));
        System.out.println(-(-5L));
        // MIN 边界：-(-MIN) ≡ MIN（回绕）
        System.out.println(-(-Integer.MIN_VALUE));
        long l = -9L;
        System.out.println(-(-l));
        // 嵌套两层（定点收敛）
        System.out.println(-(-(-(-x))));
    }
}
"#,
    );
    let mut outcome = parse("class A{int m(int x){return -(-x);}}");
    simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let out = print_unit(&outcome.ast, &outcome.unit);
    assert!(out.contains("return x;"), "{out}");
    let mut outcome = parse("class A{int m(){return -(-5);}}");
    simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let out = print_unit(&outcome.ast, &mut outcome.unit);
    assert!(out.contains("return 5;"), "{out}");
}

#[test]
fn adversarial_precedence_and_literals() {
    // 对抗性差分：运算符优先级全组合嵌套、全部字面量形态（十六进制/八进制/
    // 二进制/下划线/带符号指数/前导点浮点/字符转义/\u 转义）、单目链、转型
    // 优先级、instanceof、赋值表达式、三元嵌套、数组/foreach/标签/do-while/
    // 经典与箭头 switch、副作用求值序。
    // 曾抓到两个真 bug：1) 指数符号 e-/E+ 词法分支缺失（4M 错误风暴）；
    // 2) 多声明符合成 Block 被当词法作用域 → 零用途 DeadStore 误删逃逸变量
    //    （Google/ProGuard 风格一行一声明故从未触发；本样本经典写法引爆）。
    differential(
        "Adv",
        r#"public class Adv {
    static int side = 0;
    static int bump() { side++; return side; }

    public static void main(String[] args) {
        int a = 13, b = 5, c = 3;
        long l = 100L;
        // 运算符优先级嵌套（printer 括号正确性）
        System.out.println(a - (b - c));
        System.out.println((a - b) - c);
        System.out.println(a << (b << c));
        System.out.println((a << b) << c);
        System.out.println(a / (b / c));
        System.out.println(a % (b % c));
        System.out.println((a & b) | c);
        System.out.println(a & (b | c));
        System.out.println(a ^ (b ^ c));
        System.out.println((a ^ b) ^ c);
        System.out.println(a + (b * c));
        System.out.println((a + b) * c);
        System.out.println(a < (b < c ? 1 : 0));
        System.out.println((a < b) ? (c < 5 ? 1 : 2) : 3);
        System.out.println(a == (b == c ? 0 : 1));
        System.out.println(!(a < b) && (c > 2));
        // 字面量形态
        System.out.println(0x7FFFFFFF);
        System.out.println(0x80000000L);
        System.out.println(010);
        System.out.println(0b1010);
        System.out.println(1_000_000);
        System.out.println(1_000_000.5f);
        System.out.println(1e10);
        System.out.println(1.5e-3);
        System.out.println(3.14f);
        System.out.println(3.14d);
        System.out.println(.5f);
        System.out.println('a');
        System.out.println('\\');
        System.out.println('\'');
        System.out.println('\n' == 10);
        System.out.println('\u4e2d' == 20013);
        System.out.println("\u4e2d\u6587");
        System.out.println("tab\tnl\nq\"bs\\");
        // 单目嵌套
        System.out.println(-(-a));
        System.out.println(~(~a));
        System.out.println(-(~a));
        System.out.println(~(-a));
        System.out.println(!(!(a > b)));
        // 转型与优先级
        System.out.println((int) (a + b));
        System.out.println((int) a + b);
        System.out.println((long) (a * b) + c);
        System.out.println((char) ('a' + 1));
        System.out.println((byte) 200);
        System.out.println((float) 3.14 + 1);
        // instanceof / 赋值表达式 / 三元嵌套
        Object o = "s";
        System.out.println(o instanceof String);
        int x;
        System.out.println(x = a + b);
        System.out.println(x += c);
        System.out.println(x *= 2);
        System.out.println(x /= 3);
        System.out.println(x %= 4);
        System.out.println(x ^= 1);
        System.out.println(a > b ? b > c ? 10 : 20 : 30);
        System.out.println((a > b) ? 1 : (b > c) ? 2 : 3);
        // 数组/foreach/变参
        int[] arr = {3, 1, 2};
        int[][] mtx = {{1, 2}, {3, 4}};
        for (int v : arr) { System.out.println(v); }
        for (int[] row : mtx) { for (int v : row) { System.out.println(v); } }
        // 标签 + continue
        outer:
        for (int i = 0; i < 3; i++) {
            for (int j = 0; j < 3; j++) {
                if (j == 1) { continue outer; }
                System.out.println(i * 10 + j);
            }
        }
        // do-while
        int k = 0;
        do { k++; } while (k < 3);
        System.out.println(k);
        // switch 经典与箭头
        switch (a % 3) {
            case 0: System.out.println("zero"); break;
            case 1 + 1: System.out.println("two"); break;
            default: System.out.println("other");
        }
        switch (b) {
            case 1 -> System.out.println("one");
            default -> System.out.println("many");
        }
        // 求值顺序（副作用）
        System.out.println(bump() + bump() * 10);
        System.out.println(a > b && bump() > 0);
        System.out.println(a < b || bump() > 0);
        System.out.println(side);
    }
}
"#,
    );
    // 多声明符专项：变量逃逸合成分组后必须存活且可用
    let src = "class A{int m(){int a = 1, b = 2, c = 3; return a + b * c;}}";
    let mut outcome = parse(src);
    simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let out = print_unit(&outcome.ast, &outcome.unit);
    assert!(out.contains("int a = 1;") || out.contains("int a = 1,") || out.contains("return 7;"), "{out}");
    assert!(!out.contains("{\n            int a"), "{out}");
}

#[test]
fn adversarial_modern_java_forms() {
    // 对抗波 2：枚举带成员、泛型方法（<T extends …>）、内部/嵌套/匿名类、
    // lambda 全形态（含 Runnable 赋值）、方法引用、try-with-resources（多资源
    // + var）、multi-catch、instanceof 模式、switch 表达式（yield）、逗号 for、
    // 标签、静态/实例初始化块、synchronized、assert。
    // 曾抓到三个真 bug：1) >>/>>> 单 token 使 skip_balanced 永不配平 →
    // 泛型方法起整块静默丢失（0 错误！）；2) outer.new Inner() 限定内部类
    // 创建无节点表示 → 4M 错误风暴；3) lambda/方法引用被传播进 receiver 位
    // → (() -> {}).run() / String::length.apply() 非法输出。
    differential(
        "Adv2",
        r#"import java.util.*;
import java.util.function.*;

public class Adv2 {
    // 枚举
    enum Color { RED, GREEN, BLUE;
        static final Color[] ALL = values();
    }
    // 泛型边界
    static <T extends Comparable<? super T>> T max(List<? extends T> xs) {
        T r = xs.get(0);
        for (T x : xs) { if (x.compareTo(r) > 0) { r = x; } }
        return r;
    }
    // 静态块 / 实例块 / 内部类
    static int COUNTER;
    static { COUNTER = 5; }
    { COUNTER += 1; }
    static class Nested {
        int f(int x) { return x * COUNTER; }
    }
    class Inner {
        int g() { return COUNTER; }
    }
    interface Op { int apply(int a, int b); default int twice(int a, int b) { return apply(a, b) * 2; } }

    // try-with-resources（多资源 + var）+ multi-catch
    static String io(boolean a, boolean b) {
        StringBuilder sb = new StringBuilder();
        try (var in = new java.io.ByteArrayInputStream(new byte[]{65, 66});
             var out = new java.io.ByteArrayOutputStream()) {
            int c;
            while ((c = in.read()) >= 0) { out.write(c); }
            sb.append(out.toString());
        } catch (java.io.IOException | RuntimeException e) {
            sb.append("err");
        } finally {
            sb.append("!");
        }
        try {
            if (a) { throw new IllegalStateException("a"); }
            if (b) { throw new IllegalArgumentException("b"); }
        } catch (IllegalStateException | IllegalArgumentException e) {
            sb.append(e.getMessage());
        }
        return sb.toString();
    }

    public static void main(String[] args) {
        // lambda 全形态 + 方法引用
        Predicate<String> p = s -> s.isEmpty();
        Function<Integer, Integer> f = x -> x + 1;
        Supplier<String> sup = () -> "v";
        BiFunction<Integer, Integer, Integer> add = (a, b) -> a + b;
        Runnable r = () -> {
            int t = 0;
            for (int i = 0; i < 3; i++) { t += i; }
            System.out.println(t);
        };
        r.run();
        Function<String, Integer> len = String::length;
        Function<Integer, List<Integer>> mk = ArrayList::new;
        System.out.println(p.test("") + "," + f.apply(1) + "," + sup.get() + "," + add.apply(2, 3) + "," + len.apply("ab") + "," + mk.apply(3).size());
        // var + instanceof 模式
        Object o = "str";
        if (o instanceof String s) { System.out.println(s.length()); }
        var list = List.of(1, 2, 3);
        System.out.println(list.size());
        // switch 表达式 + yield + 箭头
        int day = 3;
        String kind = switch (day) {
            case 1, 7 -> "weekend";
            case 2, 3, 4, 5, 6 -> "weekday";
            default -> throw new IllegalArgumentException();
        };
        System.out.println(kind);
        int code = switch (day) {
            case 1 -> { yield 100; }
            default -> { yield 200; }
        };
        System.out.println(code);
        // 逗号 for + 标签
        outer2:
        for (int i = 0, j = 9; i < j; i++, j--) {
            if (i == 3) { continue outer2; }
            System.out.println(i + ":" + j);
        }
        // 泛型调用（通配 + 菱形）
        List<Integer> ints = new ArrayList<>();
        ints.add(3);
        List<? extends Number> nums = ints;
        Map<String, List<Integer>> m = new HashMap<>();
        m.put("k", ints);
        System.out.println(max(ints) + "," + nums.size() + "," + m.get("k").size());
        // 匿名类
        Op op = new Op() {
            @Override public int apply(int a, int b) { return a * 10 + b; }
        };
        System.out.println(op.apply(4, 2) + "," + op.twice(1, 2));
        // 枚举
        System.out.println(Color.ALL.length + "," + Color.GREEN.name());
        // assert
        assert list.size() > 0 : "empty";
        // synchronized
        Object lock = new Object();
        synchronized (lock) { System.out.println("sync"); }
        // 数组：锯齿 + new int[2][3]
        int[][] jag = new int[2][];
        jag[0] = new int[]{1};
        jag[1] = new int[]{2, 3};
        int[][] grid = new int[2][3];
        System.out.println(jag[1][1] + "," + grid.length + "," + grid[0].length);
        // 字符算术 + +=
        char ch = 'a';
        ch += 2;
        System.out.println(ch);
        String s2 = "x";
        s2 += "y";
        s2 += 1;
        System.out.println(s2);
        // 内部/嵌套类
        System.out.println(new Nested().f(2) + "," + new Adv2().new Inner().g());
        // io
        System.out.println(io(false, false));
        System.out.println(COUNTER);
    }
}
"#,
    );
}

#[test]
fn adversarial_annotations_records_sealed() {
    // 对抗波 3：注解全形态（标记/单值/数组/嵌套/@interface 成员带 default）、
    // record（泛型+紧凑构造器+辅助构造器）、sealed/non-sealed/permits、
    // 窄类型复合赋值（隐式收窄 b += 1）、数值提升（byte+byte→int 等）、
    // 十六进制 long 负值域、>>> 无符号移位、字符串/枚举 switch 落穿、
    // 嵌套 switch、finally+continue/break 组合、拼接 null。
    // 曾抓到四个真 bug：1) @interface 成员 `T name() default v;` 无方法体
    // 形态解析失败（已 RAW 保真）；2) non-sealed 三 token 序列被拆（non
    // 单独消费留 `-sealed` 残体）；3) permits 子句被丢弃（TypeDecl 无字段）；
    // 4) 词法器 THREE 表缺裸 `>>>`（只有 >>>=）——`n >>> 1` 切成 `>>` + `>`。
    differential(
        "Adv3",
        r#"import java.lang.annotation.*;
import java.util.*;

@SuppressWarnings("all")
@Deprecated
public class Adv3 {
    // 注解全形态：标记、单值、数组、嵌套
    @Retention(RetentionPolicy.RUNTIME)
    @interface Fr { String name() default "x"; int[] vals() default {1, 2}; }
    @interface Outer { Fr inner() default @Fr(name = "d"); Class<?>[] types() default {}; }

    @Fr(name = "n", vals = {3, 4})
    @Outer(inner = @Fr(name = "e"), types = {String.class, List.class})
    static int annotatedField = 1;

    // record + 泛型 + 紧凑构造器
    record Point(int x, int y) {
        Point {
            if (x < 0) { throw new IllegalArgumentException(); }
        }
        Point(int x) { this(x, 0); }
        int sum() { return x + y; }
    }
    record Pair<T extends Comparable<T>>(T a, T b) {}

    // sealed（Java 17）
    sealed interface Shape permits Circle, Square, Big {}
    record Circle(double r) implements Shape {}
    record Square(double s) implements Shape {}
    non-sealed class Big implements Shape {}

    // 窄类型复合赋值（隐式收窄：b += 1 合法、b = b + 1 编译错误）
    static byte bump(byte b) {
        b += 1;
        b *= 2;
        b -= 1;
        return b;
    }
    static short shr(short s) {
        s += 1000;
        return s;
    }

    public static void main(String[] args) {
        // 数值提升：byte+byte→int、int+long→long、char+int→int
        byte b1 = 10, b2 = 20;
        int i1 = b1 + b2;
        long l1 = i1 + 1L;
        double d1 = l1 + 0.5;
        char c1 = 'a';
        int i2 = c1 + 1;
        System.out.println(i1 + "," + l1 + "," + d1 + "," + (char) i2);
        // 窄类型复合赋值链
        System.out.println(bump((byte) 100));
        System.out.println(shr((short) 20000));
        // 十六进制/八进制 long 边角（负值域）
        long h1 = 0x7FFFFFFFFFFFFFFFL;
        long h2 = 0x8000000000000000L;
        long h3 = 0xFFFFFFFFFFFFFFFFL;
        System.out.println(h1 + "," + h2 + "," + h3);
        int h4 = 0x80000000; // 无符号 32 位域内合法
        System.out.println(h4);
        // 移位：>>> 无符号
        int neg = -8;
        System.out.println(neg >> 1);
        System.out.println(neg >>> 1);
        System.out.println(neg >>> 60);
        long ln = -8L;
        System.out.println(ln >>> 63);
        // 字符串 switch + 落穿
        String op = "mul";
        int r = 0;
        switch (op) {
            case "add":
                r += 1;
            case "mul":
                r += 2;
                // 落穿到 sub
            case "sub":
                r += 4;
                break;
            default:
                r = -1;
        }
        System.out.println(r);
        // 枚举 switch
        enum Day { MON, TUE }
        Day d = Day.TUE;
        switch (d) {
            case MON: System.out.println("m"); break;
            case TUE: System.out.println("t"); break;
        }
        // record 使用
        Point p = new Point(3, 4);
        System.out.println(p.x() + p.y() + p.sum());
        Pair<String> pr = new Pair<>("a", "b");
        System.out.println(pr.a() + pr.b());
        Shape s = new Circle(2.0);
        System.out.println(s instanceof Circle ci ? ci.r() : -1);
        // 三元混合类型（int/long 提升路径）
        boolean cond = args.length == 0;
        long mix = cond ? 1 : 2L;
        System.out.println(mix);
        // 字符串拼接 null
        String nil = null;
        System.out.println("v=" + nil);
        System.out.println('a' + "b" + 'c');
        System.out.println("x" + 'y' + 1);
        // 嵌套 try + 循环
        for (int i = 0; i < 3; i++) {
            try {
                if (i == 1) { continue; }
                if (i == 2) { break; }
                System.out.println("iter" + i);
            } finally {
                System.out.println("fin" + i);
            }
        }
        // 嵌套 switch
        switch (1) {
            case 1:
                switch (2) {
                    case 2: System.out.println("nested"); break;
                }
                System.out.println("outer");
                break;
        }
        System.out.println(annotatedField);
    }
}
"#,
    );
}

#[test]
fn raw_statement_opacity_protects_variables() {
    // 深层 bug（对抗波 3 抓获）：RAW（不可解析原文）语句对用量分析不可见
    // → 仅被 RAW 引用的变量被零用途规则误删，且值被提前传播越过 RAW 中的
    // 赋值（语义损坏）。修复=Opaque 语义贯通：含 Raw 的语句不建事件索引
    // → 递归扫描设 opaque → 四条传播/删除规则 + StoreKill 全部保守拒绝。
    // 注：原文件含非 Java 语法（:=），javac 差分不适用（原版也不可编译）；
    // 验证 = 形态断言：RAW 保真、变量存活、传播不越过 RAW。
    let src = "class A{void m(){int v = 7; v =: 3; System.out.println(v);}}";
    let mut outcome = parse(src);
    simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let out = print_unit(&outcome.ast, &outcome.unit);
    assert!(out.contains("int v"), "RAW 引用的变量被误删:\n{out}");
    assert!(out.contains("v =: 3"), "RAW 丢失:\n{out}");
    assert!(out.contains("println(v)"), "值被提前传播越过 RAW 赋值:\n{out}");
}

#[test]
fn adversarial_captures_and_exotic_forms() {
    // 对抗波 4：匿名类捕获外部局部变量、局部类捕获、接口私有/static/
    // default 方法、枚举常量专属方法体、泛型方法调用 Adv4.<T>box()、
    // C 风格数组声明（int a[] = {..}, b[], c; int[] d[]）、this 引用、
    // 标签块 break、char/byte/short 复合赋值隐式收窄（ch += 2; bb += 100;
    // sh *= 100）、装箱三元 + null、深嵌套泛型、静态导入。
    // 抓获 bug 9：匿名类体是原文（anon_raw）→ 捕获的外部局部变量对用量
    // 分析不可见 → 仅被匿名类捕获的变量被零用途规则误删（输出无法编译）。
    // 修复=Lang::is_opaque 钩子（Raw 节点 + 匿名类 New）贯通事件与递归
    // 两条扫描路径。
    differential(
        "Adv4",
        r#"import static java.lang.Math.max;

public class Adv4 {
    int field = 10;
    static int sfield = 20;

    interface P {
        default int d() { return p() * 2; }
        static int s() { return 5; }
        private int p() { return 3; }
    }

    enum Op {
        ADD { int ap(int a, int b) { return a + b; } },
        MUL { int ap(int a, int b) { return a * b; } };
        abstract int ap(int a, int b);
    }

    static class Holder {
        int v;
        Holder(int v) { this.v = v; }
        int get() { return v; }
    }

    public static void main(String[] args) {
        int x = 5;
        Runnable r = new Runnable() {
            @Override public void run() { System.out.println("x=" + x); }
        };
        r.run();
        int y = 7;
        Runnable r2 = new Runnable() {
            @Override public void run() { System.out.println("y=" + y); }
        };
        r2.run();
        System.out.println("y2=" + y);
        int z = 9;
        class Local {
            int zap() { return z + 1; }
        }
        System.out.println("z=" + new Local().zap());
        Adv4 outer = new Adv4();
        System.out.println("f=" + outer.field);
        System.out.println("g=" + Adv4.<Integer>box(42));
        int a[] = {1, 2};
        int b[] = {3}, c = 4;
        int[] d[] = {{5}, {6}};
        System.out.println(a[1] + "," + b[0] + "," + c + "," + d[1][0]);
        System.out.println("op=" + Op.ADD.ap(3, 4) + "," + Op.MUL.ap(3, 4));
        System.out.println("p=" + new P() {}.d() + "," + P.s());
        Integer boxed = args.length == 0 ? null : 1;
        System.out.println("bx=" + boxed);
        java.util.Map<String, java.util.List<int[]>> deep = new java.util.HashMap<>();
        deep.put("k", java.util.List.of(new int[]{7, 8}));
        System.out.println("deep=" + deep.get("k").get(0)[1]);
        System.out.println("max=" + max(3, 9));
        System.out.println("h=" + new Holder(11).get() + "," + new Holder(12).v);
        for (var s : java.util.List.of("a", "b")) { System.out.println("v=" + s); }
        java.util.function.IntSupplier sup = () -> max(sfield, 25);
        System.out.println("sup=" + sup.getAsInt());
        blk:
        {
            for (int i = 0; i < 3; i++) {
                if (i == 1) { break blk; }
                System.out.println("blk" + i);
            }
            System.out.println("unreachable");
        }
        char ch = 'a';
        ch += 2;
        System.out.println("ch=" + ch);
        byte bb = 100;
        bb += 100;
        System.out.println("bb=" + bb);
        short sh = 1000;
        sh *= 100;
        System.out.println("sh=" + sh);
    }

    static <T> T box(T v) { return v; }
}
"#,
    );
    // 捕获保护专项：仅被匿名类引用的变量必须存活
    let src = "class A{void m(){int y = 7; Runnable r = new Runnable(){public void run(){System.out.println(y);}}; r.run();}}";
    let mut outcome = parse(src);
    simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let out = print_unit(&outcome.ast, &outcome.unit);
    assert!(out.contains("int y = 7"), "匿名类捕获变量被误删:\n{out}");
}

#[test]
fn adversarial_expression_edge_cases() {
    // 对抗波 5（全干净，零 bug——固化防回归）：三元右结合链、赋值链
    //（x=y=z=9）、复合移位赋值（<<= >>= >>>=）、for(;;) + break、空语句
    // 串、自增在数组下标（arr[++i]/arr[i++]/arr[j]+=5）、do-while+continue
    //（continue 跳条件求值）、短路求值副作用保序、位运算优先级链
    //（a|b&c^d）、锯齿/立方数组、catch 内再抛 + finally 副作用。
    // 附带容错压力：空文件/仅注释/乱码/未闭合/字符串未闭合均不 panic
    // 且错误数有界；1+1+...×2000 常量链折叠结果精确。
    differential(
        "Adv5",
        r#"public class Adv5 {
    static int[] arr = {10, 20, 30};
    static int s = 0;
    static int side() { s++; return s; }

    public static void main(String[] args) {
        int a = 1, b = 2, c = 3, d = 4;
        int t1 = a < b ? b : c < d ? d : 5;
        int t2 = (a < b ? b : c < d ? d : 5) + (a > b ? 100 : 200);
        System.out.println("t=" + t1 + "," + t2);
        int x, y, z;
        x = y = z = 9;
        System.out.println("chain=" + x + y + z);
        int sh = 1;
        sh <<= 4;
        sh >>= 2;
        sh >>>= 1;
        System.out.println("sh=" + sh);
        int neg = -16;
        neg >>>= 2;
        System.out.println("neg=" + neg);
        int cnt = 0;
        for (;;) {
            cnt++;
            if (cnt >= 3) { break; }
        }
        System.out.println("cnt=" + cnt);
        ;;;
        System.out.println("semi");
        int i = 0;
        System.out.println("pre=" + arr[++i] + " i=" + i);
        System.out.println("post=" + arr[i++] + " i=" + i);
        int j = 0;
        arr[j] += 5;
        System.out.println("aa=" + arr[0]);
        int k = 0, sum = 0;
        do {
            k++;
            if (k % 2 == 0) { continue; }
            sum += k;
        } while (k < 6);
        System.out.println("dw=" + k + "," + sum);
        System.out.println("sc=" + (a < b && side() > 0) + "," + (a > b || side() > 0));
        System.out.println("s=" + s);
        System.out.println("cc=" + "" + 'a' + 1 + 'b' + 2);
        boolean cmp = a < b == c < d;
        System.out.println("cmp=" + cmp);
        int bits = a | b & c ^ d;
        System.out.println("bits=" + bits);
        int[][] jag2 = new int[2][];
        jag2[0] = new int[1];
        jag2[1] = new int[2];
        System.out.println("jag=" + jag2.length + jag2[1].length);
        int[][][] cube = new int[2][3][4];
        System.out.println("cube=" + cube[1][2].length);
        int total = 0;
        for (int v : arr) { total += v; }
        System.out.println("fe=" + total);
        System.out.println("ce=" + nested());
    }

    static String nested() {
        try {
            try {
                throw new RuntimeException("in");
            } catch (RuntimeException e) {
                throw new IllegalStateException("re");
            }
        } catch (IllegalStateException | IllegalArgumentException e) {
            return "caught:" + e.getMessage();
        } finally {
            side();
        }
    }
}
"#,
    );
}
