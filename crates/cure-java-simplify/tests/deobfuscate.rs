//! 去混淆能力演示：模拟混淆器（ZKM/Allatori/ProGuard 风格源码形态）→ cure → 验证。
//!
//! 混淆模式覆盖：不透明谓词+死分支、双异或、位/算术噪声、布尔包装三元、
//! if-return 布尔特例、new String、StringBuilder 链、装箱拆箱链、迭代器循环、
//! if-else 赋值分叉、临时变量链。
//!
//! 验证两件事：
//! 1. **语义保持**（javac 差分）：混淆版与去混淆版编译运行输出完全一致
//!    —— 特别是有副作用的地方（bump 调用序列）必须原样保留；
//! 2. **去混淆有效性**：关键混淆形态在输出中消失，出现人类可读形态。

use std::fs;
use std::path::Path;
use std::process::Command;

use cure_engine::Config;
use cure_java_parser::parse;
use cure_java_print::print_unit;
use cure_java_simplify::simplify_unit;

/// 模拟混淆器输出的程序（每个混淆模式都带确定性可观察输出）。
const OBFUSCATED: &str = r#"
public class ObfDemo {
    static int sideCount = 0;

    static int bump(int delta) {
        sideCount += delta;
        return sideCount;
    }

    public static void main(String[] args) {
        // [不透明谓词 + 死分支]
        boolean always = 2 > 1;
        if (always) {
            System.out.println("opaque-true");
        } else {
            System.out.println("dead-branch");
        }

        // [双异或] key 通过两次异或隐藏
        int key = 42;
        int y = (key ^ 84) ^ 84;
        System.out.println(y);

        // [位/算术噪声]
        int z = ((y | 0) & -1) + 0;
        System.out.println(z);

        // [布尔包装三元]
        boolean flag = (z > 10) ? true : false;
        System.out.println(flag);

        // [if-return 布尔特例]
        if (flag) {
            System.out.println("big");
        } else {
            System.out.println("small");
        }

        // [死赋值 + 临时变量链]（bump 有副作用：第一个赋值必须保留！）
        int t = bump(1);
        t = bump(2);
        int u = t;
        System.out.println(u);

        // [new String + StringBuilder 链]
        String s = new String("hello");
        String msg = new StringBuilder().append(s).append("-").append(z).toString();
        System.out.println(msg);

        // [装箱拆箱链]
        System.out.println(Integer.valueOf(42).intValue());

        // [迭代器循环]
        java.util.List<String> list = new java.util.ArrayList<>();
        list.add("a");
        list.add("bb");
        int total = 0;
        for (java.util.Iterator<String> it = list.iterator(); it.hasNext(); ) {
            String e = it.next();
            total += e.length();
        }
        System.out.println(total);

        // [if-else 赋值分叉]
        int r;
        if (flag) {
            r = 1;
        } else {
            r = 2;
        }
        System.out.println(r);

        // [拆分声明 + 拷贝链]（jadx 寄存器形态的顺序部分）
        int p;
        p = 11;
        int q;
        q = p;
        int w;
        w = q;
        System.out.println(w);

        // [valueOf 包装]
        String v = String.valueOf(42) + "x";
        System.out.println(v);

        // [拼接常量分散]
        String m2 = "a" + v + "b" + "c";
        System.out.println(m2);

        System.out.println(sideCount);
    }
}
"#;

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

