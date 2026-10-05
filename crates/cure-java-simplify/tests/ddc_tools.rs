//! 真实 ddc 反编译链路集成测试（环境可用时执行，否则跳过）：
//!
//!   干净源码 → javac(--release 17) → d8(→DEX) → **ddc 反编译** → cure 净化
//!   → 编译运行 → 与 ddc 输出的运行结果逐字节比对（语义保持）
//!
//! ddc 的去混淆能力弱于 jadx，输出大量寄存器拷贝 / 语句级 StringBuilder /
//! while(true)+break / 迭代器未还原等产物——正是 cure 的目标素材。
//! 实测：87 行 ddc 输出 → 45 行（-48%），输出行为一致。

use std::fs;
use std::path::Path;
use std::process::Command;

const JAVA_HOME: &str = "/opt/homebrew/Cellar/openjdk/27/libexec/openjdk.jdk/Contents/Home";
const D8: &str = "/Users/e/Library/Android/sdk/build-tools/36.0.0/d8";

fn have_tools() -> bool {
    let ok = Path::new(JAVA_HOME).is_dir()
        && Command::new("ddc").arg("version").output().is_ok()
        && Command::new("javac").arg("-version").output().is_ok()
        && Path::new(D8).exists();
    if !ok {
        eprintln!("skip ddc_tools: 需要 ddc / javac / d8({D8}) / JAVA_HOME");
    }
    ok
}

const CLEAN_SRC: &str = r#"
import java.util.ArrayList;
import java.util.List;

public class Demo3 {
    private static final int LIMIT = 100;

    private int base;

    public Demo3(int base) {
        this.base = base;
    }

    private int scale(int value, int factor) {
        int result = value * factor + base;
        if (result > LIMIT) {
            return LIMIT;
        }
        return result;
    }

    private String label(int value) {
        if (value < 0) {
            return "neg";
        } else if (value == 0) {
            return "zero";
        }
        return value > LIMIT ? "big" : "small";
    }

    public static void main(String[] args) {
        Demo3 d = new Demo3(7);
        int total = 0;
        for (int i = 1; i <= 5; i++) {
            total += d.scale(i, 3);
        }
        System.out.println("total=" + total);
        List<String> names = new ArrayList<>();
        names.add("alpha");
        names.add("beta");
        names.add("gamma");
        for (String n : names) {
            System.out.println(d.label(n.length()) + ":" + n);
        }
        int p;
        p = 11;
        int q;
        q = p;
        System.out.println("copy=" + q);
        boolean flag = total > 50 && total % 2 == 0;
        System.out.println("flag=" + flag);
        System.out.println(d.label(total));
    }
}
"#;

fn run_java(cmd: &mut Command) -> std::process::Output {
    cmd.env("JAVA_HOME", JAVA_HOME).output().unwrap()
}

#[test]
fn ddc_real_pipeline_semantics_preserved() {
    if !have_tools() {
        return;
    }
    let base = std::env::temp_dir().join("cure_ddc_test");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let src_file = base.join("Demo3.java");
    fs::write(&src_file, CLEAN_SRC).unwrap();

    // 1. javac（--release 17：d8 不支持新版 class）
    let out = run_java(Command::new("javac").arg("-nowarn").arg("--release").arg("17").arg("-d").arg(&base).arg(&src_file));
    assert!(out.status.success(), "javac: {}", String::from_utf8_lossy(&out.stderr));

    // 2. d8 → classes.dex
    let out = run_java(Command::new(D8).arg("--release").arg("--output").arg(&base).arg(base.join("Demo3.class")));
    assert!(out.status.success(), "d8: {}", String::from_utf8_lossy(&out.stderr));

    // 3. ddc 反编译
    let decomp = base.join("decomp");
    let out = run_java(Command::new("ddc").arg(base.join("classes.dex")).arg("-o").arg(&decomp));
    assert!(out.status.success(), "ddc: {}", String::from_utf8_lossy(&out.stderr));
    let decomp_src = find_java(&decomp);
    let ddc_src = fs::read_to_string(&decomp_src).unwrap();
    assert!(ddc_src.contains("class Demo3"), "ddc 产物异常");

    // 4. 编译运行 ddc 输出（差分基准——ddc 自身可能有类型保真度损失，
    //    如布尔物化为 int；语义保持的对照对象是 ddc 输出的行为）
    let ddc_dir = base.join("ddc_run");
    fs::create_dir_all(&ddc_dir).unwrap();
    fs::write(ddc_dir.join("Demo3.java"), strip_package(&ddc_src)).unwrap();
    let out = run_java(Command::new("javac").arg("-nowarn").arg("-d").arg(&ddc_dir).arg(ddc_dir.join("Demo3.java")));
    assert!(out.status.success(), "ddc 输出无法编译：\n{}", String::from_utf8_lossy(&out.stderr));
    let ddc_run = run_java(Command::new("java").arg("-cp").arg(&ddc_dir).arg("Demo3"));
    let ddc_out = String::from_utf8_lossy(&ddc_run.stdout).to_string();

    // 5. cure 净化
    use cure_engine::Config;
    use cure_java_parser::parse;
    use cure_java_print::print_unit;
    use cure_java_simplify::simplify_unit;
    let mut outcome = parse(&ddc_src);
    assert!(outcome.errors.is_empty(), "ddc 输出解析失败：{:?}", &outcome.errors[..outcome.errors.len().min(3)]);
    let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let cured = print_unit(&outcome.ast, &outcome.unit);

    // 6. 编译运行净化产物
    let cured_dir = base.join("cured");
    fs::create_dir_all(&cured_dir).unwrap();
    fs::write(cured_dir.join("Demo3.java"), strip_package(&cured)).unwrap();
    let out = run_java(Command::new("javac").arg("-nowarn").arg("-d").arg(&cured_dir).arg(cured_dir.join("Demo3.java")));
    assert!(
        out.status.success(),
        "cure 产物无法编译：\n{}\n== 源码 ==\n{cured}",
        String::from_utf8_lossy(&out.stderr)
    );
    let cured_run = run_java(Command::new("java").arg("-cp").arg(&cured_dir).arg("Demo3"));
    let cured_out = String::from_utf8_lossy(&cured_run.stdout).to_string();

    assert_eq!(ddc_run.status.code(), cured_run.status.code());
    assert_eq!(
        ddc_out, cured_out,
        "ddc→cure 语义改变！\n== ddc ==\n{ddc_out}\n== cure 后 ==\n{cured_out}"
    );
    let ddc_lines = ddc_src.lines().filter(|l| !l.trim().is_empty()).count();
    let cured_lines = cured.lines().filter(|l| !l.trim().is_empty()).count();
    eprintln!(
        "ddc 链路通过：{} 行 ddc 输出 → {} 行（-{}%），{} 次改写，输出一致",
        ddc_lines,
        cured_lines,
        100 - cured_lines * 100 / ddc_lines,
        report.edits
    );
    // 净化必须实际发生（寄存器拷贝等产物被消除）
    assert!(report.edits >= 10, "预期显著净化，实际 {} 次", report.edits);
}

fn strip_package(src: &str) -> String {
    src.lines()
        .filter(|l| !l.trim_start().starts_with("package "))
        .collect::<Vec<_>>()
        .join("\n")
}

fn find_java(dir: &std::path::PathBuf) -> std::path::PathBuf {
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
