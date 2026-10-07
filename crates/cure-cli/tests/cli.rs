//! CLI 端到端测试（真实进程调用）。

use std::process::Command;

use assert_cmd::Command as AssertCommand;
use predicates::prelude::*;
use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_cure");

fn cure() -> AssertCommand {
    AssertCommand::new(BIN)
}

fn tmp_java(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let p = dir.path().join(name);
    std::fs::write(&p, content).unwrap();
    p
}

#[test]
fn help_and_version() {
    cure()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("cure"));
    cure()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("cure"));
}

#[test]
fn stdin_to_stdout() {
    let src = "class A{int m(){int a=foo();int b=a;return b;}}";
    cure()
        .arg("-")
        .write_stdin(src)
        .assert()
        .success()
        .stdout(predicate::str::contains("return foo();"))
        .stdout(predicate::str::contains("class A {"));
}

#[test]
fn file_to_stdout() {
    let dir = TempDir::new().unwrap();
    let p = tmp_java(
        &dir,
        "A.java",
        r#"
class A {
    boolean check(int v) {
        boolean b = v > 10;
        if (b) {
            return true;
        } else {
            return false;
        }
    }
}
"#,
    );
    cure()
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("return v > 10;"));
}

#[test]
fn broken_source_still_succeeds_and_skips() {
    let dir = TempDir::new().unwrap();
    let p = tmp_java(
        &dir,
        "Bad.java",
        r#"
class Bad {
    void broken() {
        int x = ;
    }
    int good() {
        int a = foo();
        int b = a;
        return b;
    }
}
"#,
    );
    cure()
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("return foo();"))
        .stdout(predicate::str::contains("int x = ;"))
        .stderr(predicate::str::contains("语法问题已跳过"));
}

#[test]
fn strict_mode_fails_on_errors() {
    let dir = TempDir::new().unwrap();
    let p = tmp_java(&dir, "Bad.java", "class {{{ totally broken");
    cure()
        .arg("--strict")
        .arg(p)
        .assert()
        .failure()
        .code(1);
}

#[test]
fn check_mode_exit_code() {
    let dir = TempDir::new().unwrap();
    // 有可简化内容 → 退出码 2
    let dirty = tmp_java(&dir, "Dirty.java", "class A{int m(){return 1+0;}}");
    cure()
        .arg("--check")
        .arg(&dirty)
        .assert()
        .code(2);
    // 干净代码 → 0
    let clean = tmp_java(&dir, "Clean.java", "class A {\n    int m() {\n        return 1;\n    }\n}\n");
    cure()
        .arg("--check")
        .arg(clean)
        .assert()
        .success();
}

#[test]
fn write_in_place() {
    let dir = TempDir::new().unwrap();
    let p = tmp_java(&dir, "A.java", "class A{int m(){int a=1;int b=a;return b;}}");
    cure()
        .arg("--write")
        .arg(&p)
        .assert()
        .success();
    let after = std::fs::read_to_string(&p).unwrap();
    assert!(after.contains("return 1;"), "{after}");
}

#[test]
fn format_only() {
    let dir = TempDir::new().unwrap();
    // 1+0 不化简，仅重排格式
    let p = tmp_java(&dir, "F.java", "class F{int m(){return 1+0;}}");
    cure()
        .arg("--format-only")
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("return 1 + 0;"))
        .stdout(predicate::str::contains("class F {"));
}

#[test]
fn disable_rule() {
    let dir = TempDir::new().unwrap();
    let p = tmp_java(&dir, "D.java", "class D{int m(){int a=5;return a;}}");
    cure()
        .arg("--disable")
        .arg("local_propagation")
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("int a = 5;"));
}

#[test]
fn report_flag() {
    let dir = TempDir::new().unwrap();
    let p = tmp_java(&dir, "R.java", "class R{int m(){int a=5;return a;}}");
    cure()
        .arg("--report")
        .arg(p)
        .assert()
        .success()
        .stderr(predicate::str::contains("local_propagation"));
}

#[test]
fn invalid_option() {
    cure()
        .arg("--nope")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("未知选项"));
}

#[test]
fn real_invocation_via_command() {
    // 兼容 std::process::Command 的直接调用（供外部脚本使用）
    let out = Command::new(BIN)
        .arg("--version")
        .output()
        .expect("spawn cure");
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("cure "));
}

// ---- CLI v2（ddc 约定）：帮助路由 / 子命令 / 默认输出后缀 ----

#[test]
fn no_args_prints_help_and_exits_zero() {
    cure()
        .env("CURE_LANG", "en")
        .assert()
        .success()
        .stdout(predicate::str::contains("https://github.com/ejfkdev/cure"))
        .stdout(predicate::str::contains("Examples:"));
}

#[test]
fn help_word_and_flag_are_equivalent() {
    for arg in ["help", "-h", "--help"] {
        cure()
            .env("CURE_LANG", "en")
            .arg(arg)
            .assert()
            .success()
            .stdout(predicate::str::contains("cure"))
            .stdout(predicate::str::contains("https://github.com/ejfkdev/cure"))
            .stdout(predicate::str::contains("Examples:"));
    }
}

#[test]
fn help_zh_follows_cure_lang() {
    cure()
        .env("CURE_LANG", "zh")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("代码简化 / 格式化器"))
        .stdout(predicate::str::contains("示例:"));
}

