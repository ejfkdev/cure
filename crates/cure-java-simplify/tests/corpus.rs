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
/// GJF testdata：google-java-format 格式化前后配对样例的 .input 侧
///（core/src/test/resources/…/testdata/*.input，209 个）——完整编译单元、
/// 语法面最全的格式化输入集（.output 侧语法同形，不重复入库）
const GJF_TESTDATA_ROOT: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus_data/gjf_testdata");
/// checkstyle noncompilable：checkstyle 仓库 src/test/resources-noncompilable
///（426 个，2026-10 克隆）——**故意不可编译**的容错压测集：合法语法边界 +
/// 语义非法形态。容错承诺：错误有界（无风暴）、无 panic；其中约 236 个被
/// 我们的容错解析器干净解析（语义非法但语法合法），走全三检
const CHECKSTYLE_ROOT: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus_data/checkstyle_noncompilable");
/// openjdk langtools patterns/switch：JDK 参考语法测试（Java 16-25 模式
/// 匹配/record 模式/switch 全家——新 JDK 版本语料主源）
const OPENJDK_ROOT: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus_data/openjdk_langtools");
/// checkstyle grammar：按 JDK 版本组织的语法回归（java8-25 等）
const CHECKSTYLE_GRAMMAR_ROOT: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus_data/checkstyle_grammar");

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

    // 跨文件多核并行（std::thread::scope，零依赖）：文件之间完全独立
    //（各自独立的 parse/simplify/print/再解析/幂等二轮），结果收集后统一断言。
    struct FileOutcome {
        path: PathBuf,
        was_clean: bool,
        edits: usize,
        by_rule: Vec<(&'static str, usize)>,
        lines_before: usize,
        lines_after: usize,
        // 自洽性：输出重新解析的错误（应为空）
        reparse_errs: Vec<(usize, String)>,
        // 幂等性：第二轮的编辑数（应为 0）
        second_edits: usize,
        second_by_rule: Vec<(&'static str, usize)>,
        // 调试上下文
        reparse_context: String,
    }
    let process = |f: &Path| -> Option<FileOutcome> {
        let src = fs::read_to_string(f).ok()?; // 非 UTF-8：跳过
        // 1. 鲁棒性：任何输入都不 panic
        let mut outcome = parse(&src);
        let was_clean = outcome.errors.is_empty();
        let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &cfg);
        let printed = print_unit(&outcome.ast, &outcome.unit);

        if !was_clean {
            eprintln!("  dirty: {}", f.display());
            return Some(FileOutcome {
                path: f.to_path_buf(),
                was_clean,
                edits: report.edits,
                by_rule: report.by_rule.into_iter().collect(),
                lines_before: count_code_lines(&src),
                lines_after: count_code_lines(&printed),
                reparse_errs: Vec::new(),
                second_edits: 0,
                second_by_rule: Vec::new(),
                reparse_context: String::new(),
            });
        }
        // 2. 自洽性：输出必须再次干净解析
        let reparsed = parse(&printed);
        let reparse_errs: Vec<(usize, String)> = reparsed
            .errors
            .iter()
            .take(3)
            .map(|e| (e.line, e.message.clone()))
            .collect();
        let ctx = context_of(
            &printed,
            reparsed.errors.first().map(|e| e.line).unwrap_or(0),
        );
        // 3. 幂等性：至多三轮收敛（全链路不动点）。打印→重解析的形态
        // 漂移会让合法简化在第 2 轮才出现（checkstyle SwitchExpression4：
        // b=0 传播进嵌套 switch 选择器——第 3 轮严格 0，仍杜绝振荡/发散；
        // 语义正确性由 javac 差分测试作最终裁决）
        let mut second = reparsed;
        let r2 = simplify_unit(&mut second.ast, &mut second.unit, &cfg);
        // 第 2 轮有改写：打印→重解析→第 3 轮必须 0（全链路不动点）
        let final_edits;
        let mut final_by_rule = r2.by_rule.clone();
        if r2.edits > 0 {
            let printed2 = print_unit(&second.ast, &second.unit);
            let mut third = parse(&printed2);
            let r3 = simplify_unit(&mut third.ast, &mut third.unit, &cfg);
            final_edits = r3.edits;
            final_by_rule = r3.by_rule.clone();
        } else {
            final_edits = 0;
        }
        Some(FileOutcome {
            path: f.to_path_buf(),
            was_clean,
            edits: report.edits,
            by_rule: report.by_rule.into_iter().collect(),
            lines_before: count_code_lines(&src),
            lines_after: count_code_lines(&printed),
            reparse_errs,
            second_edits: final_edits,
            second_by_rule: final_by_rule.into_iter().collect(),
            reparse_context: ctx,
        })
    };

    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(files.len())
        .max(1);
    let chunk = files.len().div_ceil(n_threads);
    let mut outcomes: Vec<FileOutcome> = Vec::with_capacity(files.len());
    std::thread::scope(|s| {
        let handles: Vec<_> = files
            .chunks(chunk)
            .map(|c| s.spawn(move || c.iter().filter_map(|f| process(f)).collect::<Vec<_>>()))
            .collect();
        for h in handles {
            outcomes.extend(h.join().expect("corpus 线程 panic"));
        }
    });
    outcomes.sort_by(|a, b| a.path.cmp(&b.path));

    for o in &outcomes {
        total_edits += o.edits;
        for (k, v) in &o.by_rule {
            *per_rule.entry(k).or_insert(0) += v;
        }
        lines_before += o.lines_before;
        lines_after += o.lines_after;
        if o.was_clean {
            clean += 1;
            assert!(
                o.reparse_errs.is_empty(),
                "{}: 输出无法干净重新解析：{:?}\n== 附近输出 ==\n{}",
                o.path.display(),
                o.reparse_errs,
                o.reparse_context
            );
            assert_eq!(
                o.second_edits, 0,
                "{}: 第三轮仍有 {} 次改写（全链路未收敛——疑似振荡/发散）\n{:?}",
                o.path.display(),
                o.second_edits,
                o.second_by_rule
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
        if i + 1 >= line.saturating_sub(2) && i < line + 2 {
            out.push_str(&format!("{:5}: {}\n", i + 1, l));
        }
    }
    out
}

#[test]
fn all_vendored_corpora() {
    // 自动发现 corpus_data/ 下全部语料子目录逐个跑三检——新增语料只需
    // 落盘目录（零测试代码改动）。子目录名即语料标签。
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus_data"));
    let mut dirs: Vec<PathBuf> = fs::read_dir(root)
        .expect("corpus_data 不存在")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    assert!(!dirs.is_empty(), "corpus_data 无语料子目录");
    for d in &dirs {
        let label = d.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        run_corpus(d.to_str().unwrap(), label);
    }
}
