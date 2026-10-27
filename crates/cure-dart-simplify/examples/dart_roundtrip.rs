//! Dart 语料格式往返：parse → print → parse，token 序列必须逐 token 相同。
//!
//! 这个检查比 `dart format` 更严格地验证“格式化不改代码”：注释、字符串
//! 原文、运算符、换行以外的空白全部保留。环境：DART_ROOT / OFFSET / LIMIT。

use std::path::{Path, PathBuf};

use cure_dart_parser::parse;
use cure_dart_print::print_module;

fn main() {
    let root = PathBuf::from(std::env::var("DART_ROOT").unwrap_or_else(|_| {
        format!(
            "{}/.pub-cache/hosted/pub.dev",
            std::env::var("HOME").unwrap_or_default()
        )
    }));
    let offset: usize = std::env::var("DART_OFFSET")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let limit: usize = std::env::var("DART_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let mut files = Vec::new();
    walk(&root, &mut files);
    files.sort();
    let total = files.len();
    let files: Vec<PathBuf> = files
        .into_iter()
        .skip(offset)
        .take(if limit == 0 { usize::MAX } else { limit })
        .collect();
    let mut bad_tokens = 0usize;
    let mut bad_idem = 0usize;
    let mut diagnostics = 0usize;
    let mut printed_files = 0usize;
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        let out = parse(&src);
        if !out.errors.is_empty() {
            diagnostics += 1;
        }
        let printed = print_module(&out.ast);
        let out2 = parse(&printed);
        if out.ast.tokens.len() != out2.ast.tokens.len()
            || out
                .ast
                .tokens
                .iter()
                .zip(&out2.ast.tokens)
                .any(|(a, b)| a.kind != b.kind || a.text != b.text)
        {
            bad_tokens += 1;
            if bad_tokens <= 5 {
                eprintln!("TOKEN-DRIFT {}", path.display());
                for (k, (a, b)) in out
                    .ast
                    .tokens
                    .iter()
                    .zip(&out2.ast.tokens)
                    .enumerate()
                {
                    if a.kind != b.kind || a.text != b.text {
                        eprintln!(
                            "  #{k} line {}: {:?} {:?} -> {:?} {:?}",
                            a.line, a.kind, a.text, b.kind, b.text
                        );
                        if k > 0 {
                            let p = &out.ast.tokens[k - 1];
                            eprintln!("     prev: {:?} {:?}", p.kind, p.text);
                        }
                        break;
                    }
                }
            }
        }
        let printed2 = print_module(&out2.ast);
        if printed != printed2 {
            bad_idem += 1;
            if bad_idem <= 20 {
                eprintln!("IDEM-DRIFT {}", path.display());
            }
        }
        printed_files += 1;
    }
    eprintln!(
        "dart_roundtrip: 语料 {total} → 本块 {printed_files} 文件；解析诊断 {diagnostics}；token 漂移 {bad_tokens}；格式漂移 {bad_idem}"
    );
    if bad_tokens != 0 || bad_idem != 0 {
        std::process::exit(1);
    }
}

fn walk(d: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(d) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x == "dart") {
            out.push(p);
        }
    }
}