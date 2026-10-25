//! `preserve_function_source`（opt-in）：函数/类/对象字面量逐字节原样
//! 输出，`Function.prototype.toString` 与源文本一致；该模式下简化事实
//! 不得与原文打印相矛盾（保真区引用 → 声明不透明）。

use std::fs;
use std::path::Path;
use std::process::Command;

use cure_engine::Config;
use cure_ts_parser::parse_with_source;
use cure_ts_print::print_module;
use cure_ts_simplify::simplify_unit;

fn node_bin() -> Option<&'static str> {
    for c in ["node", "/Users/e/.local/share/vite-plus/bin/node"] {
        if Command::new(c).arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
            return Some(c);
        }
    }
    None
}

fn run_js(bin: &str, file: &Path) -> (String, Option<i32>) {
    let out = Command::new(bin).arg(file).output().expect("run node");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        out.status.code(),
    )
}

fn preserve_print(src: &str) -> String {
    let mut outcome = parse_with_source(src);
    assert!(
        outcome.errors.is_empty(),
        "测试源码必须干净解析：{:?}",
        outcome.errors
    );
    outcome.ast.set_preserve_source(true);
    let mut cfg = Config::default();
    cfg.preserve_function_source = true;
    let _ = simplify_unit(&mut outcome.ast, &mut outcome.unit, &cfg);
    print_module(&outcome.ast, &outcome.unit)
}

/// 原版 vs 保真输出的运行结果一致（toString 语义含在程序内比对）。
fn preserve_differential(name: &str, src: &str) -> String {
    let simplified = preserve_print(src);
    // 保真输出必须幂等（二次解析/简化/打印逐字节相同）。
    let again = preserve_print(&simplified);
    assert_eq!(simplified, again, "{name}: 保真模式不幂等");
    let Some(node) = node_bin() else {
        eprintln!("skip preserve differential: node not found");
        return simplified;
    };
    let base = std::env::temp_dir().join(format!("cure_ts_pfs_{name}"));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let orig_file = base.join("orig.js");
    let simp_file = base.join("simp.js");
    fs::write(&orig_file, src).unwrap();
    fs::write(&simp_file, &simplified).unwrap();
    let (orig_out, orig_code) = run_js(node, &orig_file);
    let (simp_out, simp_code) = run_js(node, &simp_file);
    assert_eq!(
        orig_code, simp_code,
        "{name}: 退出码改变\n== 原版 ==\n{orig_out}\n== 保真 ==\n{simp_out}\n== 输出 ==\n{simplified}"
    );
    assert_eq!(
        orig_out, simp_out,
        "{name}: 语义改变\n== 原版 ==\n{orig_out}\n== 保真 ==\n{simp_out}\n== 输出 ==\n{simplified}"
    );
    simplified
}

#[test]
fn function_to_string_is_verbatim() {
    let src = r#"function f(a,  b) {
    // keep comment
    return a +  b;
}
const g = async (x) => {
    return x*2;
};
class C {
    m(x) { return x + 1; }
    static s = () => 1;
}
const o = {
    p: 1,
    m(y) {
        return y * 3;
    },
};
const h = function named(q) { return q - 1; };
console.log(f.toString());
console.log(g.toString());
console.log(C.toString());
console.log(C.prototype.m.toString());
console.log(o.m.toString());
console.log(h.toString());
"#;
    let out = preserve_differential("to_string", src);
    assert!(out.contains("// keep comment"), "注释必须保留：\n{out}");
    assert!(out.contains("return a +  b;"), "函数体空白必须保留：\n{out}");
}

#[test]
fn object_literal_shorthand_binding_survives() {
    // 回归：local_propagation 曾把 `async` 传播进对象字面量并删除声明，
    // 保真打印恢复 `{ async }` 后成为悬挂引用（ReferenceError）。
    let src = r#"{
  let async = 11;
  let a = { async };
  console.log(a.async);
}
"#;
    let out = preserve_differential("shorthand_binding", src);
    assert!(out.contains("let async = 11;"), "声明必须保留：\n{out}");
}

#[test]
fn crlf_line_terminators_are_preserved() {
    let src = "function\r\nf\r\n(\r\nx\r\n)\r\n{\r\nreturn x;\r\n}\r\nconsole.log(JSON.stringify(f.toString()));\r\n";
    let out = preserve_differential("crlf", src);
    assert!(out.contains("function\r\nf"), "CRLF 必须逐字节保留：\n{out:?}");
}

#[test]
fn default_mode_keeps_reformatting() {
    let src = "function f(a,  b) { return a +  b; }\nconsole.log(f.toString());\n";
    let mut outcome = cure_ts_parser::parse(src);
    assert!(outcome.errors.is_empty());
    let _ = simplify_unit(&mut outcome.ast, &mut outcome.unit, &Config::default());
    let out = print_module(&outcome.ast, &outcome.unit);
    assert!(!out.contains("return a +  b;"), "默认模式应重排：\n{out}");
}
