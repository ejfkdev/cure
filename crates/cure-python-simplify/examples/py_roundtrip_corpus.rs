//! Python stdlib 格式往返：parse → print，CPython 3.15 AST 等价批量比对。
//! 环境：PY_ROOT / PY_OFFSET / PY_LIMIT；分块运行控制内存。
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use cure_python_parser::parse;
use cure_python_print::print_module;

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

    struct T {
        id: String,
        orig: String,
        cured: String,
        had_errors: usize,
    }
    let mut tests = Vec::new();
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        let out = parse(&src);
        let cured = print_module(&out.ast, out.module);
        tests.push(T {
            id: path
                .strip_prefix(&root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string(),
            orig: src,
            cured,
            had_errors: out.errors.len(),
        });
    }
    eprintln!("py_roundtrip: 语料 {total} → 本块 {} 文件", tests.len());

    let parse_errors = tests.iter().filter(|t| t.had_errors > 0).count();
    let diag_ids: Vec<String> = tests
        .iter()
        .filter(|t| t.had_errors > 0)
        .take(10)
        .map(|t| t.id.clone())
        .collect();
    let driver = std::env::var("PY_DRIVER")
        .map(PathBuf::from)
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/py_ast_driver.py"));
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
        for t in &tests {
            let line = format!(
                "{{\"id\":\"{}\",\"orig\":\"{}\",\"cured\":\"{}\"}}\n",
                esc(&t.id),
                esc(&t.orig),
                esc(&t.cured)
            );
            if stdin.write_all(line.as_bytes()).is_err() {
                break;
            }
        }
    });
    let mut ok = 0usize;
    let mut badjob = 0usize;
    let mut lines_seen = 0usize;
    let mut unparsed: Vec<String> = Vec::new();
    let mut origerr = 0usize;
    let mut curederr: Vec<(String, String)> = Vec::new();
    let mut diffs: Vec<(String, String)> = Vec::new();
    for line in BufReader::new(child.stdout.take().unwrap())
        .lines()
        .flatten()
    {
        lines_seen += 1;
        let Some((id, status, detail)) = rec(&line) else {
            if unparsed.len() < 3 {
                unparsed.push(line.chars().take(200).collect());
            }
            continue;
        };
        match status.as_str() {
            "ok" => ok += 1,
            "origerr" => origerr += 1,
            "badjob" => badjob += 1,
            "curederr" => curederr.push((id, detail)),
            "diff" => diffs.push((id, detail)),
            _ => {}
        }
    }
    let _ = child.wait();
    writer.join().unwrap();
    eprintln!(
        "结果：AST 一致 {ok} / 原文语法错误 {origerr} / cured 不可解析 {} / AST 漂移 {} / 本项目解析诊断 {parse_errors} / badjob {badjob} / lines {lines_seen}",
        curederr.len(),
        diffs.len()
    );
    for id in &diag_ids {
        eprintln!("--- DIAG {id}");
    }
    for l in &unparsed {
        eprintln!("--- UNPARSED {l}");
    }
    for (id, d) in diffs.iter().take(15) {
        eprintln!("--- DIFF {id}\n  {d}");
    }
    for (id, d) in curederr.iter().take(15) {
        eprintln!("--- CUREDERR {id}\n  {d}");
    }
    if !diffs.is_empty() || !curederr.is_empty() {
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
        let rest = &s[i..];
        let mut out = String::new();
        let mut chars = rest.chars();
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
