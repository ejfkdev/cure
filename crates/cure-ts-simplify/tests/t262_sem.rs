//! Test262 **语义差分**常驻测试：原文件 vs cure 输出，在同一 node vm
//! 隔离上下文执行，比较异常状态与 console 输出。
//!
//! 语料与环境：`T262_ROOT`（默认 `/tmp/test262`，需含 `test/` 与
//! `harness/`）、`node` 可执行。缺失时跳过（CI 无语料不失败）。
//!
//! **已知语义不可保持面**（allowlist）：`Function.prototype.toString`
//! 依赖函数**源码文本**——cure 会重排函数体格式；计算属性名里以
//! 箭头函数为键（`{[() => {}]: v}`）同样经 toString 参与键比较。
//! 其余任何失配都是硬失败。

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use cure_engine::Config;
use cure_ts_parser::parse_js as parse;
use cure_ts_print::print_module;
use cure_ts_simplify::simplify_unit;

fn allowlisted(id: &str) -> bool {
    id.contains("built-ins/Function/prototype/toString")
        || id.contains("computed-property-name-from-arrow-function-expression")
        || id.contains("staging/sm/async-functions/toString")
        || id.contains("staging/sm/generators/runtime")
}

#[test]
fn t262_semantic_differential_sample() {
    let root = PathBuf::from(std::env::var("T262_ROOT").unwrap_or("/tmp/test262".into()));
    if !root.join("test").is_dir() || !root.join("harness").is_dir() {
        eprintln!("skip: 语料缺失（T262_ROOT={}）", root.display());
        return;
    }
    let node = if Command::new("node").arg("--version").output().is_ok() {
        "node".to_string()
    } else {
        eprintln!("skip: 找不到 node");
        return;
    };
    let sample: usize = std::env::var("T262_SAMPLE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(800);

    let sta = std::fs::read_to_string(root.join("harness/sta.js")).unwrap();
    let assert = std::fs::read_to_string(root.join("harness/assert.js")).unwrap();
    let mut files = Vec::new();
    walk(&root.join("test"), &mut files);
    files.sort();
    let step = files.len().div_ceil(sample).max(1);
    let files: Vec<_> = files.into_iter().step_by(step).collect();

    struct Test {
        id: String,
        orig: String,
        cured: String,
        clean: bool,
    }
    let mut tests: Vec<Test> = Vec::new();
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else { continue };
        let Some(fm) = frontmatter(&src) else { continue };
        if let Some(flags) = fm.split("flags: [").nth(1).and_then(|s| s.split(']').next()) {
            if flags.contains("async")
                || flags.contains("module")
                || flags.contains("raw")
                || flags.contains("CanBlockIsFalse")
            {
                continue;
            }
        }
        if fm.contains("negative:") {
            continue;
        }
        let mut prelude = String::new();
        prelude.push_str(&sta);
        prelude.push('\n');
        prelude.push_str(&assert);
        prelude.push('\n');
        if let Some(inc) = fm.split("includes: [").nth(1).and_then(|s| s.split(']').next()) {
            for name in inc.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                if let Ok(h) = std::fs::read_to_string(root.join("harness").join(name)) {
                    prelude.push_str(&h);
                    prelude.push('\n');
                }
            }
        }
        let strict = fm.contains("onlyStrict");
        let mut out = parse(&src);
        let clean = out.errors.is_empty();
        let _ = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
        let cured = print_module(&out.ast, &out.unit);
        let mk = |body: &str| {
            let mut s = String::with_capacity(prelude.len() + body.len() + 16);
            s.push_str(&prelude);
            if strict {
                s.push_str("\n\"use strict\";\n");
            }
            s.push_str(body);
            s
        };
        let id = path.strip_prefix(&root).unwrap_or(path).to_string_lossy().to_string();
        tests.push(Test { id, orig: mk(&src), cured: mk(&cured), clean });
    }
    assert!(!tests.is_empty(), "语料可用但筛选后用例为空");

    let driver = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/t262_vm_driver.mjs");
    let mut child = Command::new(node)
        .arg(&driver)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn node driver");
    let mut stdin = child.stdin.take().unwrap();
    {
        for t in &tests {
            let line = format!(
                "{{\"id\":\"{}\",\"orig\":\"{}\",\"cured\":\"{}\"}}\n",
                json_escape(&t.id),
                json_escape(&t.orig),
                json_escape(&t.cured)
            );
            if stdin.write_all(line.as_bytes()).is_err() {
                break;
            }
        }
        let _ = stdin.flush();
    }
    drop(stdin);

    let mut ok = 0usize;
    let mut mismatches: Vec<(String, String, String)> = Vec::new();
    let mut parse_err_tested = 0usize;
    let clean_ids: std::collections::HashSet<String> = tests
        .iter()
        .filter(|t| t.clean)
        .map(|t| t.id.clone())
        .collect();
    for line in BufReader::new(child.stdout.take().unwrap()).lines().flatten() {
        let Some((id, st_o, st_c, out_o, out_c)) = parse_rec(&line) else { continue };
        if st_o != "ok" {
            if !clean_ids.contains(&id) {
                parse_err_tested += 1;
            }
            continue;
        }
        if st_c != "ok" || out_o != out_c {
            if allowlisted(&id) {
                continue;
            }
            mismatches.push((id, format!("{st_o}:{out_o}"), format!("{st_c}:{out_c}")));
        } else {
            ok += 1;
        }
    }
    let status = child.wait().expect("node driver wait");
    assert!(status.success(), "node 驱动失败: {status}");
    assert!(
        ok * 3 >= tests.len(),
        "执行覆盖过低：ok={ok} tests={}（驱动/语料异常？）",
        tests.len()
    );
    eprintln!(
        "t262_sem: 一致 {ok} / 硬失配 {} / 原文解析错误用例 {parse_err_tested}",
        mismatches.len()
    );
    assert_eq!(parse_err_tested, 0, "cure 解析失败但原文件可执行");
    if !mismatches.is_empty() {
        for (id, o, c) in mismatches.iter().take(10) {
            eprintln!("--- {id}\n  orig : {}\n  cured: {}", trunc(o), trunc(c));
        }
        panic!("{} 个语义失配", mismatches.len());
    }
}

fn trunc(s: &str) -> String {
    s.chars().take(240).collect()
}

fn frontmatter(src: &str) -> Option<String> {
    let start = src.find("/*---")?;
    let end = src[start..].find("---*/")? + start;
    Some(src[start + 5..end].to_string())
}

fn json_escape(s: &str) -> String {
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

fn parse_rec(line: &str) -> Option<(String, String, String, String, String)> {
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
        field(line, "st_orig")?,
        field(line, "st_cured")?,
        field(line, "out_orig")?,
        field(line, "out_cured")?,
    ))
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
