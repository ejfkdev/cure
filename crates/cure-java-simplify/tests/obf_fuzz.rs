//! 随机混淆差分模糊测试：生成器按种子随机组合六层混淆变换直接产出
//! 混淆程序（CFF 状态机 / string-array+Base64 解密器 / 寄存器噪声 /
//! 不透明谓词 / SB 语句链 / XOR 副作用包裹 / 死赋值 / 布尔包装三元），
//! cure 处理后编译运行与混淆原版**行为逐字节比对**。
//!
//! 这是覆盖度的根本保障：随机组合产生的样本空间远超手写用例，
//! 任何"守卫漏洞"（组合中某个变换的交互缺陷）都会以行为差异暴露。

use std::fs;
use std::path::Path;
use std::process::Command;

use cure_engine::Config;
use cure_java_parser::parse;
use cure_java_print::print_unit;
use cure_java_simplify::simplify_unit;

// ---------------------------------------------------------------------------
// 种子 LCG
// ---------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn range(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn chance(&mut self, pct: u64) -> bool {
        self.range(100) < pct
    }
}

// ---------------------------------------------------------------------------
// 混淆变换开关
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
struct ObfFlags {
    cff: bool,            // 控制流扁平化
    string_table: bool,    // string-array + Base64 解密器
    reg_noise: bool,       // 循环尾寄存器回拷
    opaque: bool,          // 不透明谓词 + 死分支
    sb_chain: bool,        // StringBuilder 语句链
    xor_wrap: bool,        // XOR 副作用包裹
    dead_store: bool,      // 远距死赋值
    bool_ternary: bool,    // 布尔包装三元
}

const STRING_POOL: &[&str] = &["alpha", "beta", "x", "y2", "pq", "loop", "ok"];

// ---------------------------------------------------------------------------
// 程序生成器：直接产出【混淆形态】的 main 方法体
// ---------------------------------------------------------------------------

struct Gen {
    rng: Rng,
    flags: ObfFlags,
    string_table: Vec<String>, // 明文（生成时填，输出时转 base64）
    out: String,               // main 体源码
    var_n: u32,
    // 行为追踪（生成侧自校验用）
    acc_val: i64,
    trace_val: i64,
}

impl Gen {
    fn int_var(&mut self) -> String {
        let n = self.var_n;
        self.var_n += 1;
        format!("v{n}")
    }

    /// 整数表达式（带 XOR 噪声与三元）
    fn int_expr(&mut self, depth: u32) -> String {
        if depth == 0 {
            return match self.rng.range(3) {
                0 => format!("{}", self.rng.range(10)),
                1 => "acc".into(),
                _ => "i".into(),
            };
        }
        let d = depth - 1;
        match self.rng.range(6) {
            0 => format!("({} + {})", self.int_expr(d), self.int_expr(d)),
            1 => format!("({} * {})", self.int_expr(d), self.rng.range(4)),
            2 => format!("({} ? {} : {})", self.bool_expr(d), self.int_expr(d), self.int_expr(d)),
            3 => {
                let e = self.int_expr(d);
                if self.flags.xor_wrap && self.rng.chance(50) {
                    let k = self.rng.range(200) + 1;
                    format!("(({e} ^ {k}) ^ {k})")
                } else {
                    e
                }
            }
            4 => format!("bump({})", self.rng.range(3) + 1),
            _ => self.int_expr(d),
        }
    }

    fn bool_expr(&mut self, depth: u32) -> String {
        if depth == 0 {
            return match self.rng.range(3) {
                0 => "flag".into(),
                1 => format!("(acc > {})", self.rng.range(20)),
                _ => "true".into(),
            };
        }
        let d = depth - 1;
        match self.rng.range(5) {
            0 => format!("({} && {})", self.bool_expr(d), self.bool_expr(d)),
            1 => format!("({} || {})", self.bool_expr(d), self.bool_expr(d)),
            2 => format!("(!{})", self.bool_expr(d)),
            3 => {
                let e = self.bool_expr(d);
                if self.flags.bool_ternary && self.rng.chance(50) {
                    format!("(({e}) ? true : false)")
                } else {
                    e
                }
            }
            _ => format!("({} < {})", self.int_expr(d), self.int_expr(d)),
        }
    }

