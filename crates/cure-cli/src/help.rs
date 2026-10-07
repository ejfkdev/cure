//! 帮助文本与 `rules` 子命令目录（双语，跟随 [`crate::cli_lang`]）。

use crate::cli_lang::CliLang;
use cure_java_simplify::{all_java_rules, default_java_rules};

pub(crate) const REPO_URL: &str = "https://github.com/ejfkdev/cure";

pub(crate) fn print_help() {
    match CliLang::detect() {
        CliLang::Zh => print_help_zh(),
        CliLang::En => print_help_en(),
    }
}

pub(crate) fn print_version() {
    match CliLang::detect() {
        CliLang::Zh => println!(
            "cure {}（容错式 Java 简化/格式化器）\n{REPO_URL}",
            env!("CARGO_PKG_VERSION")
        ),
        CliLang::En => println!(
            "cure {} — fault-tolerant Java simplifier/formatter\n{REPO_URL}",
            env!("CARGO_PKG_VERSION")
        ),
    }
}

fn print_help_en() {
    let v = env!("CARGO_PKG_VERSION");
    println!("cure {v} — fault-tolerant Java simplifier/formatter");
    println!("Decompiled Java in, human-written-looking Java out (semantic-preserving).");
    println!("{REPO_URL}  (MIT license)");
    println!();
    println!("Usage:");
    println!("  cure [OPTIONS] <file.java>...     files; single file → stdout by default");
    println!("  cure [OPTIONS] <dir>...           directory (recursive, parallel)");
    println!("  cure [OPTIONS] -                   read stdin, write stdout");
    println!("  cure rules                         list all simplification rules");
    println!("  cure help [COMMAND]                this help, or one command's");
    println!("  cure version | -h | -V");
    println!();
    println!("Language: CURE_LANG=zh|en forces; otherwise auto-detected from the");
    println!("locale environment (on Windows: the system UI language).");
    println!();
    println!("Directory mode mirrors the tree into <dir>-cure-out/ (hierarchy");
    println!("preserved); -o sets another root, -w rewrites in place. Broken");
    println!("regions of a file are preserved verbatim — a parse error never");
    println!("aborts the run (--strict makes it exit 1 for CI).");
    println!();
    println!("Options:");
    println!("  -o, --output <path>      output path (single file) or output root (dir)");
    println!("  -w, --write              rewrite input files in place");
    println!("      --check              dry run: exit 2 if rewritable; write nothing");
    println!("  -j, --threads <N>        worker threads (default: CPU cores × 1.5)");
    println!("      --format-only        format only, no simplification");
    println!("      --ext <list>         only given extensions (comma/space separated)");
    println!("      --copy-other         dir mode: copy non-source files verbatim");
    println!("      --diff               print unified diffs of rewritten files");
    println!("      --stats              summary statistics (files/lines/rules)");
    println!("      --report             per-file rewrite statistics on stderr");
    println!("      --disable <rule>     disable a rule (repeatable; `cure rules` lists all)");
    println!("      --dead-code          opt-in: drop private methods referenced nowhere");
    println!("      --strict             exit 1 on any parse error (default: warn)");
    println!("  -h, --help               this help");
    println!("  -V, --version            version and repository");
    println!();
    println!("Exit status: 0 ok; 1 --strict with parse errors; 2 usage error /");
    println!("--check found rewrites.");
    println!();
    println!("Examples:");
    println!("  # one file to stdout");
    println!("  echo 'class A{{int m(){{int a=foo();int b=a;return b;}}}}' | cure -");
    println!("  cure Input.java");
    println!();
    println!("  # a directory — mirror into src-cure-out/, hierarchy preserved");
    println!("  cure src/");
    println!("  cure src/ -o out/                  # another output root");
    println!("  cure src/ -w                       # rewrite in place");
    println!();
    println!("  # CI gate: exit 2 when anything is rewritable, write nothing");
    println!("  cure --check src/");
    println!();
    println!("  # see what changed");
    println!("  cure --diff src/ | less");
    println!("  cure --stats src/");
    println!();
    println!("  # tuning");
    println!("  cure -j 32 big-corpus/             # worker threads");
    println!("  cure --disable cff_recover src/    # turn one rule off");
    println!();
    println!("Performance: 371,674-file OpenJDK corpus (4.70 GB) — 14.8 s /");
    println!("~430 MiB with --check, 27.5 s written out (18-core laptop).");
}

