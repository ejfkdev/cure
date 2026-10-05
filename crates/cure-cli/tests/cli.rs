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
