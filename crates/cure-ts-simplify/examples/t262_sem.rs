//! Test262 语义差分（大规模）：正句法用例 + harness 前导，原版 vs
//! 简化版 分别在独立 vm 上下文执行，stdout/异常类比对。
//!
//! 用法：t262_sem [样本数]（默认 2000；0 = 全量）
//! 环境：T262_ROOT 覆盖语料根（默认 /tmp/test262）

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use cure_engine::Config;
use cure_ts_parser::parse;
use cure_ts_print::print_module;
use cure_ts_simplify::simplify_unit;

struct Test {
    id: String,
    orig: String,
    cured: String,
    clean_parse: bool,
}

fn main() {
    let root = PathBuf::from(std::env::var("T262_ROOT").unwrap_or("/tmp/test262".into()));
    let harness = root.join("harness");
    let sample: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(2000);
    let sta = std::fs::read_to_string(harness.join("sta.js")).unwrap();
    let assert = std::fs::read_to_string(harness.join("assert.js")).unwrap();

    let mut files: Vec<PathBuf> = Vec::new();
    walk(&root.join("test"), &mut files);
    files.sort();
    let total_found = files.len();
    let step = if sample == 0 || sample >= files.len() {
        1
    } else {
        files.len().div_ceil(sample)
    };
    let files: Vec<PathBuf> = files.into_iter().step_by(step).collect();

    // 分块全量扫描（内存安全）：T262_OFFSET/T262_LIMIT 在**过滤后**计数，
    // 每次进程只构建有限用例（49k 全量字符串会占 ~1GB）。
    let offset: usize = std::env::var("T262_OFFSET").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let limit: usize = std::env::var("T262_LIMIT").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let mut tests: Vec<Test> = Vec::new();
    let mut skipped_flags = 0usize;
    let mut eligible_seen = 0usize;
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else { continue };
        let Some(fm) = frontmatter(&src) else { continue };
        if let Some(flags) = fm.split("flags: [").nth(1).and_then(|s| s.split(']').next()) {
            if flags.contains("async")
                || flags.contains("module")
                || flags.contains("raw")
                || flags.contains("CanBlockIsFalse")
            {
                skipped_flags += 1;
                continue;
            }
        }
        if fm.contains("negative:") {
            skipped_flags += 1;
            continue;
        }
        let mut prelude = String::new();
        prelude.push_str(&sta);
        prelude.push('\n');
        prelude.push_str(&assert);
        prelude.push('\n');
        if let Some(inc) = fm.split("includes: [").nth(1).and_then(|s| s.split(']').next()) {
            for name in inc.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                let p = harness.join(name);
                if let Ok(h) = std::fs::read_to_string(&p) {
                    prelude.push_str(&h);
                    prelude.push('\n');
                }
            }
        }
        let strict = fm.contains("onlyStrict");
        if eligible_seen < offset {
            eligible_seen += 1;
            continue;
        }
        if limit != 0 && tests.len() >= limit {
            break;
        }
        eligible_seen += 1;

        let mut out = parse(&src);
        let clean_parse = out.errors.is_empty();
        let _ = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
        let cured_src = print_module(&out.ast, &out.unit);

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
        tests.push(Test { id, orig: mk(&src), cured: mk(&cured_src), clean_parse });
    }
    eprintln!(
        "t262_sem: 语料 {total_found} 文件 → 采样 {} → 用例 {}（跳过 flags/负向 {skipped_flags}）",
        files.len(),
        tests.len()
    );

    let driver = std::path::Path::new("scripts/t262_vm_driver.mjs");
    let mut child = Command::new(node_bin())
        .arg(driver)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn node");
    let clean_set: std::collections::HashSet<String> = tests
        .iter()
        .filter(|t| t.clean_parse)
        .map(|t| t.id.clone())
        .collect();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || {
        for (i, t) in tests.iter().enumerate() {
            let id = json_escape(&t.id);
            let o = json_escape(&t.orig);
            let c = json_escape(&t.cured);
            let line = format!("{{\"id\":\"{id}\",\"orig\":\"{o}\",\"cured\":\"{c}\"}}\n");
            if stdin.write_all(line.as_bytes()).is_err() {
                break;
            }
            if i % 200 == 0 {
                let _ = stdin.flush();
            }
        }
        let _ = stdin.flush();
        drop(stdin);
    });

    let mut ok = 0usize;
    let mut known = 0usize;
    let mut mismatches: Vec<(String, String, String)> = Vec::new();
    let mut skipped_invalid = 0usize;
    let mut parse_err_tested = 0usize;
    let reader = BufReader::new(child.stdout.take().unwrap());
    for line in reader.lines().flatten() {
        let Some((id, st_o, st_c, out_o, out_c)) = parse_rec(&line) else { continue };
        if st_o != "ok" {
            skipped_invalid += 1;
            if !clean_set.contains(&id) {
                parse_err_tested += 1;
            }
            continue;
        }
        if st_c != "ok" || out_o != out_c {
            // 已知不可保持面：Function.prototype.toString 源码文本
            // （cure 重排函数体）与箭头函数计算属性键（经 toString）。
            if id.contains("built-ins/Function/prototype/toString")
                || id.contains("computed-property-name-from-arrow-function-expression")
            {
                known += 1;
                continue;
            }
            mismatches.push((id, format!("{st_o}:{out_o}"), format!("{st_c}:{out_c}")));
        } else {
            ok += 1;
        }
    }
    let status = child.wait().unwrap();
    writer.join().unwrap();

    eprintln!(
        "结果：一致 {ok} / 不一致 {} / 已知 toString 限制 {known} / 跳过（原版失败）{skipped_invalid}（其中含原文解析错误 {parse_err_tested}）",
        mismatches.len()
    );
    eprintln!("node 驱动退出码: {status}");
    for (id, o, c) in mismatches.iter() {
        eprintln!("--- MISMATCH {id}");
        eprintln!("  orig : {}", o.chars().take(300).collect::<String>());
        eprintln!("  cured: {}", c.chars().take(300).collect::<String>());
    }
    if !mismatches.is_empty() {
        std::process::exit(1);
    }
}

fn node_bin() -> String {
    if Command::new("node").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
        "node".into()
    } else {
        "/Users/e/.local/share/vite-plus/bin/node".into()
    }
}

fn walk(d: &std::path::Path, out: &mut Vec<PathBuf>) {
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
