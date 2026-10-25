//! Test262 **打印往返结构差分**：解析 → 打印（不简化）→ 再解析，比较
//! 两棵 AST 的结构等价（忽略 Paren、字面量拼写与原文空白）。
//!
//! 这层专门抓“打印器改变程序形态”的问题：例如 `await super.close()`
//! 被打印成 `await (super.close)()`，二轮按标识符调用解析——语义可能
//! 侥幸不变，但 AST 形态已经漂移。
//!
//! 用法：`t262_roundtrip [样本数]`（默认 2000；0 = 全量）
//! 环境：`T262_ROOT`、`T262_OFFSET`、`T262_LIMIT`、`T262_DUMP`（失败样本落盘目录）。

use std::path::{Path, PathBuf};

use cure_ts_parser::{
    parse_js as parse, parse_js_module as parse_module,
    parse_js_module_with_source as parse_module_with_source,
    parse_js_with_source as parse_with_source,
};
use cure_ts_print::print_module;

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
    let (mut ok, mut ast_diff, mut bad_print, mut skipped) = (0usize, 0usize, 0usize, 0usize);
    let mut examples: Vec<(String, &'static str)> = Vec::new();
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else { continue };
        if is_negative(&src) {
            skipped += 1;
            continue;
        }
        let module = is_module(&src);
        let mut out1 = if module {
            if preserve {
                parse_module_with_source(&src)
            } else {
                parse_module(&src)
            }
        } else if preserve {
            parse_with_source(&src)
        } else {
            parse(&src)
        };
        if preserve {
            out1.ast.set_preserve_source(true);
        }
        if !out1.errors.is_empty() {
            skipped += 1;
            continue;
        }
        let printed = print_module(&out1.ast, &out1.unit);
        let out2 = if module { parse_module(&printed) } else { parse(&printed) };
        let id = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        if !out2.errors.is_empty() {
            bad_print += 1;
            if examples.len() < 20 {
                examples.push((id.clone(), "printed 不可再解析"));
            }
            dump_failure(dump.as_deref(), &id, &src, &printed);
            continue;
        }
        if !out1.ast.unit_equiv(&out1.unit, &out2.ast, &out2.unit) {
            ast_diff += 1;
            if examples.len() < 20 {
                examples.push((id.clone(), "AST 结构漂移"));
            }
            dump_failure(dump.as_deref(), &id, &src, &printed);
        } else {
            ok += 1;
        }
    }

    eprintln!(
        "t262_roundtrip: 语料 {total} 文件 → 采样 {} → 检查 {}（解析错误跳过 {skipped}）",
        files.len(),
        ok + ast_diff + bad_print
    );
    eprintln!(
        "结果：一致 {ok} / AST 不一致 {ast_diff} / printed 不可解析 {bad_print}"
    );
    for (id, why) in &examples {
        eprintln!("--- {why}: {id}");
    }
    if ast_diff + bad_print > 0 {
        std::process::exit(1);
    }
}

fn dump_failure(dir: Option<&Path>, id: &str, src: &str, printed: &str) {
    let Some(dir) = dir else { return };
    let safe: String = id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let _ = std::fs::create_dir_all(dir);
    let _ = std::fs::write(dir.join(format!("{safe}.orig.js")), src);
    let _ = std::fs::write(dir.join(format!("{safe}.printed.js")), printed);
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