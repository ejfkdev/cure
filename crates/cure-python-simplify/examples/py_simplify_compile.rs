//! Python stdlib 简化产物 CPython 3.15 编译检查（分块）。
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use cure_engine::Config;
use cure_python_parser::parse;
use cure_python_print::print_module;
use cure_python_simplify::simplify_module;

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
    let mut jobs: Vec<(String, String, String)> = Vec::new();
    let mut parse_diag = 0usize;
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        let mut out = parse(&src);
        if !out.errors.is_empty() {
            parse_diag += 1;
        }
        let _ = simplify_module(&mut out.ast, out.module, &Config::default());
        let cured = print_module(&out.ast, out.module);
        jobs.push((
            path.strip_prefix(&root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string(),
            src.clone(),
            cured,
        ));
    }
    eprintln!(
        "py_compile: 语料 {total} → 本块 {} 文件（解析诊断 {parse_diag}）",
        jobs.len()
    );
    let driver = std::env::var("PY_DRIVER")
        .map(PathBuf::from)
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/py_compile_driver.py"));
    let py_bin = std::env::var("PY_BIN").unwrap_or_else(|_| "python3".into());
    let mut child = Command::new(&py_bin)
        .arg(driver)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn python driver");
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || {
        for (id, orig, cured) in &jobs {
            let line = format!(
                "{{\"id\":\"{}\",\"orig\":\"{}\",\"cured\":\"{}\"}}\n",
                esc(id),
                esc(orig),
                esc(cured)
            );
            if stdin.write_all(line.as_bytes()).is_err() {
                break;
            }
        }
    });
    let mut ok = 0usize;
    let mut origerr = 0usize;
    let mut syntax: Vec<(String, String)> = Vec::new();
    for line in BufReader::new(child.stdout.take().unwrap())
        .lines()
        .flatten()
    {
        let Some((id, status, detail)) = rec(&line) else {
            continue;
        };
        match status.as_str() {
            "ok" => ok += 1,
            "origerr" => origerr += 1,
            _ => syntax.push((id, detail)),
        }
    }
    let _ = child.wait();
    writer.join().unwrap();
    eprintln!(
        "结果：CPython 编译通过 {ok} / 原版本就不可编译 {origerr} / 失败 {}",
        syntax.len()
    );
    for (id, d) in syntax.iter().take(20) {
        eprintln!("--- FAIL {id}: {d}");
    }
    if !syntax.is_empty() {
        std::process::exit(1);
    }
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

fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 16);
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
    fn field(s: &str, key: &str) -> Option<String> {
        let pat = format!("\"{key}\":\"");
        let i = s.find(&pat)? + pat.len();
        let mut out = String::new();
        let mut chars = s[i..].chars();
        while let Some(c) = chars.next() {
            match c {
                '"' => return Some(out),
                '\\' => match chars.next()? {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    'u' => {
                        let h: String = chars.by_ref().take(4).collect();
                        if let Ok(v) = u32::from_str_radix(&h, 16) {
                            if let Some(ch) = char::from_u32(v) {
                                out.push(ch);
                            }
                        }
                    }
                    o => out.push(o),
                },
                c => out.push(c),
            }
        }
        None
    }
    Some((
        field(line, "id")?,
        field(line, "status")?,
        field(line, "detail").unwrap_or_default(),
    ))
}