    /// 字符串表达式（可走解密器表）
    fn str_expr(&mut self, depth: u32) -> String {
        if depth == 0 {
            return match self.rng.range(3) {
                0 => {
                    let idx = self.rng.range(STRING_POOL.len() as u64) as usize;
                    let s = STRING_POOL[idx];
                    if self.flags.string_table {
                        self.string_table.push(s.to_string());
                        format!("d({})", self.string_table.len() - 1)
                    } else {
                        format!("\"{s}\"")
                    }
                }
                1 => "s".into(),
                _ => {
                    let e = self.int_expr(1);
                    format!("String.valueOf({e})")
                }
            };
        }
        let d = depth - 1;
        match self.rng.range(3) {
            0 => format!("({} + {})", self.str_expr(d), self.str_expr(d)),
            1 => format!("({} + {})", self.str_expr(d), self.int_expr(d)),
            _ => self.str_expr(d),
        }
    }

    /// 简单语句（用于普通序列与 CFF case 体）
    fn simple_stmt(&mut self) -> String {
        match self.rng.range(5) {
            0 => format!("acc = acc + {};", self.int_expr(2)),
            1 => format!("flag = {};", self.bool_expr(2)),
            2 => format!("s = s + {};", self.str_expr(1)),
            3 => format!("acc = acc + bump({});", self.rng.range(3) + 1),
            _ => format!("System.out.println(\"m\" + acc + \":\" + s);"),
        }
    }

    /// 带 CFF 的语句序列发射
    fn emit_stmts(&mut self, count: usize) {
        let mut stmts: Vec<String> = Vec::new();
        for _ in 0..count {
            stmts.push(self.simple_stmt());
        }
        if self.flags.cff && self.rng.chance(70) && stmts.len() >= 2 {
            // CFF：序列 → 状态机
            let stv = self.int_var();
            let exit: u32 = 900;
            self.out.push_str(&format!("int {stv} = 0;\n        while (true) {{\n            switch ({stv}) {{\n"));
            for (i, s) in stmts.iter().enumerate() {
                let next = if i + 1 == stmts.len() { exit } else { (i + 1) as u32 };
                self.out.push_str(&format!(
                    "                case {i}: {{\n                    {s}\n                    {stv} = {next};\n                    break;\n                }}\n"
                ));
            }
            self.out.push_str(&format!(
                "            }}\n            if ({stv} == {exit}) {{\n                break;\n            }}\n        }}\n        "
            ));
        } else {
            for s in &stmts {
                self.out.push_str(&format!("        {s}\n"));
            }
        }
    }

    fn emit_loop(&mut self) {
        let v = self.int_var();
        let bound = self.rng.range(3) + 2;
        self.out.push_str(&format!("int {v} = 0;\n        "));
        if self.flags.reg_noise && self.rng.chance(60) {
            // 寄存器噪声：体内经临时变量回拷
            let t = self.int_var();
            let t2 = self.int_var();
            self.out.push_str(&format!(
                "while ({v} < {bound}) {{\n            int {t} = acc + {v};\n            int {t2} = {t};\n            acc = {t2};\n            {v} = {v} + 1;\n        }}\n        "
            ));
        } else if self.flags.opaque && self.rng.chance(50) {
            self.out.push_str(&format!(
                "while (true) {{\n            if (!({v} < {bound})) {{\n                break;\n            }}\n            acc = acc + {v};\n            {v} = {v} + 1;\n        }}\n        "
            ));
        } else {
            self.out.push_str(&format!(
                "while ({v} < {bound}) {{\n            acc = acc + {v};\n            {v} = {v} + 1;\n        }}\n        "
            ));
        }
    }

    fn emit_if(&mut self) {
        let c = self.bool_expr(2);
        if self.flags.opaque && self.rng.chance(50) {
            let a = self.int_expr(1);
            let b = self.int_expr(1);
            self.out.push_str(&format!(
                "if (2 > 1) {{\n            if ({c}) {{\n                acc = acc + {a};\n            }} else {{\n                acc = acc + {b};\n            }}\n        }} else {{\n            acc = acc + 999999;\n        }}\n        "
            ));
        } else {
            let a = self.int_expr(1);
            let b = self.int_expr(1);
            self.out.push_str(&format!(
                "if ({c}) {{\n            acc = acc + {a};\n        }} else {{\n            acc = acc + {b};\n        }}\n        "
            ));
        }
    }

