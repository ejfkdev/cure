//! Test262 **简化+打印幂等**常驻测试（抽样版）：cure 输出再跑一遍 cure，
//! 文本必须逐字节一致。
//!
//! 语料/环境缺失自动跳过；`T262_ROOT` / `T262_SAMPLE` 覆盖。

use std::path::{Path, PathBuf};

use cure_engine::Config;
use cure_ts_parser::{parse_js as parse, parse_js_module as parse_module};
use cure_ts_print::print_module;
use cure_ts_simplify::simplify_unit;

#[test]
fn t262_simplify_idempotent_sample() {
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
        let mut out1 = if module { parse_module(&src) } else { parse(&src) };
        if !out1.errors.is_empty() {
            skipped += 1;
            continue;
        }
        checked += 1;
        let _ = simplify_unit(&mut out1.ast, &mut out1.unit, &Config::default());
        let once = print_module(&out1.ast, &out1.unit);
        let mut out2 = if module { parse_module(&once) } else { parse(&once) };
        let id = path.strip_prefix(&root).unwrap_or(path).to_string_lossy().to_string();
        if !out2.errors.is_empty() {
            failures.push(format!("{id}: cure 输出不可再解析"));
            continue;
        }
        let _ = simplify_unit(&mut out2.ast, &mut out2.unit, &Config::default());
        let twice = print_module(&out2.ast, &out2.unit);
        if once != twice {
            failures.push(format!("{id}: 二次 cure 文本漂移"));
        }
    }
    eprintln!("t262_idem: 抽样 {checked}（跳过 {skipped}），失败 {}", failures.len());
    assert!(checked > 0, "抽样为空：检查 T262_ROOT");
    assert!(
        failures.is_empty(),
        "简化幂等失败：\n{}",
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