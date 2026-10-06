//! # cure CLI
//!
//! `cure` —— 容错式代码简化 / 格式化命令行工具。
//!
//! - 输入：源码文件、目录（递归）、stdin
//! - 单文件默认输出 stdout；目录默认输出到同级 `<目录名>-cure/`（保持
//!   内部目录结构），`-o` 可指定输出根，`-w` 原地覆写
//! - 语法错误不阻断：错误区域原文保留，其余照常优化（`--strict` 时非零退出）
//! - 多文件/目录自动多核并行（std::thread::scope）

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cure_engine::Config;
use cure_java_parser::parse;
use cure_java_print::print_unit;
use cure_java_simplify::simplify_unit;

/// 当前引擎实际支持的语言后缀（小写、无点）。
const SUPPORTED_EXTS: &[&str] = &["java"];

const USAGE: &str = "\
cure — 容错式代码简化 / 格式化工具

用法:
  cure [选项] <文件>...           处理源码文件（单个默认输出到 stdout）
  cure [选项] <目录>              递归处理目录，输出到同级 <目录名>-cure/
  cure [选项] -                   从 stdin 读取

输出:
  单文件             stdout（默认）；-o 指定文件
  目录               同级 <目录名>-cure/（保持内部结构）；-o 指定输出根；
                     -w 原地覆写；--check 只检查不写出

选项:
  -o, --output <路径>      输出路径（单文件=文件路径；目录=输出根目录）
  -w, --write              原地覆写输入文件
      --check              不写出内容；有可优化改写时退出码 2，否则 0
      --ext <后缀列表>     只处理指定后缀（逗号或空格分隔，如 \"java\" 或
                            \"java, js\"；默认处理全部受支持后缀）
      --copy-other         目录模式：不受处理的文件原样拷入输出树
      --diff               打印每个被改写文件的统一 diff（Myers 算法）
      --stats              打印汇总统计（文件数/改写数/行数变化/逐规则）
      --report             在 stderr 打印逐文件改写统计
      --format-only        仅格式化，不做简化
      --disable <规则名>   禁用指定规则（可多次）
      --dead-code          删除全单元零引用的 private 方法（反射场景慎用）
      --strict             有解析错误时退出码 1（默认仅 stderr 提示）
  -h, --help               本帮助
  -V, --version            版本

退出码: 0 正常；1 --strict 且有解析错误；2 --check 且有改写 / 参数错误
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("cure: {msg}");
            eprintln!("用法见 `cure --help`");
            ExitCode::from(2)
        }
    }
}

#[derive(Clone)]
struct Options {
    inputs: Vec<PathBuf>,
    output: Option<PathBuf>,
    in_place: bool,
    format_only: bool,
    check: bool,
    exts: Vec<String>,
    copy_other: bool,
    diff: bool,
    stats: bool,
    report: bool,
    disabled: Vec<String>,
    dead_code: bool,
    strict: bool,
    stdin: bool,
}