    fn emit_sb_chain(&mut self) {
        // StringBuilder 语句链（混合重赋值/新变量形态）
        let parts = self.rng.range(3) + 2;
        let sb = self.int_var();
        let sb2 = self.int_var();
        let mut chain = String::from("new StringBuilder().append(s)");
        for _ in 0..parts {
            chain.push_str(&format!(".append({})", self.str_expr(0)));
        }
        self.out.push_str(&format!(
            "StringBuilder {sb} = {chain};\n        {sb} = {sb}.append(\"-\");\n        StringBuilder {sb2} = {sb}.append(\"!\");\n        s = {sb2}.toString();\n        "
        ));
    }

    fn emit_dead_store(&mut self) {
        let dz = self.int_var();
        let v = self.rng.range(9);
        self.out.push_str(&format!(
            "int {dz} = 0;\n        System.out.println(\"dz\" + 0);\n        {dz} = {v};\n        acc = acc + {dz};\n        "
        ));
    }
}

// ---------------------------------------------------------------------------
// 程序组装
// ---------------------------------------------------------------------------

fn gen_program(seed: u64) -> String {
    let mut rng = Rng(seed);
    let flags = ObfFlags {
        cff: rng.chance(60),
        string_table: rng.chance(70),
        reg_noise: rng.chance(60),
        opaque: rng.chance(60),
        sb_chain: rng.chance(50),
        xor_wrap: rng.chance(60),
        dead_store: rng.chance(40),
        bool_ternary: rng.chance(50),
    };
    let mut g = Gen {
        rng,
        flags: flags.clone(),
        string_table: Vec::new(),
        out: String::new(),
        var_n: 0,
        acc_val: 0,
        trace_val: 0,
    };

    let rounds = g.rng.range(6) + 5;
    for _ in 0..rounds {
        let choice = g.rng.range(7);
        let n = g.rng.range(3) + 2;
        match choice {
            0 | 1 => g.emit_stmts(n as usize),
            2 => g.emit_loop(),
            3 => g.emit_if(),
            4 if g.flags.sb_chain => g.emit_sb_chain(),
            5 if g.flags.dead_store => g.emit_dead_store(),
            _ => {
                let stmt = g.simple_stmt();
                g.out.push_str(&format!("        {stmt}\n"));
            }
        }
    }
    g.out.push_str("        System.out.println(\"acc=\" + acc);\n");
    g.out.push_str("        System.out.println(\"s=\" + s);\n");
    g.out.push_str("        System.out.println(\"flag=\" + flag);\n");

    // 类组装：bump 副作用方法 + 可选解密器
    let mut src = String::from("public class Fz {\n    static int trace = 0;\n\n");
    src.push_str("    static int bump(int v) {\n        trace += v;\n        return v;\n    }\n\n");
    if !g.string_table.is_empty() {
        let items: Vec<String> = g
            .string_table
            .iter()
            .map(|s| format!("\"{}\"", base64_encode(s)))
            .collect();
        src.push_str(&format!(
            "    static final String[] T = new String[]{{{}}};\n\n",
            items.join(", ")
        ));
        src.push_str(
            "    static String d(int i) {\n        return new String(java.util.Base64.getDecoder().decode(T[i]));\n    }\n\n",
        );
    }
    src.push_str("    public static void main(String[] args) {\n");
    src.push_str("        int acc = 0;\n        boolean flag = true;\n        String s = \"\";\n        int i = 1;\n        ");
    src.push_str(&g.out);
    src.push_str("        System.out.println(\"trace=\" + trace);\n");
    src.push_str("    }\n}\n");
    src
}

