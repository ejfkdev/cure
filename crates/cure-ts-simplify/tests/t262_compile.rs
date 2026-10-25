//! Test262 **全量编译差分**常驻测试（抽样版）：每个文件解析 → 简化 →
//! 打印，原版与 cure 输出各自在 node 中编译（script 用 vm.Script，
//! module 用 vm.SourceTextModule；只编译不执行）。
//!
//! 契约：**valid 原版 ⇒ cured 必须仍可编译**。负向语法用例不纳入断言
//! （cure 对非法输入的容错输出可以重新变得可编译，不属于语义保持面）。
//!
//! 语料/环境缺失自动跳过；`T262_ROOT` / `T262_SAMPLE` 覆盖。

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use cure_engine::Config;
use cure_ts_parser::{parse_js as parse, parse_js_module as parse_module};
use cure_ts_print::print_module;
use cure_ts_simplify::simplify_unit;

#[test]
fn t262_compile_differential_sample() {
    let root = PathBuf::from(std::env::var("T262_ROOT").unwrap_or("/tmp/test262".into()));
    if !root.join("test").is_dir() {
        eprintln!("skip: 语料缺失（T262_ROOT={}）", root.display());
        return;
    }
    if Command::new("node").arg("--version").output().is_err() {
        eprintln!("skip: 找不到 node");
        return;
    }
    let sample: usize = std::env::var("T262_SAMPLE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1200);
    let mut files = Vec::new();
    walk(&root.join("test"), &mut files);
    files.sort();
    let step = files.len().div_ceil(sample).max(1);
    let files: Vec<_> = files.into_iter().step_by(step).collect();

    struct T {
        id: String,
        orig: String,
        cured: String,
        module: bool,
    }
    let mut tests = Vec::new();
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else { continue };
        let module = src
            .split("flags: [")
            .nth(1)
            .and_then(|s| s.split(']').next())
            .is_some_and(|f| f.contains("module"));
        let mut out = if module { parse_module(&src) } else { parse(&src) };
        let _ = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
        let cured = print_module(&out.ast, &out.unit);
        let id = path.strip_prefix(&root).unwrap_or(path).to_string_lossy().to_string();
        tests.push(T { id, orig: src, cured, module });
    }
    assert!(!tests.is_empty());

    let driver = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/t262_compile_driver.mjs");
    let mut child = Command::new("node")
        .arg("--experimental-vm-modules")
        .arg(driver)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn node");
    {
        let mut stdin = child.stdin.take().unwrap();
        for t in &tests {
            let line = format!(
                "{{\"id\":\"{}\",\"orig\":\"{}\",\"cured\":\"{}\",\"module\":{}}}\n",
                esc(&t.id),
                esc(&t.orig),
                esc(&t.cured),
                t.module
            );
            if stdin.write_all(line.as_bytes()).is_err() {
                break;
            }
        }
        let _ = stdin.flush();
    }

    let mut ok = 0usize;
    let mut broken: Vec<(String, String)> = Vec::new();
    for line in BufReader::new(child.stdout.take().unwrap()).lines().flatten() {
        let Some((id, o, c)) = rec(&line) else { continue };
        if o == "ok" {
            if c == "ok" {
                ok += 1;
            } else {
                broken.push((id, c));
            }
        }
    }
    let status = child.wait().expect("wait node");
    assert!(status.success(), "node driver 失败: {status}");
    assert!(ok * 2 >= tests.len(), "覆盖过低：ok={ok} tests={}", tests.len());
    eprintln!("t262_compile: valid 编译通过 {ok} / cured 编译失败 {}", broken.len());
    if !broken.is_empty() {
        for (id, c) in broken.iter().take(10) {
            eprintln!("--- BROKEN {id} :: cured={c}");
        }
        panic!("{} 个 valid 文件被改坏", broken.len());
    }
}

fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o
}

fn rec(line: &str) -> Option<(String, String, String)> {
    fn f(s: &str, key: &str) -> Option<String> {
        let pat = format!("\"{key}\":\"");
        let i = s.find(&pat)? + pat.len();
        let mut out = String::new();
        let mut it = s[i..].chars();
        while let Some(c) = it.next() {
            match c {
                '"' => return Some(out),
                '\\' => match it.next()? {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    o => out.push(o),
                },
                c => out.push(c),
            }
        }
        None
    }
    Some((f(line, "id")?, f(line, "orig")?, f(line, "cured")?))
}

fn walk(d: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(d) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x == "js") {
            out.push(p);
        }
    }
}
