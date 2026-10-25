//! Test262 **简化+打印幂等差分**：cure 输出再跑一遍 cure，文本必须
//! 逐字节一致（第一轮产物即不动点）。
//!
//! 用法：`t262_idem [样本数]`（默认 2000；0 = 全量）
//! 环境：`T262_ROOT`、`T262_OFFSET`、`T262_LIMIT`、`T262_DUMP`。

use std::path::{Path, PathBuf};

use cure_engine::Config;
use cure_ts_parser::{
    parse_js as parse, parse_js_module as parse_module,
    parse_js_module_with_source as parse_module_with_source,
    parse_js_with_source as parse_with_source,
};
use cure_ts_print::print_module;
use cure_ts_simplify::simplify_unit;

fn main() {
    let root = PathBuf::from(std::env::var("T262_ROOT").unwrap_or("/tmp/test262".into()));
    let sample: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(2000);
    let mut files = Vec::new();
    walk(&root.join("test"), &mut files);
    files.sort();
    let total = files.len();
    let step = if sample == 0 || sample >= total {
        1
    } else {
        total.div_ceil(sample)
    };
    let mut files: Vec<PathBuf> = files.into_iter().step_by(step).collect();
    let offset: usize = std::env::var("T262_OFFSET")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let limit: usize = std::env::var("T262_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if offset > 0 {
        files.drain(..offset.min(files.len()));
    }
    if limit > 0 {
        files.truncate(limit);
    }

    let preserve = std::env::var_os("T262_PRESERVE").is_some();
    let dump = std::env::var("T262_DUMP").ok().map(PathBuf::from);
    let (mut ok, mut diff, mut bad_reparse, mut skipped) = (0usize, 0usize, 0usize, 0usize);
    let mut examples: Vec<(String, &'static str)> = Vec::new();
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else { continue };
        if is_negative(&src) {
            skipped += 1;
            continue;
        }
        let module = is_module(&src);
        let parse_mode = |text: &str| {
            if module {
                if preserve {
                    parse_module_with_source(text)
                } else {
                    parse_module(text)
                }
            } else if preserve {
                parse_with_source(text)
            } else {
                parse(text)
            }
        };
        let mut cfg = Config::default();
        if preserve {
            cfg.preserve_function_source = true;
        }
        let mut out1 = parse_mode(&src);
        if !out1.errors.is_empty() {
            skipped += 1;
            continue;
        }
        if preserve {
            out1.ast.set_preserve_source(true);
        }
        let _ = simplify_unit(&mut out1.ast, &mut out1.unit, &cfg);
        let once = print_module(&out1.ast, &out1.unit);

        let mut out2 = parse_mode(&once);
        let id = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        if !out2.errors.is_empty() {
            bad_reparse += 1;
            if examples.len() < 20 {
                examples.push((id.clone(), "cure 输出不可再解析"));
            }
            dump_failure(dump.as_deref(), &id, &once, "");
            continue;
        }
        if preserve {
            out2.ast.set_preserve_source(true);
        }
        let _ = simplify_unit(&mut out2.ast, &mut out2.unit, &cfg);
        let twice = print_module(&out2.ast, &out2.unit);
        if once != twice {
            diff += 1;
            if examples.len() < 20 {
                examples.push((id.clone(), "二次 cure 文本漂移"));
            }
            dump_failure(dump.as_deref(), &id, &once, &twice);
        } else {
            ok += 1;
        }
    }

    eprintln!(
        "t262_idem: 语料 {total} 文件 → 采样 {} → 检查 {}（解析错误跳过 {skipped}）",
        files.len(),
        ok + diff + bad_reparse
    );
    eprintln!("结果：稳定 {ok} / 文本漂移 {diff} / 不可再解析 {bad_reparse}");
    for (id, why) in &examples {
        eprintln!("--- {why}: {id}");
    }
    if diff + bad_reparse > 0 {
        std::process::exit(1);
    }
}

fn dump_failure(dir: Option<&Path>, id: &str, once: &str, twice: &str) {
    let Some(dir) = dir else { return };
    let safe: String = id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let _ = std::fs::create_dir_all(dir);
    let _ = std::fs::write(dir.join(format!("{safe}.once.js")), once);
    if !twice.is_empty() {
        let _ = std::fs::write(dir.join(format!("{safe}.twice.js")), twice);
    }
}

/// 负向语法用例不参与打印/幂等契约（原文件本就不该解析成功）。
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