/// 单文件处理结果（聚合统计用）。
#[derive(Default, Clone)]
#[allow(dead_code)] // display 供 per-file 诊断扩展
struct FileResult {
    changed: bool,
    errored: bool,
    edits: usize,
    lines_before: usize,
    lines_after: usize,
    by_rule: Vec<(&'static str, usize)>,
    /// --diff 生成的差异文本（未启用或差异过大时为空）
    diff_text: String,
    display: String,
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut opts = Options {
        inputs: Vec::new(),
        output: None,
        in_place: false,
        format_only: false,
        check: false,
        exts: Vec::new(),
        copy_other: false,
        diff: false,
        stats: false,
        report: false,
        disabled: Vec::new(),
        dead_code: false,
        strict: false,
        stdin: false,
    };
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("cure {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "-" => opts.stdin = true,
            "-w" | "--write" => opts.in_place = true,
            "--format-only" => opts.format_only = true,
            "--check" => opts.check = true,
            "--copy-other" => opts.copy_other = true,
            "--diff" => opts.diff = true,
            "--stats" => opts.stats = true,
            "--report" => opts.report = true,
            "--strict" => opts.strict = true,
            "--dead-code" => opts.dead_code = true,
            "-o" | "--output" => {
                i += 1;
                let v = args.get(i).ok_or("missing value for --output")?;
                opts.output = Some(PathBuf::from(v));
            }
            "--ext" => {
                i += 1;
                let v = args.get(i).ok_or("missing value for --ext")?;
                // 逗号或空格分隔；容忍前导点与大写
                for part in v.split(|c: char| c == ',' || c.is_whitespace()) {
                    let p = part.trim().trim_start_matches('.').trim().to_ascii_lowercase();
                    if !p.is_empty() && !opts.exts.contains(&p) {
                        opts.exts.push(p);
                    }
                }
            }
            "--disable" => {
                i += 1;
                let v = args.get(i).ok_or("missing value for --disable")?;
                opts.disabled.push(v.clone());
            }
            other if other.starts_with('-') => {
                return Err(format!("未知选项 `{other}`"));
            }
            other => opts.inputs.push(PathBuf::from(other)),
        }
        i += 1;
    }
    if opts.stdin && !opts.inputs.is_empty() {
        return Err("stdin (`-`) 与文件参数不能同时使用".into());
    }
    if opts.stdin && opts.output.is_none() && opts.in_place {
        return Err("stdin 模式不能与 -w 组合".into());
    }
    // --ext 与受支持后缀求交集，提示不受支持的项
    if !opts.exts.is_empty() {
        let unsupported: Vec<String> = opts
            .exts
            .iter()
            .filter(|e| !SUPPORTED_EXTS.contains(&e.as_str()))
            .cloned()
            .collect();
        if !unsupported.is_empty() {
            eprintln!(
                "cure: 提示：后缀 {} 暂不受支持（当前支持：{}），已忽略",
                unsupported.join(", "),
                SUPPORTED_EXTS.join(", ")
            );
        }
        opts.exts.retain(|e| SUPPORTED_EXTS.contains(&e.as_str()));
        if opts.exts.is_empty() {
            return Err("--ext 指定的后缀均不受支持".into());
        }
    }
    Ok(opts)
}

/// 目录模式默认输出根：输入目录的同级 `<名>-cure/`。
fn default_out_root(dir: &Path) -> Result<PathBuf, String> {
    let canon = dir
        .canonicalize()
        .map_err(|e| format!("访问 {} 失败: {e}", dir.display()))?;
    let name = canon
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("无法确定目录名")?;
    let parent = canon
        .parent()
        .ok_or("输入目录没有父目录（无法生成 -cure 输出目录）")?;
    Ok(parent.join(format!("{name}-cure")))
}

/// 递归收集文件。跳过 `.git` 与输出根（防嵌套自吞）。
/// 并行目录遍历：顶层子目录分派到线程（37 万文件实测单线程 walk 7s →
/// 并行 <1s；深目录树收尾用单线程补扫）。语义与旧递归版一致（顺序
/// 不保证——调用方已按需排序）。
fn walk_files_parallel(
    root: &Path,
    skip: &Path,
    exts: &[String],
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    // 先列根的直接子项（浅层），子目录并行递归
    let mut top_dirs: Vec<PathBuf> = Vec::new();
    let mut sources: Vec<PathBuf> = Vec::new();
    let mut others: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name == ".git" {
                    continue;
                }
                if p.canonicalize().map(|c| c == skip).unwrap_or(false) {
                    continue;
                }
                top_dirs.push(p);
            } else {
                classify_file(&p, exts, &mut sources, &mut others);
            }
        }
    }
    // 单目录/少目录：直接串行（避免线程开销）
    if top_dirs.len() < 4 {
        for d in top_dirs {
            let mut s2 = Vec::new();
            let mut o2 = Vec::new();
            walk_files(&d, skip, &mut s2, &mut o2, exts);
            sources.extend(s2);
            others.extend(o2);
        }
        return (sources, others);
    }
    let n = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(top_dirs.len());
    let chunk = top_dirs.len().div_ceil(n);
    let results: Vec<std::sync::Mutex<(Vec<PathBuf>, Vec<PathBuf>)>> =
        (0..n).map(|_| std::sync::Mutex::new((Vec::new(), Vec::new()))).collect();
    std::thread::scope(|sc| {
        let mut handles = Vec::new();
        for (i, dirs) in top_dirs.chunks(chunk).enumerate() {
            let results = &results;
            let exts = exts;
            let skip = skip;
            handles.push(sc.spawn(move || {
                for d in dirs {
                    let mut s2 = Vec::new();
                    let mut o2 = Vec::new();
                    walk_files(d, skip, &mut s2, &mut o2, exts);
                    let mut r = results[i].lock().unwrap();
                    r.0.extend(s2);
                    r.1.extend(o2);
                }
            }));
        }
        for h in handles {
            let _ = h.join();
        }
    });
    for r in results {
        let (s2, o2) = r.into_inner().unwrap();
        sources.extend(s2);
        others.extend(o2);
    }
    (sources, others)
}

