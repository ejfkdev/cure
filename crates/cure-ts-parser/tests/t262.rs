//! Test262 官方 ECMAScript 一致性测试集（本机 /tmp/test262 存在时执行）：
//!
//! - **正句法干净解析率**（抽样自 49,266 个应合法文件）
//! - **零 panic 硬断言**（容错契约：任何输入不得中止进程）
//! - **零进展保险丝**：词法层非 Eof token 必须消费 ≥1 字节——曾因
//!   转义白名单不一致产出空 ident 无限累积（100GB OOM、系统崩溃），
//!   保险丝把这类 bug 降级为「跳一字节+记错」
//!
//! 语料缺失时跳过（CI 无 /tmp/test262）。

use std::io::Read;

const ROOT: &str = "/tmp/test262/test";

fn corpus_available() -> bool {
    std::path::Path::new(ROOT).is_dir()
}

/// 均匀抽样 4000 个（全量 49k 在测试里太慢）。
fn sample_files() -> Vec<String> {
    let mut all = Vec::new();
    let mut stack = vec![std::path::PathBuf::from(ROOT)];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "js") {
                all.push(p.to_string_lossy().to_string());
            }
        }
    }
    all.sort();
    let n = all.len();
    if n == 0 {
        return all;
    }
    let step = (n + 3999) / 4000;
    all.into_iter().step_by(step.max(1)).collect()
}

#[test]
fn t262_no_panics_and_parse_rate() {
    if !corpus_available() {
        eprintln!("skip t262: /tmp/test262 不存在");
        return;
    }
    let files = sample_files();
    let n = std::thread::available_parallelism().map(|x| x.get()).unwrap_or(8);
    let chunk = (files.len() + n - 1) / n;
    let mut handles = Vec::new();
    for t in 0..n {
        let part: Vec<String> = files[t * chunk..((t + 1) * chunk).min(files.len())].to_vec();
        handles.push(std::thread::spawn(move || {
            let (mut clean, mut errs, mut panics) = (0usize, 0usize, 0usize);
            for path in part {
                let mut src = String::new();
                if std::fs::File::open(&path)
                    .and_then(|mut f| f.read_to_string(&mut src))
                    .is_err()
                {
                    continue;
                }
                match std::panic::catch_unwind(|| cure_ts_parser::parse(&src).errors.len()) {
                    Ok(0) => clean += 1,
                    Ok(_) => errs += 1,
                    Err(_) => panics += 1,
                }
            }
            (clean, errs, panics)
        }));
    }
    let (mut clean, mut errs, mut panics) = (0, 0, 0);
    for h in handles {
        let (c, e, p) = h.join().unwrap();
        clean += c;
        errs += e;
        panics += p;
    }
    let total = clean + errs;
    assert_eq!(panics, 0, "{panics} 个文件 panic——容错契约被破坏");
    let rate = clean as f64 / total.max(1) as f64;
    eprintln!("t262 抽样 {total} 文件：clean {clean}（{rate:.1}%）/ errored {errs}");
    assert!(rate >= 0.85, "抽样解析率 {rate:.1}% 低于 85% 基线");
}

#[test]
fn t262_zero_progress_fuse() {
    // 保险丝单元验证：无效转义形态只跳字节，绝不无限累积
    let src = "var x = a._\\uZZZZ;";
    let (toks, errs, _comments) = cure_ts_parser::Lexer::new(src).tokenize();
    assert!(!toks.is_empty());
    assert!(toks.len() < 20, "token 数 {}——零进展累积回来了！", toks.len());
    assert!(errs.iter().any(|e| e.message.contains("zero-progress")));
}