#[test]
fn deobfuscate_simulated_obfuscator() {
    // ---- 处理 ----
    let mut outcome = parse(OBFUSCATED);
    assert!(outcome.errors.is_empty(), "混淆源必须能干净解析");
    let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let cleaned = print_unit(&outcome.ast, &outcome.unit);

    // ---- 1. 语义保持（javac 差分） ----
    if javac_available() {
        let base = std::env::temp_dir().join("cure_deobf");
        let _ = fs::remove_dir_all(&base);
        let orig_dir = base.join("orig");
        let clean_dir = base.join("clean");
        fs::create_dir_all(&orig_dir).unwrap();
        fs::create_dir_all(&clean_dir).unwrap();
        let of = orig_dir.join("ObfDemo.java");
        let cf = clean_dir.join("ObfDemo.java");
        fs::write(&of, OBFUSCATED).unwrap();
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
            assert!(
                out.status.success(),
                "javac 失败：\n{}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        let (a_out, a_code) = compile_and_run(&orig_dir, "ObfDemo");
        let (b_out, b_code) = compile_and_run(&clean_dir, "ObfDemo");
        assert_eq!(a_code, b_code);
        assert_eq!(
            a_out, b_out,
            "去混淆改变了可观察行为！\n== 原版 ==\n{a_out}\n== 去混淆版 ==\n{b_out}"
        );
        eprintln!("语义差分通过；输出：{}", a_out.replace('\n', " / ").trim());
    } else {
        eprintln!("skip javac differential: javac not found");
    }

    // ---- 2. 去混淆有效性 ----
    // 不透明谓词被击穿：死分支与谓词变量消失
    assert!(cleaned.contains("opaque-true"), "{cleaned}");
    assert!(!cleaned.contains("dead-branch"), "{cleaned}");
    assert!(!cleaned.contains("always"), "{cleaned}");

    // 双异或被还原为常量
    assert!(!cleaned.contains("^ 84"), "{cleaned}");
    assert!(cleaned.contains("int y = 42;"), "{cleaned}");

    // 位/算术噪声消失（z 归约为 y 的直传）
    assert!(!cleaned.contains("| 0"), "{cleaned}");
    assert!(!cleaned.contains("& -1"), "{cleaned}");
    assert!(cleaned.contains("int z = y;") || cleaned.contains("int z = 42;"), "{cleaned}");

    // 布尔包装三元 → 直接布尔
    assert!(!cleaned.contains("? true : false"), "{cleaned}");

    // new String / StringBuilder / 装箱链被还原
    assert!(!cleaned.contains("new String("), "{cleaned}");
    assert!(!cleaned.contains("new StringBuilder"), "{cleaned}");
    assert!(!cleaned.contains("Integer.valueOf"), "{cleaned}");
    // new String + SB 链 → 字面量拼接（并继续折叠成 "hello-"）
    assert!(cleaned.contains(r#""hello-"#), "{cleaned}");

    // 迭代器循环 → for-each
    assert!(cleaned.contains("for (String e : list)"), "{cleaned}");
    assert!(!cleaned.contains("Iterator"), "{cleaned}");

    // if-else 赋值分叉 → 三元赋值 → r 唯一使用处继续内联
    assert!(cleaned.contains("println(flag ? 1 : 2);"), "{cleaned}");
    assert!(!cleaned.contains("int r;"), "{cleaned}");

    // 副作用安全：bump(1)/bump(2) 调用都保留（带副作用的死赋值不删），
    // 且赋值传播把 `t = bump(2); int u = t;` 归并为 `int u = bump(2);`
    assert!(cleaned.contains("int t = bump(1);"), "{cleaned}");
    assert!(cleaned.contains("int u = bump(2);"), "{cleaned}");
    assert!(!cleaned.contains("t = bump(2);"), "{cleaned}");

    // 拆分声明 + 拷贝链 → 全链塌缩为常量
    assert!(cleaned.contains("println(11);"), "{cleaned}");
    assert!(!cleaned.contains("int p;"), "{cleaned}");
    assert!(!cleaned.contains("q = p;"), "{cleaned}");

    // valueOf 剥离 + 拼接常量合并（Int+Str 字面量直接折成 "42x"）
    assert!(!cleaned.contains("String.valueOf"), "{cleaned}");
    assert!(cleaned.contains(r#"String v = "42x";"#), "{cleaned}");
    assert!(cleaned.contains(r#""a" + v + "bc""#), "{cleaned}");

    // ---- 3. 统计 ----
    let obf_lines = OBFUSCATED.lines().filter(|l| !l.trim().is_empty()).count();
    let clean_lines = cleaned.lines().filter(|l| !l.trim().is_empty()).count();
    eprintln!(
        "去混淆：{} 次改写，非空行 {} → {}（-{}%）",
        report.edits,
        obf_lines,
        clean_lines,
        100 - clean_lines * 100 / obf_lines
    );
    let mut rules: Vec<_> = report.by_rule.iter().collect();
    rules.sort();
    for (k, v) in rules {
        eprintln!("  {k}: {v}");
    }
    assert!(report.edits >= 12, "预期大量改写，实际 {}", report.edits);
}
