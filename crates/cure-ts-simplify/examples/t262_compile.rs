//! Test262 **全量编译差分**：每个文件（含 async/module/negative）解析 →
//! 简化 → 打印，原版与 cure 输出各自在 node 中编译（不执行）。
//! valid 原版 ⇒ cured 必须仍可编译；负向语法用例两边都应 SyntaxError。
//!
//! 用法：t262_compile [样本数，0=全量]；T262_OFFSET/T262_LIMIT 分块。

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use cure_engine::Config;
use cure_ts_parser::parse;
use cure_ts_print::print_module;
use cure_ts_simplify::simplify_unit;

fn main() {
    let root = PathBuf::from(std::env::var("T262_ROOT").unwrap_or("/tmp/test262".into()));
    let sample: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(0);
    let offset: usize = std::env::var("T262_OFFSET").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let limit: usize = std::env::var("T262_LIMIT").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let mut files = Vec::new();
    walk(&root.join("test"), &mut files);
    files.sort();
    let step = if sample == 0 || sample >= files.len() {
        1
    } else {
        files.len().div_ceil(sample)
    };
    let files: Vec<_> = files.into_iter().step_by(step).collect();

    struct T {
        id: String,
        orig: String,
        cured: String,
        module: bool,
    }
    let mut tests: Vec<T> = Vec::new();
    let mut seen = 0usize;
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else { continue };
        if seen < offset {
            seen += 1;
            continue;
        }
        if limit != 0 && tests.len() >= limit {
            break;
        }
        seen += 1;
        let module = src
            .split("flags: [")
            .nth(1)
            .and_then(|s| s.split(']').next())
            .is_some_and(|f| f.contains("module"));
        let mut out = parse(&src);
        let _ = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
        let cured = print_module(&out.ast, &out.unit);
        let id = path.strip_prefix(&root).unwrap_or(path).to_string_lossy().to_string();
        tests.push(T { id, orig: src, cured, module });
    }
    eprintln!("t262_compile: 采样 {} → 用例 {}", files.len(), tests.len());

    let driver = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/t262_compile_driver.mjs");
    let mut child = Command::new("node")
        .arg("--experimental-vm-modules")
        .arg(driver)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn node");
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
    drop(stdin);

    let mut orig_ok = 0usize;
    let mut orig_syntax = 0usize;
    let mut valid_broken: Vec<(String, String)> = Vec::new();
    let mut neg_now_valid: Vec<String> = Vec::new();
    let mut both_syntax = 0usize;
    for line in BufReader::new(child.stdout.take().unwrap()).lines().flatten() {
        let Some((id, o, c)) = rec(&line) else { continue };
        match (o.as_str(), c.as_str()) {
            ("ok", "ok") => orig_ok += 1,
            ("ok", other) => valid_broken.push((id, other.to_string())),
            ("syntax", "ok") => neg_now_valid.push(id),
            ("syntax", _) => {
                orig_syntax += 1;
                both_syntax += 1;
            }
            (_, _) => {
                // 原版非 syntax 编译失败（极少：module flag 不匹配等）
                if c == "ok" {
                    neg_now_valid.push(id);
                }
            }
        }
    }
    let status = child.wait().expect("wait node");
    eprintln!(
        "结果：valid 原版编译通过 {orig_ok} / valid 但 cured 编译失败 {} / 负向仍语法错 {both_syntax} / 负向变可编译 {} / node {status}",
        valid_broken.len(),
        neg_now_valid.len()
    );
    for (id, c) in valid_broken.iter().take(30) {
        eprintln!("--- BROKEN {id} :: cured={c}");
    }
    for id in neg_now_valid.iter().take(10) {
        eprintln!("--- NEG-OK {id}");
    }
    if !valid_broken.is_empty() || !neg_now_valid.is_empty() {
        std::process::exit(1);
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
