//! 刁钻混淆测试：多种模式**组合、嵌套、多层叠加**，javac 差分验证语义保持。
//!
//! 覆盖：多层常量隐藏（声明链×异或对×位噪声×死赋值）、嵌套不透明谓词、
//! 循环混合断路+寄存器噪声、多层字符串混淆（SB×new String×valueOf×分散常量×length）、
//! 布尔旗标三元嵌套、迭代器+SB 语句链+continue 组合、副作用调用的异或包裹。

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

const HARD_OBFUSCATED: &str = r#"
public class HardObf {
    static int trace = 0;

    static int mark(int v) {
        trace += v;
        return v;
    }

    public static void main(String[] args) {
        // ===== [多层常量隐藏]：声明链 + 双异或 + 位噪声 + 拆分赋值 =====
        int k = 20;
        int k2 = k + 22;
        int k3 = k2 - 22;
        int k4 = ((k3 ^ 0x5A) ^ 0x5A) | 0;
        int k5;
        k5 = k4 & -1;
        System.out.println("k=" + k5);

        // ===== [嵌套不透明谓词]：双层 + 死分支 =====
        boolean o1 = 2 > 1;
        boolean o2 = 3 < 4;
        if (o1 && o2) {
            System.out.println("live");
        } else {
            System.out.println("dead1");
        }
        if (1 > 2) {
            System.out.println("dead2");
        }

        // ===== [循环混合]：while(true) + 真实断路 + 寄存器回拷噪声 =====
        int v24 = 0;
        int v25 = 0;
        v24 = 1;
        while (true) {
            if (v24 >= 4) {
                break;
            } else {
                int v32 = v25 + v24;
                int v33 = v24 + 1;
                v25 = v32;
                v24 = v33;
            }
        }
        System.out.println("sum=" + v25);

        // ===== [多层字符串]：SB 常量链 + new String + valueOf + 分散常量 =====
        String s1 = new String(new StringBuilder().append("he").append("llo").toString());
        String s2 = String.valueOf(s1.length()) + "!";
        String s3 = "a" + s2 + "b" + "c" + "d";
        System.out.println(s3);

        // ===== [布尔旗标三元嵌套] =====
        boolean flag = (v25 > 3 ? true : false);
        boolean flag2 = flag ? (v25 > 5 ? true : false) : false;
        System.out.println("f=" + (flag2 ? 1 : 0));

        // ===== [迭代器 + SB 语句链 + continue 组合] =====
        java.util.List<String> list = new java.util.ArrayList<>();
        list.add("x1");
        list.add("yy");
        java.util.Iterator<String> it = list.iterator();
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

        // ===== [副作用异或包裹]：mark 调用恰好一次、位置不变 =====
        int r = (mark(5) ^ 0x5A) ^ 0x5A;
        System.out.println("r=" + r);

        System.out.println("trace=" + trace);
    }
}
"#;

#[test]
fn hard_composed_obfuscation_deobfuscated() {
    // ---- 处理 ----
    let mut outcome = parse(HARD_OBFUSCATED);
    assert!(outcome.errors.is_empty(), "混淆源解析失败：{:?}", outcome.errors);
    let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let cleaned = print_unit(&outcome.ast, &outcome.unit);

    // ---- 1. 语义保持（javac 差分） ----
    if javac_available() {
        let base = std::env::temp_dir().join("cure_hard");
        let _ = fs::remove_dir_all(&base);
        let orig_dir = base.join("orig");
        let clean_dir = base.join("clean");
        fs::create_dir_all(&orig_dir).unwrap();
        fs::create_dir_all(&clean_dir).unwrap();
        let of = orig_dir.join("HardObf.java");
        let cf = clean_dir.join("HardObf.java");
        fs::write(&of, HARD_OBFUSCATED).unwrap();
        fs::write(&cf, &cleaned).unwrap();
        for (d, f) in [(&orig_dir, &of), (&clean_dir, &cf)] {
            let out = Command::new("javac")
                .arg("-encoding")
                .arg("UTF-8")
                .arg("-nowarn")
                .arg("-d")
                .arg(d)
                .arg(f)
                .output()
                .unwrap();
            assert!(out.status.success(), "javac：{}", String::from_utf8_lossy(&out.stderr));
        }
        let (a_out, a_code) = compile_and_run(&orig_dir, "HardObf");
        let (b_out, b_code) = compile_and_run(&clean_dir, "HardObf");
        assert_eq!(a_code, b_code);
        assert_eq!(
            a_out, b_out,
            "去混淆改变行为！\n== 原版 ==\n{a_out}\n== 去混淆版 ==\n{b_out}"
        );
        eprintln!("语义差分通过：{}", a_out.replace('\n', " / ").trim());
    }

    // ---- 2. 各层混淆被击穿 ----
    // 多层常量链：整链塌缩成 "k=20"（含 Str+Int 字面量拼接折叠）
    assert!(cleaned.contains(r#"println("k=20")"#), "{cleaned}");
    assert!(!cleaned.contains("0x5A"), "{cleaned}");
    assert!(!cleaned.contains("k2"), "{cleaned}");

    // 嵌套不透明谓词：死分支与谓词变量消失
    assert!(cleaned.contains("live"), "{cleaned}");
    assert!(!cleaned.contains("dead1"), "{cleaned}");
    assert!(!cleaned.contains("dead2"), "{cleaned}");
    assert!(!cleaned.contains("o1"), "{cleaned}");
    assert!(!cleaned.contains("o2"), "{cleaned}");

    // 循环：while(true)+寄存器回拷 → 干净的 while 循环
    assert!(cleaned.contains("while (v24 < 4)"), "{cleaned}");
    assert!(cleaned.contains("v25 = v25 + v24;"), "{cleaned}");
    assert!(!cleaned.contains("v32"), "{cleaned}");
    assert!(!cleaned.contains("v33"), "{cleaned}");

    // 多层字符串：SB 链 + new String + valueOf + length + Int/Str 拼接全折叠
    assert!(!cleaned.contains("new StringBuilder"), "{cleaned}");
    assert!(!cleaned.contains("new String("), "{cleaned}");
    assert!(!cleaned.contains("String.valueOf"), "{cleaned}");
    assert!(cleaned.contains(r#"println("a5!bcd")"#), "{cleaned}");

    // 布尔旗标三元嵌套 → 合取
    assert!(cleaned.contains("v25 > 3 && v25 > 5"), "{cleaned}");

    // 迭代器 + SB 语句链 → for-each + 累加拼接（内联 next() 提取已支持）
    assert!(cleaned.contains("for (String e : list)"), "{cleaned}");
    assert!(cleaned.contains("acc = acc + e + \"-\";"), "{cleaned}");
    assert!(!cleaned.contains("new StringBuilder"), "{cleaned}");
    assert!(!cleaned.contains("continue"), "{cleaned}");
    assert!(!cleaned.contains("acc2"), "{cleaned}");

    // 副作用异或包裹被剥开（mark 调用保留恰好一次；
    // r 不内联进 println 参数位是正确的保守行为——接收者 System.out 先求值）
    assert!(cleaned.contains("int r = mark(5);"), "{cleaned}");
    assert!(!cleaned.contains("0x5A"), "{cleaned}");

    // ---- 3. 统计 ----
    let a = HARD_OBFUSCATED.lines().filter(|l| !l.trim().is_empty()).count();
    let b = cleaned.lines().filter(|l| !l.trim().is_empty()).count();
    eprintln!(
        "刁钻混淆：{} 次改写，非空行 {} → {}（-{}%）",
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
