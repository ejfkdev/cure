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
    let out = run_java(Command::new("javac").arg("-encoding").arg("UTF-8").arg("-nowarn").arg("--release").arg("17").arg("-d").arg(&base).arg(&src_file));
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
    let out = run_java(Command::new("javac").arg("-encoding").arg("UTF-8").arg("-nowarn").arg("-d").arg(&ddc_dir).arg(ddc_dir.join("Demo3.java")));
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
    let out = run_java(Command::new("javac").arg("-encoding").arg("UTF-8").arg("-nowarn").arg("-d").arg(&cured_dir).arg(cured_dir.join("Demo3.java")));
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

// ---------------------------------------------------------------------------
// 双层数混淆：刁钻源级混淆（多层常量隐藏/嵌套不透明谓词/SB 语句链/寄存器回拷）
// → javac（常量层被编译器折叠，结构性混淆存活）→ d8 → ddc（叠加寄存器伪影）
// → cure 净化 → 编译运行比对。
// 实测：80 → 53 行，15 次改写，输出与 ddc/原始双重一致。
// ---------------------------------------------------------------------------

const HARD_SRC: &str = r#"
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
}"#;

#[test]
fn ddc_dual_layer_obfuscation() {
    if !have_tools() {
        return;
    }
    let base = std::env::temp_dir().join("cure_ddc_hard");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let src_file = base.join("HardObf.java");
    fs::write(&src_file, HARD_SRC).unwrap();

    // javac → d8 → ddc
    let out = run_java(Command::new("javac").arg("-encoding").arg("UTF-8").arg("-nowarn").arg("--release").arg("17").arg("-d").arg(&base).arg(&src_file));
    assert!(out.status.success());
    let out = run_java(Command::new(D8).arg("--release").arg("--output").arg(&base).arg(base.join("HardObf.class")));
    assert!(out.status.success());
    let decomp = base.join("decomp");
    let out = run_java(Command::new("ddc").arg(base.join("classes.dex")).arg("-o").arg(&decomp));
    assert!(out.status.success());
    let decomp_src = fs::read_to_string(find_java(&decomp)).unwrap();

    // ddc 行为基准
    let ddc_dir = base.join("ddc_run");
    fs::create_dir_all(&ddc_dir).unwrap();
    fs::write(ddc_dir.join("HardObf.java"), strip_package(&decomp_src)).unwrap();
    let out = run_java(Command::new("javac").arg("-encoding").arg("UTF-8").arg("-nowarn").arg("-d").arg(&ddc_dir).arg(ddc_dir.join("HardObf.java")));
    assert!(out.status.success());
    let ddc_run = run_java(Command::new("java").arg("-cp").arg(&ddc_dir).arg("HardObf"));
    let ddc_out = String::from_utf8_lossy(&ddc_run.stdout).to_string();

    // cure
    use cure_engine::Config;
    use cure_java_parser::parse;
    use cure_java_print::print_unit;
    use cure_java_simplify::simplify_unit;
    let mut outcome = parse(&decomp_src);
    assert!(outcome.errors.is_empty(), "双层数 ddc 输出解析失败");
    let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let cured = print_unit(&outcome.ast, &outcome.unit);

    let cured_dir = base.join("cured");
    fs::create_dir_all(&cured_dir).unwrap();
    fs::write(cured_dir.join("HardObf.java"), strip_package(&cured)).unwrap();
    let out = run_java(Command::new("javac").arg("-encoding").arg("UTF-8").arg("-nowarn").arg("-d").arg(&cured_dir).arg(cured_dir.join("HardObf.java")));
    assert!(out.status.success(), "cure 产物编译失败：\n{}\n{cured}", String::from_utf8_lossy(&out.stderr));
    let cured_run = run_java(Command::new("java").arg("-cp").arg(&cured_dir).arg("HardObf"));
    let cured_out = String::from_utf8_lossy(&cured_run.stdout).to_string();

    assert_eq!(ddc_run.status.code(), cured_run.status.code());
    assert_eq!(ddc_out, cured_out, "双层数语义改变！\nddc={ddc_out}\ncured={cured_out}");
    let a = decomp_src.lines().filter(|l| !l.trim().is_empty()).count();
    let b = cured.lines().filter(|l| !l.trim().is_empty()).count();
    eprintln!(
        "双层数链路通过：ddc 输出 {} 行 → {} 行（-{}%），{} 次改写",
        a,
        b,
        100 - b * 100 / a,
        report.edits
    );
    assert!(report.edits >= 10, "双层数预期显著净化，实际 {}", report.edits);
}
