//! # cure CLI
//!
//! `cure` —— 容错式 Java 代码简化 / 格式化命令行工具。
//!
//! - 输入：Java 源码文件（或 stdin）
//! - 输出：简化 + 规范格式化后的源码（stdout 或 `--output`）
//! - 语法错误不阻断：错误区域原文保留，其余照常优化（`--strict` 时非零退出）

use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use cure_engine::Config;
use cure_java_parser::parse;
use cure_java_print::print_unit;
use cure_java_simplify::simplify_unit;

const USAGE: &str = "\
cure — 容错式 Java 代码简化 / 格式化工具

用法:
  cure [选项] <文件.java>...     处理一个或多个 Java 源文件
  cure [选项] -                  从 stdin 读取

选项:
  -o, --output <文件>     输出路径（默认 stdout；多输入时不可用）
  -w, --write             原地覆写输入文件
      --format-only       仅格式化，不做简化
      --check             不写出内容；有可优化改写时退出码 2，否则 0
      --report            在 stderr 打印逐规则改写统计
      --disable <规则名>  禁用指定规则（可多次）
      --strict            有解析错误时退出码 1（默认仅 stderr 提示）
  -h, --help              本帮助
  -V, --version           版本
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
    files: Vec<PathBuf>,
    output: Option<PathBuf>,
    in_place: bool,
    format_only: bool,
    check: bool,
    report: bool,
    disabled: Vec<String>,
    dead_code: bool,
    strict: bool,
    stdin: bool,
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut opts = Options {
        files: Vec::new(),
        output: None,
        in_place: false,
        format_only: false,
        check: false,
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
            "--report" => opts.report = true,
            "--strict" => opts.strict = true,
            "--dead-code" => opts.dead_code = true,
            "-o" | "--output" => {
                i += 1;
                let v = args.get(i).ok_or("missing value for --output")?;
                opts.output = Some(PathBuf::from(v));
            }
            "--disable" => {
                i += 1;
                let v = args.get(i).ok_or("missing value for --disable")?;
                opts.disabled.push(v.clone());
            }
            other if other.starts_with('-') => {
                return Err(format!("未知选项 `{other}`"));
            }
            other => opts.files.push(PathBuf::from(other)),
        }
        i += 1;
    }
    if opts.stdin && !opts.files.is_empty() {
        return Err("stdin (`-`) 与文件参数不能同时使用".into());
    }
    if opts.output.is_some() && (opts.files.len() > 1 || opts.stdin) {
        return Err("--output 只支持单输入".into());
    }
    if opts.check && (opts.in_place || opts.output.is_some() || opts.format_only) {
        return Err("--check 不能与 --write/--output/--format-only 组合".into());
    }
    if opts.files.is_empty() && !opts.stdin {
        return Err("缺少输入（文件或 `-` 表示 stdin）".into());
    }
    Ok(opts)
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let opts = parse_args(args)?;
    let mut any_changed = false;
    let mut any_error = false;

    if opts.stdin {
        let mut src = String::new();
        std::io::stdin()
            .read_to_string(&mut src)
            .map_err(|e| format!("读取 stdin 失败: {e}"))?;
        let (changed, errored) = process_source(&src, &opts, None)?;
        any_changed |= changed;
        any_error |= errored;
        return Ok(final_code(any_changed, any_error, &opts));
    }

    // 多文件并行（std::thread::scope，零依赖）：文件之间完全独立。
    // stdout 输出模式只允许单文件（parse_args 已保证），并行时全部走
    // -w/-o 或 --check，无输出交错问题；stderr 诊断按文件前缀。
    if opts.files.len() > 1 && opts.output.is_none() {
        let n_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(opts.files.len())
            .max(1);
        let chunk = opts.files.len().div_ceil(n_threads);
        let owned: Vec<Vec<PathBuf>> = opts.files.chunks(chunk).map(|c| c.to_vec()).collect();
        let opts2 = opts.clone();
        let results: Vec<Result<Vec<(bool, bool)>, String>> = std::thread::scope(|s| {
            let handles: Vec<_> = owned
                .into_iter()
                .map(|c| {
                    let opts = opts2.clone();
                    s.spawn(move || {
                        let mut out = Vec::new();
                        for path in &c {
                            let src = fs::read_to_string(path).map_err(|e| {
                                format!("读取 {} 失败: {e}", path.display())
                            })?;
                            let out_path = if opts.in_place {
                                Some(path.clone())
                            } else {
                                None
                            };
                            let r = process_source(&src, &opts, out_path)?;
                            out.push(r);
                        }
                        Ok(out)
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().expect("cure 线程 panic")).collect()
        });
        for r in results {
            for (changed, errored) in r? {
                any_changed |= changed;
                any_error |= errored;
            }
        }
        return Ok(final_code(any_changed, any_error, &opts));
    }

    for (idx, path) in opts.files.iter().enumerate() {
        let src = fs::read_to_string(path)
            .map_err(|e| format!("读取 {} 失败: {e}", path.display()))?;
        if opts.files.len() > 1 && idx > 0 {
            eprintln!();
        }
        if opts.files.len() > 1 {
            eprintln!("// {}", path.display());
        }
        let out_path = if opts.in_place {
            Some(path.clone())
        } else {
            opts.output.clone()
        };
        let (changed, errored) = process_source(&src, &opts, out_path)?;
        any_changed |= changed;
        any_error |= errored;
    }
    Ok(final_code(any_changed, any_error, &opts))
}

fn final_code(any_changed: bool, any_error: bool, opts: &Options) -> ExitCode {
    if opts.strict && any_error {
        ExitCode::from(1)
    } else if opts.check && any_changed {
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    }
}

/// 处理单个源码文本；返回 (是否发生改写, 是否有解析错误)。
fn process_source(
    src: &str,
    opts: &Options,
    out_path: Option<PathBuf>,
) -> Result<(bool, bool), String> {
    let mut outcome = parse(src);
    let had_errors = !outcome.errors.is_empty();

    let mut changed = false;
    if !opts.format_only {
        let mut cfg = Config::default();
        for r in &opts.disabled {
            cfg.disabled_rules.insert(r.clone());
        }
        cfg.remove_dead_methods = opts.dead_code;
        let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &cfg);
        if opts.report {
            // 行数统计（注释在词法层被丢弃，这里按总行数计）
            eprintln!(
                "行数 {} → {}（改写 {} 次 / {} 轮）",
                src.lines().count(),
                print_unit(&outcome.ast, &outcome.unit).lines().count(),
                report.edits,
                report.iterations
            );
            eprintln!(
                "逐规则：{}",
                if report.by_rule.is_empty() {
                    "(无)".to_string()
                } else {
                    let detail: Vec<String> = report
                        .by_rule
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect();
                    detail.join(", ")
                }
            );
        }
        changed = report.edits > 0;
    }
    let printed = print_unit(&outcome.ast, &outcome.unit);

    if had_errors {
        eprintln!(
            "cure: {} 处语法问题已跳过（保留原文）:",
            outcome.errors.len()
        );
        for e in outcome.errors.iter().take(10) {
            eprintln!("  {}:{}: {}", e.line, e.col, e.message);
        }
        if outcome.errors.len() > 10 {
            eprintln!("  … 其余 {} 处省略", outcome.errors.len() - 10);
        }
    }

    let changed_output = changed || printed != src;
    if opts.check {
        return Ok((changed_output, had_errors));
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
    Ok((changed_output, had_errors))
}
