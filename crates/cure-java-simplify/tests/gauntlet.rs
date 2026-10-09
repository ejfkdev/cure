//! 终极组合混淆（gauntlet）：跨方法字符串解密器（string-array + Base64 helper）
//! × 控制流扁平化 × 虚拟执行 × 寄存器噪声 × 不透明谓词 × XOR 异或 × SB 链
//! —— 全部叠在一个程序里，javac 差分验证语义。
//!
//! 模式参考 javascript-obfuscator / Allatori 的 string-array 家族。

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
    (
        String::from_utf8_lossy(&run.stdout).to_string(),
        run.status.code(),
    )
}

const GAUNTLET: &str = r#"
public class Gauntlet {
    // ===== [跨方法字符串表 + 解密 helper]（string-array 家族）=====
    static final String[] T = new String[]{"c3Vw", "ZXI=", "aGVsbG8="};

    static String d(int i) {
        return new String(java.util.Base64.getDecoder().decode(T[i]));
    }

    static int mark(int v) {
        side += v;
        return v;
    }

    static int side = 0;

    public static void main(String[] args) {
        // ===== [解密 + 拼接 + 多层字符串]=====
        String s = d(0) + d(1) + "!";
        System.out.println(s);

        // ===== [CFF 状态机：体内含解密调用 + 寄存器噪声 + 不透明谓词]=====
        int v24 = 0;
        int v25 = 0;
        int v7 = 0;
        v24 = 1;
        while (true) {
            switch (v24) {
                case 0: {
                    if (2 > 1) {
                        v24 = 1;
                    } else {
                        v24 = 5;
                    }
                    break;
                }
                case 1: {
                    if (v25 < 3) {
                        v24 = 2;
                    } else {
                        v24 = 4;
                    }
                    break;
                }
                case 2: {
                    int v32 = v25 + mark(1);
                    v25 = v32;
                    v24 = 1;
                    break;
                }
                case 4: {
                    v7 = v25 * 100;
                    v24 = 9;
                    break;
                }
                case 5: {
                    v7 = -1;
                    v24 = 9;
                    break;
                }
            }
            if (v24 == 9) {
                break;
            }
        }
        System.out.println("v7=" + v7);

        // ===== [解密 + SB 语句链 + 迭代器组合]=====
        java.util.List<String> list = new java.util.ArrayList<>();
        list.add(d(2));
        list.add("world");
        java.util.Iterator it = list.iterator();
        String acc = "";
        while (true) {
            if (!(it.hasNext())) {
                break;
            } else {
                String e = (String) it.next();
                StringBuilder sb = new StringBuilder().append(acc);
                sb = sb.append(e);
                StringBuilder sb2 = sb.append("-");
                String acc2 = sb2.toString();
                acc = acc2;
                continue;
            }
        }
        System.out.println("acc=" + acc);

        // ===== [XOR 噪声 + 虚拟执行组合]=====
        int r = (mark(5) ^ 0x5A) ^ 0x5A;
        String f = String.format("%s=%d", "total", r + "abc".length());
        System.out.println(f);
        System.out.println("side=" + side);
    }
}
"#;

#[test]
fn gauntlet_full_stack_deobfuscation() {
    let mut outcome = parse(GAUNTLET);
    assert!(outcome.errors.is_empty(), "解析失败：{:?}", outcome.errors);
    let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let cleaned = print_unit(&outcome.ast, &outcome.unit);

    // ---- javac 差分 ----
    if javac_available() {
        let base = std::env::temp_dir().join("cure_gauntlet");
        let _ = fs::remove_dir_all(&base);
        let orig_dir = base.join("orig");
        let clean_dir = base.join("clean");
        fs::create_dir_all(&orig_dir).unwrap();
        fs::create_dir_all(&clean_dir).unwrap();
        fs::write(orig_dir.join("Gauntlet.java"), GAUNTLET).unwrap();
        fs::write(clean_dir.join("Gauntlet.java"), &cleaned).unwrap();
        for d in [&orig_dir, &clean_dir] {
            let out = Command::new("javac")
                .arg("-encoding")
                .arg("UTF-8")
                .arg("-nowarn")
                .arg("-d")
                .arg(d)
                .arg(d.join("Gauntlet.java"))
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "javac 失败：\n{}\n== 净化输出 ==\n{cleaned}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        let (a_out, a_code) = compile_and_run(&orig_dir, "Gauntlet");
        let (b_out, b_code) = compile_and_run(&clean_dir, "Gauntlet");
        assert_eq!(a_code, b_code);
        assert_eq!(a_out, b_out, "组合混淆还原改变行为！\n== 净化输出 ==\n{cleaned}");
        eprintln!("语义差分通过：{}", a_out.replace('\n', " / ").trim());
    }

    // ---- 各层击穿 ----
    // 跨方法解密：d(0)+d(1) → "super"
    assert!(cleaned.contains(r#"println("super!")"#), "{cleaned}");
    // 解密 helper 不再被【调用】（方法本身保留是死方法——死方法删除不在范围）
    assert!(!cleaned.contains(r#"d(0)"#), "{cleaned}");
    assert!(!cleaned.contains(r#"d(1)"#), "{cleaned}");
    assert!(!cleaned.contains(r#"d(2)"#), "{cleaned}");

    // CFF：状态机整体还原（含不透明谓词折叠、mark 副作用保留）
    assert!(!cleaned.contains("switch"), "{cleaned}");
    assert!(cleaned.contains("while (v25 < 3)"), "{cleaned}");
    assert!(cleaned.contains("mark(1)"), "{cleaned}");
    // v7 经 v25 表达（CFF 还原 + 后续传播把 v7 内联进 println）
    assert!(cleaned.contains(r#""v7=" + v25 * 100"#), "{cleaned}");

    // SB 语句链 + 迭代器：d(2) 经 Base64 解码成 "hello" 后入列表
    assert!(cleaned.contains(r#"add("hello")"#), "{cleaned}");
    // list 里的 d(2) 保留为调用？—— d(2) 内联后是 new String(decode("aGVsbG8="))
    // → Base64 折叠 → "hello" ✓
    assert!(cleaned.contains(r#"add("hello")"#), "{cleaned}");

    // XOR 噪声剥除（mark 保留恰好一次；r 不内联进 println 参数是正确的
    // 保守行为——接收者 System.out 先求值）；"abc".length() 虚拟执行折叠
    assert!(cleaned.contains("mark(5)"), "{cleaned}");
    assert!(!cleaned.contains("0x5A"), "{cleaned}");
    assert!(cleaned.contains(r#"String.format("%s=%d", "total", r + 3)"#), "{cleaned}");

    // ---- 统计 ----
    let a = GAUNTLET.lines().filter(|l| !l.trim().is_empty()).count();
    let b = cleaned.lines().filter(|l| !l.trim().is_empty()).count();
    eprintln!(
        "终极组合：{} 次改写，非空行 {} → {}（-{}%）",
        report.edits,
        a,
        b,
        100 - b * 100 / a
    );
    let mut rules: Vec<_> = report.by_rule.iter().collect();
    rules.sort();
    for (k, v) in rules {
        eprintln!("  {k}: {v}");
    }
    assert!(report.edits >= 20, "预期大规模净化，实际 {}", report.edits);
}
