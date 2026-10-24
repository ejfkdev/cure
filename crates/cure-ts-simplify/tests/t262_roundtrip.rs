//! Test262 **打印往返结构差分**常驻测试（抽样版）：解析 → 打印（不简化）
//! → 再解析，两棵 AST 必须结构等价（忽略 Paren、字面量拼写、Group 展开）。
//!
//! 语料/环境缺失自动跳过；`T262_ROOT` / `T262_SAMPLE` 覆盖。

use std::path::{Path, PathBuf};

use cure_ts_parser::{parse, parse_module};
use cure_ts_print::print_module;

#[test]
fn t262_print_roundtrip_sample() {
    let root = PathBuf::from(std::env::var("T262_ROOT").unwrap_or("/tmp/test262".into()));
    if !root.join("test").is_dir() {
        eprintln!("skip: 语料缺失（T262_ROOT={}）", root.display());
        return;
    }
    let sample: usize = std::env::var("T262_SAMPLE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1500);
    let mut files = Vec::new();
    walk(&root.join("test"), &mut files);
    files.sort();
    let step = files.len().div_ceil(sample).max(1);
    let files: Vec<_> = files.into_iter().step_by(step).collect();

    let mut checked = 0usize;
    let mut skipped = 0usize;
    let mut failures: Vec<String> = Vec::new();
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else { continue };
        if is_negative(&src) {
            skipped += 1;
            continue;
        }
        let module = is_module(&src);
        let out1 = if module { parse_module(&src) } else { parse(&src) };
        if !out1.errors.is_empty() {
            skipped += 1;
            continue;
        }
        checked += 1;
        let printed = print_module(&out1.ast, &out1.unit);
        let out2 = if module { parse_module(&printed) } else { parse(&printed) };
        let id = path.strip_prefix(&root).unwrap_or(path).to_string_lossy().to_string();
        if !out2.errors.is_empty() {
            failures.push(format!("{id}: printed 不可再解析"));
        } else if !out1.ast.unit_equiv(&out1.unit, &out2.ast, &out2.unit) {
            failures.push(format!("{id}: AST 结构漂移"));
        }
    }
    eprintln!("t262_roundtrip: 抽样 {checked}（跳过 {skipped}），失败 {}", failures.len());
    assert!(checked > 0, "抽样为空：检查 T262_ROOT");
    assert!(
        failures.is_empty(),
        "打印往返结构漂移：\n{}",
        failures.iter().take(20).cloned().collect::<Vec<_>>().join("\n")
    );
}

fn is_module(src: &str) -> bool {
    let Some(start) = src.find("/*---") else { return false };
    let Some(end) = src[start..].find("---*/") else { return false };
    src[start + 5..start + end]
        .split("flags: [")
        .nth(1)
        .and_then(|f| f.split(']').next())
        .is_some_and(|f| f.contains("module"))
}

fn is_negative(src: &str) -> bool {
    let Some(start) = src.find("/*---") else { return false };
    let Some(end) = src[start..].find("---*/") else { return false };
    src[start + 5..start + end].contains("negative:")
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