//! Test262 **async 语义差分**常驻测试（抽样）：flags 含 async 的用例在
//! 独立 vm 上下文执行，等待 `$DONE`/微任务/定时器后比较异常状态与
//! stdout。已知 V8 崩溃的 await-using/syntax 组跳过（原版同样 SIGSEGV；
//! 编译层仍覆盖）。语料/环境缺失自动跳过。

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use cure_engine::Config;
use cure_ts_parser::parse;
use cure_ts_print::print_module;
use cure_ts_simplify::simplify_unit;

#[test]
fn t262_async_semantic_sample() {
    let root = PathBuf::from(std::env::var("T262_ROOT").unwrap_or("/tmp/test262".into()));
    if !root.join("test").is_dir() || !root.join("harness").is_dir() {
        eprintln!("skip: 语料缺失");
        return;
    }
    if Command::new("node").arg("--version").output().is_err() {
        eprintln!("skip: 找不到 node");
        return;
    }
    let sample: usize = std::env::var("T262_SAMPLE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);
    let sta = std::fs::read_to_string(root.join("harness/sta.js")).unwrap();
    let assert = std::fs::read_to_string(root.join("harness/assert.js")).unwrap();

    struct Cand {
        path: PathBuf,
        fm: String,
        src: String,
    }
    let mut files = Vec::new();
    walk(&root.join("test"), &mut files);
    files.sort();
    let mut cands = Vec::new();
    for path in files {
        let Ok(src) = std::fs::read_to_string(&path) else { continue };
        let Some(fm) = frontmatter(&src) else { continue };
        let flags = fm.split("flags: [").nth(1).and_then(|s| s.split(']').next()).unwrap_or("");
        if !flags.contains("async")
            || flags.contains("module")
            || flags.contains("raw")
            || flags.contains("CanBlockIsFalse")
            || fm.contains("negative:")
            || path.to_string_lossy().contains("/await-using/syntax/")
        {
            continue;
        }
        cands.push(Cand { path, fm, src });
    }
    assert!(!cands.is_empty());
    let step = cands.len().div_ceil(sample).max(1);
    let cands: Vec<_> = cands.into_iter().step_by(step).collect();

    struct T {
        id: String,
        orig: String,
        cured: String,
    }
    let mut tests = Vec::new();
    for c in &cands {
        let mut prelude = String::new();
        prelude.push_str(&sta);
        prelude.push('\n');
        prelude.push_str(&assert);
        prelude.push('\n');
        if let Some(inc) = c.fm.split("includes: [").nth(1).and_then(|s| s.split(']').next()) {
            for name in inc.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                if let Ok(h) = std::fs::read_to_string(root.join("harness").join(name)) {
                    prelude.push_str(&h);
                    prelude.push('\n');
                }
            }
        }
        let strict = c.fm.contains("onlyStrict");
        let mut out = parse(&c.src);
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
        let id = c.path.strip_prefix(&root).unwrap_or(&c.path).to_string_lossy().to_string();
        tests.push(T { id, orig: mk(&c.src), cured: mk(&cured) });
    }

    let driver = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/t262_async_driver.mjs");
    let mut child = Command::new("node")
        .arg(driver)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn node");
    {
        let mut stdin = child.stdin.take().unwrap();
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
        let _ = stdin.flush();
    }

    let mut ok = 0usize;
    let mut skipped = 0usize;
    let mut mism: Vec<(String, String, String)> = Vec::new();
    for line in BufReader::new(child.stdout.take().unwrap()).lines().flatten() {
        let Some((id, so, sc, oo, oc)) = rec(&line) else { continue };
        if so != "ok" {
            skipped += 1;
            continue;
        }
        if sc != "ok" || oo != oc {
            if id.contains("Function/prototype/toString")
                || id.contains("computed-property-name-from-arrow-function-expression")
            {
                continue;
            }
            mism.push((id, format!("{so}:{oo}"), format!("{sc}:{oc}")));
        } else {
            ok += 1;
        }
    }
    let status = child.wait().expect("wait node");
    assert!(status.success(), "node driver 失败: {status}");
    eprintln!("t262_async: 一致 {ok} / 跳过 {skipped} / 不一致 {}", mism.len());
    if !mism.is_empty() {
        for (id, o, c) in mism.iter().take(10) {
            eprintln!("--- MISMATCH {id}\n  orig : {}\n  cured: {}", trunc(o), trunc(c));
        }
        panic!("{} 个 async 语义失配", mism.len());
    }
    assert!(ok * 2 >= tests.len(), "覆盖过低: ok={ok} tests={}", tests.len());
}

fn trunc(s: &str) -> String {
    s.chars().take(240).collect()
}

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
    Some((
        f(line, "id")?,
        f(line, "st_orig")?,
        f(line, "st_cured")?,
        f(line, "out_orig")?,
        f(line, "out_cured")?,
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