#[test]
fn version_word_and_flag() {
    let want = format!("cure {}", env!("CARGO_PKG_VERSION"));
    for arg in ["version", "-V", "--version"] {
        cure()
            .env("CURE_LANG", "en")
            .arg(arg)
            .assert()
            .success()
            .stdout(predicate::str::contains(want.as_str()))
            .stdout(predicate::str::contains("https://github.com/ejfkdev/cure"));
    }
}

#[test]
fn rules_subcommand_lists_catalog() {
    for args in [&["rules"][..], &["help", "rules"][..], &["rules", "-h"][..]] {
        cure()
            .env("CURE_LANG", "en")
            .args(args)
            .assert()
            .success()
            .stdout(predicate::str::contains("simplification rules"))
            .stdout(predicate::str::contains("const_fold_bin"))
            .stdout(predicate::str::contains("cff_recover"))
            // 默认/选配分组与 --disable 提示
            .stdout(predicate::str::contains("Enabled by default"))
            .stdout(predicate::str::contains("--disable"));
    }
}

#[test]
fn unknown_help_topic_is_usage_error() {
    cure()
        .env("CURE_LANG", "en")
        .args(["help", "definitely-not-a-topic"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("unknown help topic"));
}

#[test]
fn default_dir_output_suffix_is_cure_out() {
    let parent = TempDir::new().unwrap();
    let src = parent.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("A.java"), "class A{int m(){return 1;}}").unwrap();

    cure()
        .arg(&src)
        .assert()
        .success();

    let out_root = parent.path().join("src-cure-out");
    assert!(out_root.is_dir(), "默认输出根应为 {out_root:?}");
    let out = out_root.join("A.java");
    assert!(out.is_file());
    assert_eq!(std::fs::read_to_string(&out).unwrap().trim_end(), "class A {\n    int m() {\n        return 1;\n    }\n}");
    // 旧后缀不再产生
    assert!(!parent.path().join("src-cure").exists());
}

#[test]
fn broken_pipe_exits_quietly() {
    use std::io::Write;
    let dir = TempDir::new().unwrap();
    let p = tmp_java(&dir, "A.java", "class A{int m(){int a=foo();int b=a;return b;}}");
    // head 提前关闭管道：不应 panic（"failed printing to stdout"）
    let out = Command::new("sh")
        .arg("-c")
        .arg(format!("{BIN} {:?} | head -1", p.as_os_str()))
        .output()
        .unwrap();
    assert!(out.status.success(), "pipe exit: {}", out.status);
    assert!(!String::from_utf8_lossy(&out.stderr).contains("failed printing"));
}

#[test]
fn single_file_with_unimplemented_ext_is_clear_error() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("foo.rs");
    std::fs::write(&p, "fn main() {}").unwrap();
    cure()
        .env("CURE_LANG", "en")
        .arg(&p)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("no language backend"))
        .stderr(predicate::str::contains("java"));
    // 无扩展名的单文件：回退首个后端（Java 管线），不报错
    let p2 = dir.path().join("noext");
    std::fs::write(&p2, "class A{int m(){return 1;}}").unwrap();
    cure()
        .arg(&p2)
        .assert()
        .success()
        .stdout(predicate::str::contains("return 1;"));
}

// ---- 简化效果指标（--stats：节点/判定点/嵌套 + 文件分类）----

#[test]
fn stats_reports_structural_metrics_and_classification() {
    // 结构简化文件（恒真谓词 if + 逆运算噪声对）
    let dir = TempDir::new().unwrap();
    let p = tmp_java(
        &dir,
        "Noise.java",
        "class Noise{int f(){int a=0;if(2<3){a=1;}a+=1;a-=1;return a;}}",
    );
    let out = cure()
        .env("CURE_LANG", "zh")
        .arg("--stats")
        .arg(&p)
        .assert()
        .success()
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("节点"), "{stderr}");
    assert!(stderr.contains("判定点"), "{stderr}");
    assert!(stderr.contains("简化质量：结构简化 1"), "{stderr}");
    assert!(
        stderr.contains("判定点 1 → 0"),
        "if(常量谓词) 折叠应减判定点: {stderr}"
    );

    // 纯格式文件：无可简化结构（无规则命中）→ 分类为「仅格式」
    let p2 = tmp_java(&dir, "Fmt.java", "class Fmt{int m(){return foo();}}");
    let out2 = cure()
        .env("CURE_LANG", "zh")
        .arg("--stats")
        .arg(&p2)
        .assert()
        .success()
        .get_output()
        .clone();
    let stderr2 = String::from_utf8_lossy(&out2.stderr);
    assert!(
        stderr2.contains("简化质量：结构简化 0 / 仅格式 1 / 未变 0"),
        "{stderr2}"
    );
    assert!(stderr2.contains("节点 4 → 4"), "{stderr2}");
}

#[test]
fn metrics_unit_counting() {
    // 引擎侧 TreeMetrics 精确性：判定点 = if + while + for + case + && + 三元
    let mut out = cure_java_parser::parse(
        "class A{int f(int x){if(x>0){x=1;}while(x<3){x=x+1;}for(int i=0;i<2;i++){}\
         switch(x){case 1:break;default:break;}return x>0?1:(x&&true?2:3);}}",
    );
    let before = cure_java_ast::unit_metrics(&out.ast, &out.unit);
    // if=1 while=1 for=1 case×2(default)=2 &&=1 ?:×2=2 → 8
    assert_eq!(before.decisions, 8, "{before:?}");
    assert!(before.nodes > 40, "{before:?}");
    assert!(before.max_depth >= 2, "{before:?}");
    let _ = &mut out;
}