fn base64_encode(s: &str) -> String {
    const TBL: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = s.as_bytes();
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(TBL[((n >> 18) & 63) as usize] as char);
        out.push(TBL[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TBL[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TBL[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

// ---------------------------------------------------------------------------
// 差分执行
// ---------------------------------------------------------------------------

fn compile_and_run(dir: &Path, class: &str) -> (String, Option<i32>) {
    let run = Command::new("java")
        .arg("-cp")
        .arg(dir)
        .arg(class)
        .output()
        .expect("run java");
    (
        String::from_utf8_lossy(&run.stdout).to_string(),
        run.status.code(),
    )
}

fn javac_available() -> bool {
    Command::new("javac")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn obf_fuzz_differential() {
    if !javac_available() {
        eprintln!("skip: javac not found");
        return;
    }
    // 16 个种子的全差分（生成组合混淆 → cure → 行为对拍）
    // 种子空间远超此数；CI 期望 ~30s
    let seeds: Vec<u64> = (1..=16).collect();
    let mut total_edits = 0usize;
    let mut applied_flags = std::collections::BTreeSet::new();

    for seed in seeds {
        let src = gen_program(seed);
        let cls = format!("Fz{seed}");
        let src = src.replace("Fz", &cls);

        // 解析 + 简化
        let mut outcome = parse(&src);
        assert!(
            outcome.errors.is_empty(),
            "seed {seed}: 生成的混淆程序解析失败：{:?}",
            &outcome.errors[..outcome.errors.len().min(3)]
        );
        let report = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
        let cured = print_unit(&outcome.ast, &outcome.unit);
        total_edits += report.edits;
        for k in report.by_rule.keys() {
            applied_flags.insert(*k);
        }

        // 编译运行双方
        let base = std::env::temp_dir().join(format!("cure_fuzz_{seed}"));
        let _ = fs::remove_dir_all(&base);
        let orig_dir = base.join("orig");
        let clean_dir = base.join("clean");
        fs::create_dir_all(&orig_dir).unwrap();
        fs::create_dir_all(&clean_dir).unwrap();
        fs::write(orig_dir.join(format!("{cls}.java")), &src).unwrap();
        fs::write(clean_dir.join(format!("{cls}.java")), &cured).unwrap();
        for d in [&orig_dir, &clean_dir] {
            let out = Command::new("javac")
                .arg("-nowarn")
                .arg("-d")
                .arg(d)
                .arg(d.join(format!("{cls}.java")))
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "seed {seed}: javac 失败：\n{}\n== 混淆源 ==\n{src}\n== 净化输出 ==\n{cured}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        let (a_out, a_code) = compile_and_run(&orig_dir, &cls);
        let (b_out, b_code) = compile_and_run(&clean_dir, &cls);
        assert_eq!(a_code, b_code, "seed {seed}: 退出码不一致");
        assert_eq!(
            a_out, b_out,
            "seed {seed}: 混淆还原改变行为！\n== 混淆源 ==\n{src}\n== 净化输出 ==\n{cured}\n== 混淆版行为 ==\n{a_out}\n== 净化版行为 ==\n{b_out}"
        );

        // 幂等性：第二轮 0 改写
        let mut second = parse(&cured);
        let r2 = simplify_unit(&mut second.ast, &mut second.unit, &Config::default());
        assert_eq!(r2.edits, 0, "seed {seed}: 第二轮仍有 {} 次改写", r2.edits);

        // 简化确实发生（组合混淆不应全部被跳过）
        assert!(
            report.edits >= 2,
            "seed {seed}: 预期显著简化，实际 {} 次\n== 源 ==\n{src}",
            report.edits
        );
    }

    eprintln!(
        "obf_fuzz: 16 个组合混淆程序全部行为一致，共 {total_edits} 次改写；触发规则：{:?}",
        applied_flags
    );
    assert!(total_edits >= 40, "组合触发过弱：{total_edits}");
}

#[test]
fn obf_fuzz_parse_only_more_seeds() {
    // 更多种子：不编译，只验证（1）容错解析 0 错误（2）不 panic
    // （3）输出可重新干净解析（自洽）（4）幂等
    for seed in 17..=48u64 {
        let src = gen_program(seed).replace("Fz", &format!("Fz{seed}"));
        let mut outcome = parse(&src);
        assert!(
            outcome.errors.is_empty(),
            "seed {seed}: 解析失败：{:?}",
            &outcome.errors[..outcome.errors.len().min(2)]
        );
        simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
        let cured = print_unit(&outcome.ast, &outcome.unit);
        let reparsed = parse(&cured);
        assert!(
            reparsed.errors.is_empty(),
            "seed {seed}: 输出不可重新解析：{:?}",
            &reparsed.errors[..reparsed.errors.len().min(2)]
        );
        let mut second = reparsed;
        let r2 = simplify_unit(&mut second.ast, &mut second.unit, &Config::default());
        assert_eq!(r2.edits, 0, "seed {seed}: 第二轮仍有改写");
    }
    eprintln!("obf_fuzz_parse_only: 种子 17-48 全部通过（解析/自洽/幂等）");
}
