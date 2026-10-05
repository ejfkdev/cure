//! 真实混淆工具链集成测试（环境可用时执行，否则跳过）：
//!
//!   干净源码 → javac → **ProGuard 混淆** → **jadx 反编译** → cure 处理
//!   → 编译运行 → 与原始输出逐字节比对（语义保持）
//!
//! 发现（诚实记录）：现代 jadx 对 javac 字节码的重建已较干净，
//! 其残留产物（寄存器式临时变量 i/i2、内联进表达式的 it.next()、
//! 布尔循环旗标）需要**循环级数据流分析**才能消除——这是明确的下一步方向。
//! 源码级混淆模式（模拟混淆器测试 tests/deobfuscate.rs）当前规则全部命中。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const JAVA_HOME: &str = "/opt/homebrew/Cellar/openjdk/27/libexec/openjdk.jdk/Contents/Home";

fn tool_ok(bin: &str) -> bool {
    // 能成功 spawn 即视为存在（proguard 无 -version 选项，报错也说明二进制活着）
    Command::new(bin)
        .env("JAVA_HOME", JAVA_HOME)
        .arg("-version")
        .output()
        .is_ok()
}

fn have_tools() -> bool {
    let ok = tool_ok("proguard") && tool_ok("jadx") && tool_ok("javac") && Path::new(JAVA_HOME).is_dir();
    if !ok {
        eprintln!("skip real_tools: 需要 proguard / jadx / javac / JAVA_HOME={JAVA_HOME}");
    }
    ok
}

const CLEAN_SRC: &str = r#"
import java.util.ArrayList;
import java.util.List;

public class Demo2 {
    public static void main(String[] args) {
        List<String> list = new ArrayList<>();
        list.add("a");
        list.add("bb");
        list.add("ccc");
        int total = 0;
        for (String s : list) {
            total += s.length();
        }
        System.out.println("total=" + total);
        int x = 0;
        while (x < 5) {
            x++;
        }
        System.out.println("x=" + x);
        boolean found = false;
        for (String s : list) {
            if (s.startsWith("b")) {
                found = true;
                break;
            }
        }
        System.out.println("found=" + found);
        int key = 42;
        int hidden = (key ^ 84) ^ 84;
        System.out.println("noise=" + ((hidden | 0) & -1) + 0);
        int r;
        boolean flag = hidden > 10;
        if (flag) {
            r = 1;
        } else {
            r = 2;
        }
        System.out.println("r=" + r);
    }
}
"#;

#[test]
fn proguard_jadx_real_pipeline_preserves_semantics() {
    if !have_tools() {
        return;
    }
    let base = std::env::temp_dir().join("cure_realtools");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let src_file = base.join("Demo2.java");
    fs::write(&src_file, CLEAN_SRC).unwrap();

    macro_rules! run {
        ($bin:expr) => {{
            let mut c = Command::new($bin);
            c.env("JAVA_HOME", JAVA_HOME);
            c
        }};
    }

    // 1. javac + 运行原始
    let out = run!("javac")
        .arg("-nowarn")
        .arg("-d")
        .arg(&base)
        .arg(&src_file)
        .output()
        .unwrap();
    assert!(out.status.success());
    let orig_run = run!("java")
        .arg("-cp")
        .arg(&base)
        .arg("Demo2")
        .output()
        .unwrap();
    let orig_out = String::from_utf8_lossy(&orig_run.stdout).to_string();

    // 2. proguard 混淆
    let jar = base.join("orig.jar");
    let obf = base.join("obf.jar");
    let jar_out = run!("jar")
        .arg("cf")
        .arg(&jar)
        .arg("-C")
        .arg(&base)
        .arg("Demo2.class")
        .output()
        .unwrap();
    assert!(jar_out.status.success(), "jar 打包失败");
    let pro = base.join("pg.pro");
    fs::write(
        &pro,
        format!(
            "-injars {}\n-outjars {}\n-libraryjars {}/jmods/java.base.jmod(!module-info.class)\n-keep public class Demo2 {{ public static void main(java.lang.String[]); }}\n-dontwarn\n-dontnote\n",
            jar.display(),
            obf.display(),
            JAVA_HOME
        ),
    )
    .unwrap();
    let out = run!("proguard")
        .arg(format!("@{}", pro.display()))
        .output()
        .unwrap();
    assert!(out.status.success(), "proguard: {}", String::from_utf8_lossy(&out.stderr));

    // 3. jadx 反编译
    let decomp = base.join("decomp");
    let out = run!("jadx")
        .arg("-d")
        .arg(&decomp)
        .arg("--no-res")
        .arg("-j")
        .arg("2")
        .arg(&obf)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "jadx({}): {}{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let decomp_src = find_java(&decomp);
    let obfuscated_src = fs::read_to_string(&decomp_src).unwrap();
    assert!(
        obfuscated_src.contains("class Demo2"),
        "反编译产物异常：\n{obfuscated_src}"
    );

    // 4. cure 处理
    use cure_engine::Config;
    use cure_java_parser::parse;
    use cure_java_print::print_unit;
    use cure_java_simplify::simplify_unit;
    let mut outcome = parse(&obfuscated_src);
    let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let cured = print_unit(&outcome.ast, &outcome.unit);

    // 5. 编译运行去混淆产物（剥离 defpackage 包名；类保持 Demo2）
    let cured_no_pkg = cured
        .lines()
        .filter(|l| !l.trim_start().starts_with("package "))
        .collect::<Vec<_>>()
        .join("\n");
    let cured_dir = base.join("cured");
    fs::create_dir_all(&cured_dir).unwrap();
    fs::write(cured_dir.join("Demo2.java"), &cured_no_pkg).unwrap();
    let out = run!("javac")
        .arg("-nowarn")
        .arg("-d")
        .arg(&cured_dir)
        .arg(cured_dir.join("Demo2.java"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "cure 产物编译失败：\n{}\n== 源码 ==\n{cured_no_pkg}",
        String::from_utf8_lossy(&out.stderr)
    );
    let cured_run = run!("java")
        .arg("-cp")
        .arg(&cured_dir)
        .arg("Demo2")
        .output()
        .unwrap();
    let cured_out = String::from_utf8_lossy(&cured_run.stdout).to_string();

    // 语义保持：输出必须逐字节一致
    assert_eq!(
        orig_run.status.code(),
        cured_run.status.code(),
        "退出码不一致"
    );
    assert_eq!(
        orig_out, cured_out,
        "真实链路（ProGuard→jadx→cure）语义改变！\n== 原始 ==\n{orig_out}\n== cure 后 ==\n{cured_out}"
    );
    eprintln!(
        "真实工具链通过：javac→ProGuard→jadx→cure，输出一致（{} 行源码，cure 改写 {} 次）",
        obfuscated_src.lines().count(),
        report.edits
    );
}

fn find_java(dir: &PathBuf) -> PathBuf {
    let mut stack = vec![dir.clone()];
    while let Some(d) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&d) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().and_then(|x| x.to_str()) == Some("java") {
                    return p;
                }
            }
        }
    }
    panic!("no java file under {}", dir.display());
}
