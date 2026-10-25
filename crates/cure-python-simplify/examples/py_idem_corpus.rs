//! Python stdlib 简化幂等：simplify→print 两轮文本必须一致；
//! 同时检查二轮可解析。环境：PY_ROOT / PY_OFFSET / PY_LIMIT。
use std::path::{Path, PathBuf};

use cure_engine::Config;
use cure_python_parser::parse;
use cure_python_print::print_module;
use cure_python_simplify::simplify_module;

fn once(src: &str) -> (String, usize, usize, String) {
    let mut out = parse(src);
    let errs = out.errors.len();
    let first_err = out
        .errors
        .first()
        .map(|e| format!("{}:{}:{}", e.line, e.col, e.message))
        .unwrap_or_default();
    let report = simplify_module(&mut out.ast, out.module, &Config::default());
    let printed = print_module(&out.ast, out.module);
    (printed, errs, report.edits, first_err)
}

fn main() {
    let root = PathBuf::from(std::env::var("PY_ROOT").unwrap_or_else(|_| {
        "/opt/homebrew/opt/python@3.15/Frameworks/Python.framework/Versions/3.15/lib/python3.15"
            .into()
    }));
    let offset: usize = std::env::var("PY_OFFSET")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let limit: usize = std::env::var("PY_LIMIT")
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
    let (mut stable, mut drift, mut undeclared, mut diag, mut total_edits) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    let mut examples: Vec<(String, String)> = Vec::new();
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        let (a, e1, edits, _) = once(&src);
        if e1 > 0 {
            diag += 1;
        }
        total_edits += edits;
        let (b, e2, _, ferr) = once(&a);
        if e2 > 0 {
            undeclared += 1;
            if examples.len() < 10 {
                examples.push((path.display().to_string(), format!("second parse: {ferr}")));
            }
            continue;
        }
        if a != b {
            drift += 1;
            let d = first_diff(&a, &b);
            if examples.len() < 10 {
                examples.push((path.display().to_string(), d));
            }
        } else {
            stable += 1;
        }
    }
    eprintln!(
        "py_idem: 语料 {total} → 本块 {}；稳定 {stable} / 漂移 {drift} / 二轮解析失败 {undeclared} / 一轮诊断 {diag} / edits {total_edits}",
        files.len()
    );
    for (p, d) in &examples {
        eprintln!("--- {p}\n  {d}");
    }
    if drift > 0 || undeclared > 0 {
        std::process::exit(1);
    }
}

fn first_diff(a: &str, b: &str) -> String {
    let al: Vec<&str> = a.lines().collect();
    let bl: Vec<&str> = b.lines().collect();
    for i in 0..al.len().max(bl.len()) {
        let x = al.get(i).copied().unwrap_or("<EOF>");
        let y = bl.get(i).copied().unwrap_or("<EOF>");
        if x != y {
            return format!("line {}:\n  once:  {x}\n  twice: {y}", i + 1);
        }
    }
    "<none>".into()
}

fn walk(d: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(d) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x == "py" || x == "pyi") {
            out.push(p);
        }
    }
}
