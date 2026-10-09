//! 虚拟执行（部分求值）混淆样本：混淆器把常量藏进方法调用里——
//! substring/format/parseInt/Base64/字符算术/字面量数组——cure 在编译期
//! "执行"这些纯调用，还原成字面量。javac 差分验证语义。
//!
//! 参考模式来源：obfuscator.io / javascript-obfuscator 的 string-array、
//! 字符串编码与死常量变换家族的 Java 等价形态。

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

const EVAL_OBFUSCATED: &str = r#"
public class EvalObf {
    public static void main(String[] args) {
        // ===== [substring 藏匿] =====
        String a = "HelloWorld".substring(0, 5) + "!";
        System.out.println(a);

        // ===== [String.format 拼装] =====
        String b = String.format("%s=%d", "count", 42);
        System.out.println(b);

        // ===== [parseInt/valueOf 双重转换] =====
        String c = String.valueOf(Integer.parseInt("2024"));
        System.out.println(c);

        // ===== [Base64 编码字符串] =====
        String d = new String(java.util.Base64.getDecoder().decode("aGVsbG8="));
        System.out.println(d);

        // ===== [字符算术链]（字符级异或/加减后拼回字符串） =====
        String e = "" + (char) ('a' + 2) + (char) ('b' + 1);
        System.out.println(e);

        // ===== [字面量表 + 下标取值]（string-array 的极简形态） =====
        String[] table = {"zero", "one", "two"};
        String f = table[2];
        System.out.println(f);

        // ===== [字符串杂项] =====
        int g = "abcdef".indexOf('d');
        String h = "  padded  ".trim();
        String i2 = "a,b,c".replace(',', '-');
        boolean j = "Config".equalsIgnoreCase("CONFIG");
        boolean k = "prefix_ok".startsWith("prefix");
        System.out.println(g);
        System.out.println(h);
        System.out.println(i2);
        System.out.println(j);
        System.out.println(k);

        // ===== [Math/Character] =====
        int l = Math.max(3, 5) + Math.abs(-2);
        boolean m = Character.isDigit('7');
        char n = Character.toUpperCase('q');
        System.out.println(l);
        System.out.println(m);
        System.out.println(n);

        // ===== [组合：所有层叠加] =====
        String o = "SecretData".substring(0, 6).toLowerCase().length() == 0 ? "x" : String.format("%s-%d", "val", 7);
        System.out.println(o);
    }
}
"#;

#[test]
fn virtual_execution_obfuscation_deobfuscated() {
    let mut outcome = parse(EVAL_OBFUSCATED);
    assert!(outcome.errors.is_empty(), "解析失败：{:?}", outcome.errors);
    let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let cleaned = print_unit(&outcome.ast, &outcome.unit);

    // ---- javac 差分 ----
    if javac_available() {
        let base = std::env::temp_dir().join("cure_eval");
        let _ = fs::remove_dir_all(&base);
        let orig_dir = base.join("orig");
        let clean_dir = base.join("clean");
        fs::create_dir_all(&orig_dir).unwrap();
        fs::create_dir_all(&clean_dir).unwrap();
        fs::write(orig_dir.join("EvalObf.java"), EVAL_OBFUSCATED).unwrap();
        fs::write(clean_dir.join("EvalObf.java"), &cleaned).unwrap();
        for d in [&orig_dir, &clean_dir] {
            let out = Command::new("javac")
                .arg("-encoding")
                .arg("UTF-8")
                .arg("-nowarn")
                .arg("-d")
                .arg(d)
                .arg(d.join("EvalObf.java"))
                .output()
                .unwrap();
            assert!(out.status.success(), "javac：{}", String::from_utf8_lossy(&out.stderr));
        }
        let run = |d: &Path| {
            Command::new("java")
                .arg("-cp")
                .arg(d)
                .arg("EvalObf")
                .output()
                .unwrap()
        };
        let (a, b) = (run(&orig_dir), run(&clean_dir));
        assert_eq!(a.status.code(), b.status.code());
        assert_eq!(
            String::from_utf8_lossy(&a.stdout),
            String::from_utf8_lossy(&b.stdout),
            "虚拟执行改写改变了行为！"
        );
        eprintln!(
            "语义差分通过：{}",
            String::from_utf8_lossy(&a.stdout).replace('\n', " / ").trim()
        );
    }

    // ---- 各层还原 ----
    // substring → 字面量拼接
    assert!(cleaned.contains(r#"println("Hello!"#), "{cleaned}");
    // format → 直接拼接
    assert!(cleaned.contains(r#"println("count=42"#), "{cleaned}");
    // parseInt+valueOf 双重转换 → 字符串字面量
    assert!(cleaned.contains(r#"println("2024")"#), "{cleaned}");
    // Base64 → 解码明文
    assert!(cleaned.contains(r#"println("hello")"#), "{cleaned}");
    // 字符算术 → 字符字面量拼接（(char)('a'+2)+(char)('b'+1) = "cc"）
    assert!(cleaned.contains(r#"println("cc")"#), "{cleaned}");
    // 字面量表下标 → 元素（声明链传播后 table 折叠）
    assert!(cleaned.contains(r#"println("two")"#), "{cleaned}");
    // 字符串杂项
    assert!(cleaned.contains("println(3);"), "{cleaned}");
    assert!(cleaned.contains(r#"println("padded")"#), "{cleaned}");
    assert!(cleaned.contains(r#"println("a-b-c")"#), "{cleaned}");
    assert!(cleaned.contains("println(true);"), "{cleaned}");
    // Math/Character
    assert!(cleaned.contains("println(7);"), "{cleaned}");
    assert!(cleaned.contains("println('Q');"), "{cleaned}");
    // toLowerCase() 属 locale 敏感 → 不折（保守），但 substring/format 已折
    assert!(cleaned.contains(r#""Secret".toLowerCase()"#), "{cleaned}");
    assert!(cleaned.contains(r#""val-7""#), "{cleaned}");

    let a = EVAL_OBFUSCATED.lines().filter(|l| !l.trim().is_empty()).count();
    let b = cleaned.lines().filter(|l| !l.trim().is_empty()).count();
    eprintln!(
        "虚拟执行：{} 次改写，非空行 {} → {}（-{}%）",
        report.edits,
        a,
        b,
        100 - b * 100 / a
    );
    assert!(report.edits >= 15, "预期大量求值，实际 {}", report.edits);
}
