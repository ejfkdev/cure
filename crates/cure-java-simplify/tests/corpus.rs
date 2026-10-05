//! 语料库验证：对真实 Java 代码（google-java-format 仓库源码）跑全链路，验证：
//! 1. **鲁棒性**：任何文件都不 panic（容错承诺）；
//! 2. **自洽性**：干净解析的文件，其格式化输出必须能再次干净解析
//!    （打印器产出合法 Java）；
//! 3. **幂等性**：对输出再跑一轮 simplify 必须是 0 改写（fixed point 稳定，
//!    解析→打印→解析→简化 全链路收敛）；
//! 4. **统计**：改写次数 / 逐规则触发 / 行数变化（优化能力度量）。
//!
//! 语料不存在时跳过。

use std::fs;
use std::path::{Path, PathBuf};

use cure_engine::Config;
use cure_java_parser::parse;
use cure_java_print::print_unit;
use cure_java_simplify::simplify_unit;

/// 语料已复制进仓库（tests/corpus_data/）：代码演进后可随时全量重跑，
/// 不依赖本机外部 checkout。
/// - gjf/：google-java-format 仓库 84 个手写源文件（规范代码基准）
/// - fernflower_obf/：fernflower 测试集 66 个真实 ProGuard 混淆输出（目标域）
const CORPUS_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus_data/gjf");
const OBF_CORPUS_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus_data/fernflower_obf");

fn collect_java_files(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == ".git" || name == "node_modules" {
                continue;
            }
            collect_java_files(&p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("java") {
            out.push(p);
        }
    }
}

#[test]
fn corpus_robust_consistent_idempotent() {
    run_corpus(CORPUS_ROOT, "google-java-format");
}

#[test]
fn obfuscated_corpus_robust_consistent_idempotent() {
    run_corpus(OBF_CORPUS_ROOT, "fernflower-obfuscated");
}

fn run_corpus(root: &str, label: &str) {
    if !Path::new(root).is_dir() {
        eprintln!("skip corpus {label}: {root} not found");
        return;
    }
    let mut files = Vec::new();
    collect_java_files(Path::new(root), &mut files);
    files.sort();
    assert!(!files.is_empty(), "corpus {label} empty?");

    let cfg = Config::default();
    let mut clean = 0usize;
    let mut dirty = 0usize;
    let mut total_edits = 0usize;
    // 注释在词法层被丢弃（反编译产物无注释；格式化保真是后续项），
    // 统计用"非空非注释行"做公平对比
    let mut lines_before = 0usize;
    let mut lines_after = 0usize;
    let mut per_rule: std::collections::BTreeMap<&'static str, usize> = Default::default();

    for f in &files {
        let src = match fs::read_to_string(f) {
            Ok(s) => s,
            Err(_) => continue, // 非 UTF-8 文件：跳过读取，不 panic
        };
        // 1. 鲁棒性：任何输入都不 panic
        let mut outcome = parse(&src);
        let was_clean = outcome.errors.is_empty();
        let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &cfg);
        let printed = print_unit(&outcome.ast, &outcome.unit);

        total_edits += report.edits;
        for (k, v) in report.by_rule {
            *per_rule.entry(k).or_insert(0) += v;
        }
        lines_before += count_code_lines(&src);
        lines_after += count_code_lines(&printed);
        if !was_clean {
            eprintln!("  dirty: {}", f.display());
        }

        if was_clean {
            clean += 1;
            // 2. 自洽性：输出必须再次干净解析
            let reparsed = parse(&printed);
            assert!(
                reparsed.errors.is_empty(),
                "{}: 输出无法干净重新解析：{:?}\n== 附近输出 ==\n{}",
                f.display(),
                &reparsed.errors[..reparsed.errors.len().min(3)],
                context_of(&printed, reparsed.errors.first().map(|e| e.line).unwrap_or(0))
            );
            // 3. 幂等性：第二轮 0 改写
            let mut second = reparsed;
            let r2 = simplify_unit(&mut second.ast, &mut second.unit, &cfg);
            assert_eq!(
                r2.edits, 0,
                "{}: 第二轮仍有 {} 次改写（未收敛）\n{:?}",
                f.display(),
                r2.edits,
                &r2.by_rule
            );
        } else {
            dirty += 1;
        }
    }

    eprintln!(
        "corpus[{label}]: {} 文件（干净 {} / 含语法错误 {}），{} 次改写，{} 行 → {} 行",
        clean + dirty,
        clean,
        dirty,
        total_edits,
        lines_before,
        lines_after
    );
    for (k, v) in &per_rule {
        eprintln!("  rule {k}: {v}");
    }
    if total_edits > 0 {
        // 混淆语料期望高改写率（这正是目标域）；干净手写语料期望低改写率
        // （证明"不乱动好代码"）—— 两者都不能是 0（证明引擎在工作）
        eprintln!(
            "平均每千行改写: {:.1}",
            total_edits as f64 * 1000.0 / lines_before as f64
        );
    }
}

/// 去掉注释与空行后的行数（粗粒度词法剥离）。
fn count_code_lines(src: &str) -> usize {
    let mut out = 0usize;
    let mut in_block = false;
    for line in src.lines() {
        let mut l = line.trim();
        if in_block {
            if let Some(i) = l.find("*/") {
                in_block = false;
                l = l[i + 2..].trim();
            } else {
                continue;
            }
        }
        if l.starts_with("/*") && !l.ends_with("*/") {
            in_block = true;
            continue;
        }
        if l.starts_with("//") || l.starts_with("/*") || l.is_empty() {
            continue;
        }
        out += 1;
    }
    out
}

fn context_of(text: &str, line: usize) -> String {
    let mut out = String::new();
    for (i, l) in text.lines().enumerate() {
        if i + 1 >= line.saturating_sub(2) && i + 1 <= line + 2 {
            out.push_str(&format!("{:5}: {}\n", i + 1, l));
        }
    }
    out
}
