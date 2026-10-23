//! Test262 **async 语义差分**：仅 flags 含 async（排除 module/raw/负向），
//! 原版与 cure 输出各在全新 node vm 上下文执行并等待 $DONE/定时器。
//! 用法：t262_async_sem [样本数，0=全量]；T262_OFFSET/T262_LIMIT 分块。

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
    let sta = std::fs::read_to_string(root.join("harness/sta.js")).unwrap();
    let assert = std::fs::read_to_string(root.join("harness/assert.js")).unwrap();
    let mut files = Vec::new();
    walk(&root.join("test"), &mut files);
    files.sort();
    // 先按 flags 过滤出**候选**，再在候选上抽样（否则 10% 的 async
    // 占比会让 sample=600 只落到 ~60 个用例）
    struct Cand { path: PathBuf, fm: String, src: String }
    let mut cands: Vec<Cand> = Vec::new();
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else { continue };
        let Some(fm) = frontmatter(&src) else { continue };
        let flags = fm.split("flags: [").nth(1).and_then(|s| s.split(']').next()).unwrap_or("");
        if !flags.contains("async")
            || flags.contains("module")
            || flags.contains("raw")
            || flags.contains("CanBlockIsFalse")
            || fm.contains("negative:")
        {
            continue;
        }
        // V8 已知崩溃用例（原版同样 SIGSEGV，与 cure 无关）：await-using
        // 语法组的 throwsAsync 用例。编译层仍覆盖这些文件。
        if path.to_string_lossy().contains("/await-using/syntax/") {
            continue;
        }
        cands.push(Cand { path: path.clone(), fm, src });
    }
    let step = if sample == 0 || sample >= cands.len() {
        1
    } else {
        cands.len().div_ceil(sample)
    };
    let cands: Vec<Cand> = cands.into_iter().step_by(step).collect();

    struct T { id: String, orig: String, cured: String, clean: bool }
    let mut tests = Vec::new();
    let mut seen = 0usize;
    for c in &cands {
        let path = &c.path;
        let fm = &c.fm;
        let src = &c.src;
        if seen < offset {
            seen += 1;
            continue;
        }
        if limit != 0 && tests.len() >= limit {
            break;
        }
        seen += 1;
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
        let mut out = parse(src);
        let clean = out.errors.is_empty();
        let _ = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
        let cured = print_module(&out.ast, &out.unit);
        let mk = |body: &str| {
            let mut s = String::with_capacity(prelude.len() + body.len() + 16);
            s.push_str(&prelude);
            if strict { s.push_str("\n\"use strict\";\n"); }
            s.push_str(body);
            s
        };
        let id = path.strip_prefix(&root).unwrap_or(path).to_string_lossy().to_string();
        tests.push(T { id, orig: mk(src), cured: mk(&cured), clean });
    }
    eprintln!("t262_async: 采样 {} → 用例 {}", files.len(), tests.len());

    let driver = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/t262_async_driver.mjs");
    let mut child = Command::new("node")
        .arg(driver)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn node");
    let mut stdin = child.stdin.take().unwrap();
    for t in &tests {
        let line = format!(
            "{{\"id\":\"{}\",\"orig\":\"{}\",\"cured\":\"{}\"}}\n",
            esc(&t.id), esc(&t.orig), esc(&t.cured)
        );
        if stdin.write_all(line.as_bytes()).is_err() { break; }
    }
    let _ = stdin.flush();
    drop(stdin);

    let clean_ids: std::collections::HashSet<String> =
        tests.iter().filter(|t| t.clean).map(|t| t.id.clone()).collect();
    let mut ok = 0usize;
    let mut known = 0usize;
    let mut skipped = 0usize;
    let mut parse_err = 0usize;
    let mut mism: Vec<(String, String, String)> = Vec::new();
    for line in BufReader::new(child.stdout.take().unwrap()).lines().flatten() {
        let Some((id, so, sc, oo, oc)) = rec(&line) else { continue };
        if so != "ok" {
            skipped += 1;
            if !clean_ids.contains(&id) { parse_err += 1; }
            continue;
        }
        if sc != "ok" || oo != oc {
            if id.contains("Function/prototype/toString")
                || id.contains("computed-property-name-from-arrow-function-expression")
            {
                known += 1;
                continue;
            }
            mism.push((id, format!("{so}:{oo}"), format!("{sc}:{oc}")));
        } else {
            ok += 1;
        }
    }
    let status = child.wait().expect("wait node");
    eprintln!(
        "结果：一致 {ok} / 不一致 {} / 已知 toString 限制 {known} / 跳过（原版失败）{skipped}（含原文解析错误 {parse_err}）/ node {status}",
        mism.len()
    );
    for (id, o, c) in mism.iter().take(20) {
        eprintln!("--- MISMATCH {id}\n  orig : {}\n  cured: {}", trunc(o), trunc(c));
    }
    if !mism.is_empty() { std::process::exit(1); }
}

fn trunc(s: &str) -> String { s.chars().take(260).collect() }

fn frontmatter(src: &str) -> Option<String> {
    let start = src.find("/*---")?;
    let end = src[start..].find("---*/")? + start;
    Some(src[start + 5..end].to_string())
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

fn rec(line: &str) -> Option<(String, String, String, String, String)> {
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
    Some((f(line, "id")?, f(line, "st_orig")?, f(line, "st_cured")?, f(line, "out_orig")?, f(line, "out_cured")?))
}

fn walk(d: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(d) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() { walk(&p, out); }
        else if p.extension().is_some_and(|x| x == "js") { out.push(p); }
    }
}
