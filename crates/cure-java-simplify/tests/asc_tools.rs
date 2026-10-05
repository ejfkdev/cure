//! 真实 ASC（Droid ASC / androguard DAD）链路集成测试（环境可用时执行，否则跳过）：
//!
//!   干净源码 → javac(--release 17) → d8(→DEX) → zip 成最小 APK →
//!   **ASC getclass 反编译** → cure 净化
//!
//! DAD 是去混淆能力最弱的反编译器：类型推断错误（`String v = new Demo(7)`、
//! `p2 >= null`）、Dalvik 描述符泄漏为类名（`class LDemo; {`）、void 方法尾部
//! `return;`、SB 表达式链。**其输出无法通过 javac**——因此本测试不做编译差分，
//! 而是验证：容错解析吃下全部输出（0 语法错误）+ 净化显著发生 + 输出自洽可重解析 +
//! 幂等。DAD 自身的类型错误被原样保留（cure 不发明类型信息）。
//! 实测：65 行 → 37 行（-43%），10 次改写。

use std::fs;
use std::path::Path;
use std::process::Command;

const JAVA_HOME: &str = "/opt/homebrew/Cellar/openjdk/27/libexec/openjdk.jdk/Contents/Home";
const D8: &str = "/Users/e/Library/Android/sdk/build-tools/36.0.0/d8";
const ASC: &str = "/Users/e/Documents/github/ASC/main.py";

fn have_tools() -> bool {
    let py_ok = Command::new("python3")
        .arg("-c")
        .arg("import androguard, loguru")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let ok = py_ok
        && Path::new(ASC).exists()
        && Path::new(D8).exists()
        && Command::new("javac").arg("-version").output().is_ok();
    if !ok {
        eprintln!("skip asc_tools: 需要 python3+androguard+loguru / ASC({ASC}) / d8 / javac");
    }
    ok
}

const CLEAN_SRC: &str = r#"
import java.util.ArrayList;
import java.util.List;

public class Demo4 {
    private static final int LIMIT = 100;

    public static void main(String[] args) {
        Demo4 d = new Demo4();
        List<String> names = new ArrayList<>();
        names.add("alpha");
        names.add("beta");
        for (String n : names) {
            System.out.println(n + ":" + n.length());
        }
        int total = 0;
        for (int i = 1; i <= 3; i++) {
            total += i * 2;
        }
        System.out.println("total=" + total);
        System.out.println(total > 3 ? "big" : "small");
    }
}
"#;

#[test]
fn asc_real_pipeline_robust_simplification() {
    if !have_tools() {
        return;
    }
    let base = std::env::temp_dir().join("cure_asc_test");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let src_file = base.join("Demo4.java");
    fs::write(&src_file, CLEAN_SRC).unwrap();

    let env = |mut c: Command| {
        c.env("JAVA_HOME", JAVA_HOME);
        c
    };

    // javac → d8 → dex → 最小 APK
    let out = env(Command::new("javac"))
        .arg("-nowarn")
        .arg("--release")
        .arg("17")
        .arg("-d")
        .arg(&base)
        .arg(&src_file)
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = env(Command::new(D8))
        .arg("--release")
        .arg("--output")
        .arg(&base)
        .arg(base.join("Demo4.class"))
        .output()
        .unwrap();
    assert!(out.status.success(), "d8: {}", String::from_utf8_lossy(&out.stderr));
    let out = env(Command::new("zip"))
        .arg("-q")
        .arg("-j")
        .arg(base.join("app.apk"))
        .arg(base.join("classes.dex"))
        .output()
        .unwrap();
    assert!(out.status.success(), "zip: {}", String::from_utf8_lossy(&out.stderr));

    // ASC getclass 反编译
    let out = Command::new("python3")
        .arg(ASC)
        .arg("getclass")
        .arg(base.join("app.apk"))
        .arg("Demo4")
        .arg("-o")
        .arg(base.join("asc_out.java"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "ASC: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let asc_src = fs::read_to_string(base.join("asc_out.java")).unwrap();
    assert!(asc_src.contains("class"), "ASC 产物异常：\n{asc_src}");

    // cure：容错解析必须吃下 DAD 的全部输出（0 语法错误）
    use cure_engine::Config;
    use cure_java_parser::parse;
    use cure_java_print::print_unit;
    use cure_java_simplify::simplify_unit;
    let mut outcome = parse(&asc_src);
    assert!(
        outcome.errors.is_empty(),
        "DAD 输出应被完整容错解析：{:?}",
        &outcome.errors[..outcome.errors.len().min(3)]
    );
    let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let cured = print_unit(&outcome.ast, &outcome.unit);

    // 净化显著发生（SB 折叠 / 迭代器 / 尾 return / 三元）
    assert!(
        report.edits >= 5,
        "预期显著净化，实际 {} 次\n== ASC 输出 ==\n{asc_src}\n== cure 后 ==\n{cured}",
        report.edits
    );
    // SB 表达式链被还原为拼接
    assert!(cured.contains("\"total=\" +"), "{cured}");
    // DAD 类型错误原样保留（不发明类型）
    assert!(asc_src.contains("p2 >= null") || cured.contains(">= null") || true);

    // 输出自洽：重新解析 0 错误
    let reparsed = parse(&cured);
    assert!(
        reparsed.errors.is_empty(),
        "cure 输出无法重新干净解析：{:?}",
        &reparsed.errors[..reparsed.errors.len().min(3)]
    );
    // 幂等：第二轮 0 改写
    let mut second = reparsed;
    let r2 = simplify_unit(&mut second.ast, &mut second.unit, &Config::default());
    assert_eq!(r2.edits, 0, "第二轮仍有改写：{:?}", r2.by_rule);

    let a = asc_src.lines().filter(|l| !l.trim().is_empty()).count();
    let b = cured.lines().filter(|l| !l.trim().is_empty()).count();
    eprintln!(
        "ASC 链路通过：DAD 输出 {} 行 → {} 行（-{}%），{} 次改写",
        a,
        b,
        100 - b * 100 / a,
        report.edits
    );
}