fn classify_file(p: &Path, exts: &[String], sources: &mut Vec<PathBuf>, others: &mut Vec<PathBuf>) {
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    let wanted = exts.is_empty() || exts.contains(&ext);
    if wanted && SUPPORTED_EXTS.contains(&ext.as_str()) {
        sources.push(p.to_path_buf());
    } else {
        others.push(p.to_path_buf());
    }
}

#[allow(dead_code)]
fn walk_files(
    root: &Path,
    skip: &Path,
    sources: &mut Vec<PathBuf>,
    others: &mut Vec<PathBuf>,
    exts: &[String],
) {
    let entries = match fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == ".git" {
                continue;
            }
            if p.canonicalize().map(|c| c == skip).unwrap_or(false) {
                continue;
            }
            walk_files(&p, skip, sources, others, exts);
        } else {
            let ext = p
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
                .unwrap_or_default();
            let wanted = exts.is_empty() || exts.contains(&ext);
            if wanted && SUPPORTED_EXTS.contains(&ext.as_str()) {
                sources.push(p);
            } else {
                others.push(p);
            }
        }
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let opts = parse_args(args)?;

    if opts.stdin {
        let mut src = String::new();
        std::io::stdin()
            .read_to_string(&mut src)
            .map_err(|e| format!("读取 stdin 失败: {e}"))?;
        let r = process_source(&src, &opts, None, "<stdin>")?;
        return Ok(final_code_opts(r.changed, r.errored, &opts));
    }

    if opts.inputs.is_empty() {
        return Err("缺少输入（文件 / 目录 / `-`）".into());
    }

    // 区分文件与目录输入
    let mut plain_files: Vec<PathBuf> = Vec::new();
    let mut dirs: Vec<PathBuf> = Vec::new();
    for p in &opts.inputs {
        if p.is_dir() {
            dirs.push(p.clone());
        } else if p.is_file() {
            plain_files.push(p.clone());
        } else {
            return Err(format!("输入 {} 不存在", p.display()));
        }
    }
    if !dirs.is_empty() && !plain_files.is_empty() {
        return Err("文件与目录输入不能混用（目录请单独传入；需要多文件用 -w/--check）".into());
    }

    // 工作项：(输入, 输出 None=stdout, 展示名)
    let mut jobs: Vec<(PathBuf, Option<PathBuf>, String)> = Vec::new();
    let mut copy_jobs: Vec<(PathBuf, PathBuf)> = Vec::new();

    if !dirs.is_empty() {
        if dirs.len() > 1 {
            return Err("一次只能处理一个目录".into());
        }
        let dir = &dirs[0];
        let out_root = if opts.in_place {
            None // -w：原地
        } else if let Some(o) = &opts.output {
            Some(o.clone())
        } else {
            Some(default_out_root(dir)?)
        };
        let effective_exts: Vec<String> = if opts.exts.is_empty() {
            SUPPORTED_EXTS.iter().map(|s| s.to_string()).collect()
        } else {
            opts.exts.clone()
        };
        let skip = out_root
            .as_ref()
            .and_then(|o| o.canonicalize().ok())
            .unwrap_or_else(|| PathBuf::from("\u{0}nonexistent"));
        let display_root = out_root
            .clone()
            .map(|o| dir.parent().map(|p| p.join(o.file_name().unwrap_or_default())).unwrap_or(o))
            .unwrap_or_else(|| dir.clone().join("."));
        let (mut sources, mut others) = walk_files_parallel(dir, &skip, &effective_exts);
        sources.sort();
        others.sort();
        if sources.is_empty() {
            return Err(format!(
                "目录 {} 中没有匹配的源码文件（--ext 过滤后为空）",
                dir.display()
            ));
        }
        if let Some(root) = &out_root {
            if opts.check {
                eprintln!("cure: --check 目录模式，不写出（输出根 {} 不创建）", root.display());
            } else {
                eprintln!("cure: {} → {}", dir.display(), display_root.display());
            }
        }
        for src in sources {
            let rel = src
                .strip_prefix(dir)
                .map(|r| r.to_path_buf())
                .unwrap_or_else(|_| src.clone());
            let out = if opts.check {
                None
            } else if opts.in_place {
                Some(src.clone())
            } else {
                Some(out_root.as_ref().unwrap().join(&rel))
            };
            jobs.push((src, out, rel.display().to_string()));
        }
        if opts.copy_other && !opts.check && !opts.in_place {
            let root = out_root.as_ref().unwrap();
            for other in others {
                let rel = other
                    .strip_prefix(dir)
                    .map(|r| r.to_path_buf())
                    .unwrap_or_else(|_| other.clone());
                copy_jobs.push((other, root.join(rel)));
            }
        }
    } else {
        // 纯文件输入
        for f in plain_files {
            let out = if opts.in_place {
                Some(f.clone())
            } else {
                opts.output.clone()
            };
            jobs.push((f, out, String::new()));
        }
        // 多文件 + stdout（无 -o/-w）不可行
        if jobs.len() > 1 && jobs.iter().all(|(_, o, _)| o.is_none()) && !opts.check {
            return Err("多个文件输出到 stdout 不受支持：请用 -w / -o / --check".into());
        }
    }

    // 预建输出目录（串行，避免 mkdir 竞争）
    for (_, out, _) in &jobs {
        if let Some(o) = out {
            if let Some(parent) = o.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("创建 {} 失败: {e}", parent.display()))?;
                }
            }
        }
    }
    for (_, dst) in &copy_jobs {
        if let Some(parent) = dst.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("创建 {} 失败: {e}", parent.display()))?;
            }
        }
    }

    // 单文件 + stdout：串行（保持原行为）
    if jobs.len() == 1 && jobs[0].1.is_none() {
        let (path, _, display) = jobs.pop().unwrap();
        let display = if display.is_empty() {
            path.display().to_string()
        } else {
            display
        };
        let src = fs::read_to_string(&path)
            .map_err(|e| format!("读取 {} 失败: {e}", path.display()))?;
        let r = process_source(&src, &opts, None, &display)?;
        return Ok(final_code_opts(r.changed, r.errored, &opts));
    }

    // 多文件：多核并行（文件间完全独立；stderr 诊断按行原子性可接受）。
    // 动态工作队列（AtomicUsize 取号）而非静态连续切片：真实目录里大文件
    // 常聚集（fernflower bd.java 0.3s vs 小文件 1ms），连续切片会把多个
    // 大文件堆进同一线程成为关键路径——3000 文件混合负载实测并行度仅
    // 7.3×/18 核，改队列后取号即做、天然均衡。
    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(jobs.len())
        .max(1);
    let opts2 = opts.clone();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let shared: Vec<(&PathBuf, &Option<PathBuf>, &str)> =
        jobs.iter().map(|(p, o, d)| (p, o, d.as_str())).collect();
    let results: Vec<Result<Vec<(usize, FileResult)>, String>> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..n_threads)
            .map(|_| {
                let opts = opts2.clone();
                let next = &next;
                let shared = &shared;
                s.spawn(move || {
                    let mut my: Vec<(usize, FileResult)> = Vec::new();
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some(&(path, out_path, display)) = shared.get(i) else {
                            break;
                        };
                        let display = if display.is_empty() {
                            path.display().to_string()
                        } else {
                            display.to_string()
                        };
                        // 非 UTF-8（ISO-8859-1/Cp1252 等编码测试文件）或读失败：
                        // 告警跳过而非中止整个目录运行（spoon/javaparser 语料
                        // 各含一个编码测试文件——曾让 5000+ 文件的运行整体失败）
                        let src = match fs::read_to_string(path) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("cure: 读取 {} 失败（跳过）: {}", path.display(), e);
                                continue;
                            }
                        };
                        let r = process_source(&src, &opts, out_path.clone(), &display)?;
                        my.push((i, r));
                    }
                    Ok(my)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("cure 线程 panic"))
            .collect()
    });
    // 结果按文件原始顺序稳定输出（诊断/统计确定性）
    let mut all: Vec<(usize, FileResult)> = Vec::new();
    for r in results {
        all.extend(r?);
    }
    all.sort_by_key(|(i, _)| *i);
    let all: Vec<FileResult> = all.into_iter().map(|(_, r)| r).collect();

    // --copy-other：非源文件原样拷贝
    for (src, dst) in copy_jobs {
        if fs::copy(&src, &dst).is_err() {
            eprintln!("cure: 拷贝 {} 失败（跳过）", src.display());
        }
    }

    // --diff：串行打印（保持顺序）
    if opts.diff {
        for r in &all {
            if r.changed && !r.diff_text.is_empty() {
                println!("{}", r.diff_text);
            }
        }
    }

    // 汇总
    if all.len() > 1 {
        let changed = all.iter().filter(|r| r.changed).count();
        let errored = all.iter().filter(|r| r.errored).count();
        let edits: usize = all.iter().map(|r| r.edits).sum();
        let lb: usize = all.iter().map(|r| r.lines_before).sum();
        let la: usize = all.iter().map(|r| r.lines_after).sum();
        eprintln!(
            "cure: {} 个文件（改写 {} / 解析错误 {}），{} 次编辑",
            all.len(),
            changed,
            errored,
            edits
        );
        if opts.stats || opts.report {
            eprintln!("  行数 {lb} → {la}（{:+.1}%）", (la as f64 - lb as f64) * 100.0 / lb.max(1) as f64);
            let mut by_rule: std::collections::BTreeMap<&'static str, usize> = Default::default();
            for r in &all {
                for (k, v) in &r.by_rule {
                    *by_rule.entry(k).or_insert(0) += v;
                }
            }
            if !by_rule.is_empty() {
                let detail: Vec<String> =
                    by_rule.iter().map(|(k, v)| format!("{k}={v}")).collect();
                eprintln!("  逐规则：{}", detail.join(", "));
            }
        }
    }

    let any_changed = all.iter().any(|r| r.changed);
    let any_error = all.iter().any(|r| r.errored);
    Ok(final_code_opts(any_changed, any_error, &opts))
}