fn print_help_zh() {
    let v = env!("CARGO_PKG_VERSION");
    println!("cure {v} — 容错式 Java 简化/格式化器");
    println!("反编译 Java 进，像人手写的 Java 出（语义保持）。");
    println!("{REPO_URL}  (MIT 许可证)");
    println!();
    println!("用法:");
    println!("  cure [选项] <文件.java>...        文件；单文件默认输出到 stdout");
    println!("  cure [选项] <目录>...             目录（递归、并行）");
    println!("  cure [选项] -                     读 stdin、写 stdout");
    println!("  cure rules                        列出全部简化规则");
    println!("  cure help [子命令]                本帮助，或某子命令的帮助");
    println!("  cure version | -h | -V");
    println!();
    println!("语言: CURE_LANG=zh|en 强制指定；否则按 locale 环境自动检测");
    println!("（Windows：系统 UI 语言）。");
    println!();
    println!("目录模式默认镜像到 <目录>-cure-out/（保持层级结构）；-o 指定别的");
    println!("输出根，-w 原地覆写。文件的坏区域原文保留——语法错误不中断运行");
    println!("（--strict 时退出码 1，供 CI 使用）。");
    println!();
    println!("选项:");
    println!("  -o, --output <路径>      输出路径（单文件）或输出根目录（目录）");
    println!("  -w, --write              原地覆写输入文件");
    println!("      --check              干跑：有可优化改写 → 退出码 2，不写出");
    println!("  -j, --threads <N>        并行工作线程数（默认=CPU 核心数 × 1.5）");
    println!("      --format-only        仅格式化，不做简化");
    println!("      --ext <后缀列表>     只处理指定后缀（逗号或空格分隔）");
    println!("      --copy-other         目录模式：不受处理的文件原样拷入输出树");
    println!("      --diff               打印被改写文件的统一 diff");
    println!("      --stats              汇总统计（文件数/行数/逐规则）");
    println!("      --report             stderr 打印逐文件改写统计");
    println!("      --disable <规则名>   禁用指定规则（可多次；`cure rules` 列出全部）");
    println!("      --dead-code          选配：删除全单元零引用的 private 方法");
    println!("      --strict             有解析错误 → 退出码 1（默认仅 stderr 提示）");
    println!("  -h, --help               本帮助");
    println!("  -V, --version            版本与仓库地址");
    println!();
    println!("退出码: 0 正常；1 --strict 且有解析错误；2 参数错误 / --check 发现有可改写。");
    println!();
    println!("示例:");
    println!("  # 单文件到 stdout");
    println!("  echo 'class A{{int m(){{int a=foo();int b=a;return b;}}}}' | cure -");
    println!("  cure Input.java");
    println!();
    println!("  # 目录——镜像到 src-cure-out/，保持层级结构");
    println!("  cure src/");
    println!("  cure src/ -o out/                  # 指定别的输出根");
    println!("  cure src/ -w                       # 原地覆写");
    println!();
    println!("  # CI 门禁：有可优化改写 → 退出码 2，不写出任何内容");
    println!("  cure --check src/");
    println!();
    println!("  # 看改了什么");
    println!("  cure --diff src/ | less");
    println!("  cure --stats src/");
    println!();
    println!("  # 调优");
    println!("  cure -j 32 big-corpus/             # 指定线程数");
    println!("  cure --disable cff_recover src/    # 关掉某条规则");
    println!();
    println!("性能: 371,674 文件 OpenJDK 语料（4.70 GB）——--check 14.8 s / 峰值");
    println!("~430 MiB，写出 27.5 s（18 核笔记本）。");
}

/// `cure rules` / `cure help rules`：规则目录。
pub(crate) fn print_rules_help() {
    match CliLang::detect() {
        CliLang::Zh => print_rules_zh(),
        CliLang::En => print_rules_en(),
    }
}

fn rule_catalog() -> (Vec<&'static str>, Vec<&'static str>) {
    let defaults: Vec<&'static str> = default_java_rules().iter().map(|r| r.name()).collect();
    let all: Vec<&'static str> = all_java_rules().iter().map(|r| r.name()).collect();
    let opt_in: Vec<&'static str> =
        all.into_iter().filter(|n| !defaults.contains(n)).collect();
    (defaults, opt_in)
}

fn print_rules_en() {
    let (defaults, opt_in) = rule_catalog();
    println!("cure rules — {} simplification rules", defaults.len() + opt_in.len());
    println!("Names are what --disable takes.");
    println!();
    println!("Enabled by default ({}):", defaults.len());
    for n in &defaults {
        println!("  {n}");
    }
    println!();
    println!("Opt-in ({}; --dead-code enables unreachable_after_terminal):", opt_in.len());
    for n in &opt_in {
        println!("  {n}");
    }
    println!();
    println!("Example:  cure --disable cff_recover --disable xor_noise src/");
}

fn print_rules_zh() {
    let (defaults, opt_in) = rule_catalog();
    println!("cure rules — 共 {} 条简化规则", defaults.len() + opt_in.len());
    println!("名字即 --disable 的取值。");
    println!();
    println!("默认启用（{} 条）：", defaults.len());
    for n in &defaults {
        println!("  {n}");
    }
    println!();
    println!("选配（{} 条；--dead-code 启用 unreachable_after_terminal）：", opt_in.len());
    for n in &opt_in {
        println!("  {n}");
    }
    println!();
    println!("示例:  cure --disable cff_recover --disable xor_noise src/");
}