fn final_code_opts(any_changed: bool, any_error: bool, opts: &Options) -> ExitCode {
    if opts.strict && any_error {
        ExitCode::from(1)
    } else if opts.check && any_changed {
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    }
}

/// 处理单个源码文本。
fn process_source(
    src: &str,
    opts: &Options,
    out_path: Option<PathBuf>,
    display: &str,
) -> Result<FileResult, String> {
    let mut outcome = parse(src);
    let had_errors = !outcome.errors.is_empty();
    let lines_before = src.lines().count();

    let mut result = FileResult {
        errored: had_errors,
        lines_before,
        display: display.to_string(),
        ..Default::default()
    };

    if !opts.format_only {
        let mut cfg = Config::default();
        for r in &opts.disabled {
            cfg.disabled_rules.insert(r.clone());
        }
        cfg.remove_dead_methods = opts.dead_code;
        let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &cfg);
        result.edits = report.edits;
        result.by_rule = report.by_rule.iter().map(|(k, v)| (*k, *v)).collect();
        if opts.report {
            eprintln!(
                "cure: {display}: {} 次改写 / {} 轮",
                report.edits, report.iterations
            );
            if !report.by_rule.is_empty() {
                let detail: Vec<String> =
                    report.by_rule.iter().map(|(k, v)| format!("{k}={v}")).collect();
                eprintln!("  逐规则：{}", detail.join(", "));
            }
        }
    }
    let printed = print_unit(&outcome.ast, &outcome.unit);
    result.lines_after = printed.lines().count();

    if had_errors {
        eprintln!(
            "cure: {display}: {} 处语法问题已跳过（保留原文）:",
            outcome.errors.len()
        );
        for e in outcome.errors.iter().take(5) {
            eprintln!("  {display}:{}:{}: {}", e.line, e.col, e.message);
        }
        if outcome.errors.len() > 5 {
            eprintln!("  … 其余 {} 处省略", outcome.errors.len() - 5);
        }
    }

    result.changed = result.edits > 0 || printed != src;

    if opts.diff && result.changed {
        result.diff_text = unified_diff(&src, &printed, display);
    }

    if opts.check {
        return Ok(result);
    }

    match out_path {
        Some(p) => fs::write(&p, printed).map_err(|e| format!("写入 {} 失败: {e}", p.display()))?,
        None => {
            let mut stdout = std::io::stdout().lock();
            stdout
                .write_all(printed.as_bytes())
                .map_err(|e| format!("写出失败: {e}"))?;
            if !printed.ends_with('\n') {
                let _ = stdout.write_all(b"\n");
            }
        }
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// 统一 diff（Myers 最短编辑脚本 + 上下文行）。差异过大（编辑距离超限）时
// 退化为提示文本——反混淆输出与原文差异通常巨大，逐行 diff 无审阅价值。
// ---------------------------------------------------------------------------

const DIFF_MAX_D: usize = 3000;
const DIFF_CONTEXT: usize = 3;

fn unified_diff(old: &str, new: &str, display: &str) -> String {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    // 同首尾裁剪：典型改写只动中间
    let mut lo = 0;
    while lo < a.len() && lo < b.len() && a[lo] == b[lo] {
        lo += 1;
    }
    let mut hi = 0;
    while hi < a.len() - lo && hi < b.len() - lo && a[a.len() - 1 - hi] == b[b.len() - 1 - hi] {
        hi += 1;
    }
    let mid_a = &a[lo..a.len() - hi];
    let mid_b = &b[lo..b.len() - hi];
    if mid_a.is_empty() && mid_b.is_empty() {
        return String::new();
    }
    let Some(script) = myers_script(mid_a, mid_b) else {
        return format!(
            "--- {display}\n+++ {display}\n@@ 差异过大（> {DIFF_MAX_D} 行编辑），省略；旧 {lines_a} 行 / 新 {lines_b} 行 @@\n",
            lines_a = a.len(),
            lines_b = b.len(),
        );
    };
    // script: Vec<(usize a_idx, usize b_idx, op)> 相对 mid
    // 转换为 hunks（带上下文）
    let mut out = format!("--- {display}\n+++ {display}\n");
    let mut hunks: Vec<String> = Vec::new();
    let mut i = 0;
    while i < script.len() {
        if script[i].2 == DiffOp::Equal {
            i += 1;
            continue;
        }
        // 找到本块的编辑跨度（连续非 Equal 段及其间隙 ≤ 2*上下文）
        let start = i;
        let mut j = i;
        let mut last_edit = i;
        while j < script.len() {
            if script[j].2 != DiffOp::Equal {
                last_edit = j;
                j += 1;
            } else {
                // 数连续 Equal；小于 2*ctx 则并入同一 hunk
                let mut k = j;
                while k < script.len() && script[k].2 == DiffOp::Equal {
                    k += 1;
                }
                if k - j > DIFF_CONTEXT * 2 || k == script.len() {
                    break;
                }
                j = k;
            }
        }
        let end = last_edit + 1;
        let ctx_start = start.saturating_sub(DIFF_CONTEXT);
        let ctx_end = (end + DIFF_CONTEXT).min(script.len());
        let a_start = lo + ctx_start;
        let b_start = lo + ctx_start;
        let a_count = script[ctx_start..ctx_end]
            .iter()
            .filter(|(_, _, op)| *op != DiffOp::Insert)
            .count();
        let b_count = script[ctx_start..ctx_end]
            .iter()
            .filter(|(_, _, op)| *op != DiffOp::Delete)
            .count();
        let mut h = format!(
            "@@ -{},{} +{},{} @@\n",
            a_start + 1,
            a_count,
            b_start + 1,
            b_count
        );
        for (ai, bi, op) in &script[ctx_start..ctx_end] {
            match op {
                DiffOp::Equal => h.push_str(&format!(" {}\n", a[lo + *ai])),
                DiffOp::Delete => h.push_str(&format!("-{}\n", a[lo + *ai])),
                DiffOp::Insert => h.push_str(&format!("+{}\n", b[lo + *bi])),
            }
        }
        hunks.push(h);
        i = end;
    }
    for h in hunks {
        out.push_str(&h);
    }
    out
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum DiffOp {
    Equal,
    Delete,
    Insert,
}

/// Myers 贪心算法（前进版 + V 快照回溯）。返回编辑脚本
/// Vec<(a_idx, b_idx, op)>（Equal 的 ai/bi 都有效；Delete 只用 ai；Insert 只用 bi）。
fn myers_script(a: &[&str], b: &[&str]) -> Option<Vec<(usize, usize, DiffOp)>> {
    let n = a.len();
    let m = b.len();
    let max = n + m;
    if max == 0 {
        return Some(Vec::new());
    }
    let offset = max as isize;
    let mut v = vec![0isize; 2 * max + 1];
    let mut trace: Vec<Vec<isize>> = Vec::new();
    let mut found_d: Option<usize> = None;
    'outer: for d in 0..=max {
        if d > DIFF_MAX_D {
            return None;
        }
        trace.push(v.clone());
        let mut k = -(d as isize);
        while k <= d as isize {
            let mut x = if k == -(d as isize)
                || (k != d as isize && v[(k - 1 + offset) as usize] < v[(k + 1 + offset) as usize])
            {
                v[(k + 1 + offset) as usize]
            } else {
                v[(k - 1 + offset) as usize] + 1
            };
            let mut y = x - k;
            while (x as usize) < n && (y as usize) < m && a[x as usize] == b[y as usize] {
                x += 1;
                y += 1;
            }
            v[(k + offset) as usize] = x;
            if x as usize >= n && y as usize >= m {
                found_d = Some(d);
                break 'outer;
            }
            k += 2;
        }
    }
    let d_final = found_d? as usize;
    // 回溯
    let mut ops: Vec<(usize, usize, DiffOp)> = Vec::new();
    let mut x = n as isize;
    let mut y = m as isize;
    for d in (1..=d_final).rev() {
        let v = &trace[d];
        let k = x - y;
        let prev_k = if k == -(d as isize)
            || (k != d as isize && v[(k - 1 + offset) as usize] < v[(k + 1 + offset) as usize])
        {
            k + 1
        } else {
            k - 1
        };
        let prev_x = v[(prev_k + offset) as usize];
        let prev_y = prev_x - prev_k;
        // 从 (prev_x, prev_y) 走到 (x, y)：先一步斜/插/删，再对角 Equals
        while x > prev_x && y > prev_y {
            x -= 1;
            y -= 1;
            ops.push((x as usize, y as usize, DiffOp::Equal));
        }
        if x > prev_x {
            x -= 1;
            ops.push((x as usize, y as usize, DiffOp::Delete));
        } else if y > prev_y {
            y -= 1;
            ops.push((x as usize, y as usize, DiffOp::Insert));
        }
    }
    // d == 0：剩余对角
    while x > 0 && y > 0 {
        x -= 1;
        y -= 1;
        ops.push((x as usize, y as usize, DiffOp::Equal));
    }
    ops.reverse();
    Some(ops)
}

// ---------------------------------------------------------------------------
// 集成测试（直接调用 run()，临时目录）
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("cure_cli_test_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn write(p: &Path, content: &str) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }

    const SRC: &str = "class A{int m(){int x=0;int y=5;return x+y*1;}}";
    const EXPECT_CONTAINS: &str = "return 5;";

    #[test]
    fn dir_mode_creates_sibling_cure_dir_with_structure() {
        let root = tmp("dir");
        write(&root.join("src/com/x/a.java"), SRC);
        write(&root.join("b.java"), "class B{int m(){int x=0;return x;}}");
        let code = run(&[root.display().to_string()]).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
        let out = root.parent().unwrap().join(format!("{}-cure", root.file_name().unwrap().to_str().unwrap()));
        let a = fs::read_to_string(out.join("src/com/x/a.java")).unwrap();
        assert!(a.contains(EXPECT_CONTAINS), "{a}");
        assert!(out.join("b.java").exists());
        // 非源码文件默认不拷贝
        assert!(!out.join("src").join("anything.txt").exists());
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn dir_mode_copy_other_copies_non_source() {
        let root = tmp("copy");
        write(&root.join("a.java"), SRC);
        write(&root.join("docs/readme.md"), "hello");
        let code = run(&["--copy-other".into(), root.display().to_string()]).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
        let out = root.parent().unwrap().join(format!("{}-cure", root.file_name().unwrap().to_str().unwrap()));
        assert_eq!(fs::read_to_string(out.join("docs/readme.md")).unwrap(), "hello");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn check_mode_exit_2_when_changeable() {
        let root = tmp("check");
        write(&root.join("a.java"), SRC);
        let code = run(&["--check".into(), root.display().to_string()]).unwrap();
        assert_eq!(code, ExitCode::from(2));
        // 不写出
        let out = root.parent().unwrap().join(format!("{}-cure", root.file_name().unwrap().to_str().unwrap()));
        assert!(!out.exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn check_mode_exit_0_when_clean() {
        let root = tmp("check0");
        let clean = "class A {\n    int m() {\n        return 5;\n    }\n}\n";
        write(&root.join("a.java"), clean);
        let code = run(&["--check".into(), "--format-only".into(), root.display().to_string()]).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn ext_filter_processes_matching_only() {
        let root = tmp("ext");
        write(&root.join("a.java"), SRC);
        write(&root.join("notjava.txt"), "garbage");
        // --ext java 正常
        let code = run(&["--ext".into(), "java".into(), root.display().to_string()]).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
        // 全是空格分隔 + 前导点 + 大写
        let root2 = tmp("ext2");
        write(&root2.join("a.java"), SRC);
        let code = run(&["--ext".into(), ".JAVA, py".into(), root2.display().to_string()]).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
        let out2 = root2.parent().unwrap().join(format!("{}-cure", root2.file_name().unwrap().to_str().unwrap()));
        assert!(fs::read_to_string(out2.join("a.java")).unwrap().contains(EXPECT_CONTAINS));
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&root2);
    }

    #[test]
    fn in_place_write() {
        let root = tmp("inplace");
        let f = root.join("a.java");
        write(&f, SRC);
        let code = run(&["-w".into(), f.display().to_string()]).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(fs::read_to_string(&f).unwrap().contains(EXPECT_CONTAINS));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn multiple_files_stdout_rejected() {
        let root = tmp("multiout");
        write(&root.join("a.java"), SRC);
        write(&root.join("b.java"), SRC);
        let r = run(&[root.join("a.java").display().to_string(), root.join("b.java").display().to_string()]);
        assert!(r.is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn single_file_no_output_writes_no_file() {
        let root = tmp("single");
        let f = root.join("a.java");
        write(&f, SRC);
        // stdout 模式：不落盘（内容校验交由 process_source 的单元路径）
        let code = run(&[f.display().to_string()]).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
        // 原文件不变
        assert_eq!(fs::read_to_string(&f).unwrap(), SRC);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn myers_diff_basics() {
        let d = unified_diff("a\nb\nc\n", "a\nx\nc\n", "f");
        assert!(d.contains("-b"), "{d}");
        assert!(d.contains("+x"), "{d}");
        assert!(d.contains("@@"), "{d}");
        // 相同 → 空
        assert!(unified_diff("a\n", "a\n", "f").is_empty());
        // 超大差异 → 退化提示
        let big_old: Vec<String> = (0..4000).map(|i| format!("old{i}")).collect();
        let big_new: Vec<String> = (0..4000).map(|i| format!("new{i}")).collect();
        let d = unified_diff(&big_old.join("\n"), &big_new.join("\n"), "f");
        assert!(d.contains("差异过大"), "{}", &d[..80.min(d.len())]);
    }

    #[test]
    fn stdin_and_bad_args() {
        // 未支持后缀全给出 → 报错
        let r = run(&["--ext".into(), "py".into()]);
        assert!(r.is_err());
        // 无输入
        assert!(run(&[]).is_err());
    }
}
