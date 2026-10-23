//! # cure-ts-simplify
//!
//! TS/JS 简化门面：收集模块全部可执行体（顶层语句/函数体/类方法/
//! 静态块/字段初始化器），对每个根跑 [`cure_engine`] 的不动点规则
//! 引擎（默认规则集 + TS 专属规则）。
//!
//! JS 语义安全由 [`cure_ts_ast`] 的 Lang 实现承担：
//! - 数值折叠走 f64/ToInt32 域（±2^53 精确性回验；-0/NaN/±Inf 拒折）
//! - `var` 提升陷阱（块外引用 → is_opaque）
//! - 可选链元素不透明（链短路语义）
//! - 解构模式不透明（绑定名不可见）
//! - is_bool 仅语法级（比较/逻辑/!）——真值不确定性由规则守卫覆盖


use cure_engine::{Config, Report, Rule};
use cure_ts_ast::{
    simple_params, ClassDecl, ClassMember, ModuleUnit, NodeData, TopItem, TsAst, TsId,
};

/// 单根简化（引擎默认规则 + TS 专属规则）。
pub fn simplify(ast: &mut TsAst, root: TsId, cfg: &Config) -> Report {
    let rules = default_ts_rules();
    cure_engine::simplify(ast, root, &rules, cfg)
}

/// 整个模块：逐体优化，签名/类型层不动。
pub fn simplify_unit(ast: &mut TsAst, unit: &mut ModuleUnit, cfg: &Config) -> Report {
    let mut total = Report::default();
    for _cycle in 0..6 {
        let mut round = Report::default();
        for item in &mut unit.items {
            simplify_top_item(ast, item, cfg, &mut round);
        }
        total.edits += round.edits;
        total.iterations += round.iterations;
        total.discarded += round.discarded;
        for (k, v) in round.by_rule {
            *total.by_rule.entry(k).or_insert(0) += v;
        }
        if round.edits == 0 {
            break;
        }
    }
    total
}

fn simplify_top_item(ast: &mut TsAst, item: &mut TopItem, cfg: &Config, total: &mut Report) {
    match item {
        TopItem::Stmt(s) => {
            simplify_stmt_root(ast, s, cfg, total);
        }
        TopItem::Func { sig_raw, body, .. } => {
            // 参数名种入作用域基表（简单参数；模式参数不种——保守）
            let params = sig_params(sig_raw);
            ast.set_param_scope(&params);
            if let Some(b) = body {
                let r = simplify(ast, *b, cfg);
                merge_report(total, r);
            }
            ast.clear_param_scope();
        }
        TopItem::Class(c) => {
            simplify_class(ast, c, cfg, total);
        }
        _ => {}
    }
}

fn simplify_class(ast: &mut TsAst, c: &mut ClassDecl, cfg: &Config, total: &mut Report) {
    for m in &mut c.members {
        simplify_class_member(ast, m, cfg, total);
    }
}

fn simplify_class_member(ast: &mut TsAst, m: &mut ClassMember, cfg: &Config, total: &mut Report) {
    match m {
        ClassMember::Method { sig_raw, body } => {
            let params = sig_params(sig_raw);
            ast.set_param_scope(&params);
            if let Some(b) = body {
                let r = simplify(ast, *b, cfg);
                merge_report(total, r);
            }
            ast.clear_param_scope();
        }
        ClassMember::StaticBlock { body } => {
            let r = simplify(ast, *body, cfg);
            merge_report(total, r);
        }
        ClassMember::Field { init, .. } => {
            if let Some(i) = init {
                simplify_stmt_root(ast, i, cfg, total);
            }
        }
        ClassMember::Raw(_) => {}
    }
}

/// 顶层语句根：普通语句直接跑；VarDecl init 根折叠（引擎 Replace 无
/// 父槽的既有边界——由本侧拿新根回写）。
fn simplify_stmt_root(ast: &mut TsAst, s: &mut TsId, cfg: &Config, total: &mut Report) {
    if matches!(ast.data(*s), NodeData::VarDecl { .. }) {
        // 声明根：init 折叠后回写孩子
        if let Some(&init) = ast.children(*s).first() {
            let rules = default_ts_rules();
            let mut cur = init;
            let mut new_edits = 0;
            for _ in 0..8 {
                match cure_engine::fold_root(ast, cur, &rules, cfg) {
                    Some((new_root, name)) => {
                        ast.set_child(*s, 0, new_root);
                        cur = new_root;
                        *total.by_rule.entry(name).or_insert(0) += 1;
                        new_edits += 1;
                    }
                    None => break,
                }
            }
            if new_edits > 0 {
                total.edits += new_edits;
                let r = simplify(ast, *s, cfg);
                merge_report(total, r);
            } else {
                let r = simplify(ast, *s, cfg);
                merge_report(total, r);
            }
        } else {
            let r = simplify(ast, *s, cfg);
            merge_report(total, r);
        }
        return;
    }
    let r = simplify(ast, *s, cfg);
    merge_report(total, r);
}

/// 签名原文的首对圆括号内参数（简单名）。
fn sig_params(sig_raw: &str) -> Vec<String> {
    let open = match sig_raw.find('(') {
        Some(o) => o,
        None => return vec![],
    };
    // 平衡找配对 `)`
    let bytes = sig_raw.as_bytes();
    let mut depth = 0i32;
    let mut close = None;
    for (i, &b) in bytes.iter().enumerate().skip(open) {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = match close {
        Some(c) => c,
        None => return vec![],
    };
    let inner = &sig_raw[open + 1..close];
    simple_params(inner).unwrap_or_default()
}

fn merge_report(total: &mut Report, r: Report) {
    total.edits += r.edits;
    total.iterations += r.iterations;
    total.discarded += r.discarded;
    for (k, v) in r.by_rule {
        *total.by_rule.entry(k).or_insert(0) += v;
    }
}

/// TS 规则集：引擎默认规则（Lang 钩子承担 JS 语义守卫）+ TS 专属规则。
pub fn default_ts_rules() -> Vec<Box<dyn Rule<TsAst>>> {
    cure_engine::rules::default_rules()
}

// ---------------------------------------------------------------------------
// 参数作用域桥（TsAst 的 param_scope 为私有——本 crate 的门面是唯一
// 合法写入方）。SetParamScope trait 桥接。
// ---------------------------------------------------------------------------

/// 参数作用域注入 trait（pub API——parser/simplify 门面共用）。
pub trait SetParamScope {
    fn set_param_scope(&mut self, params: &[String]);
    fn clear_param_scope(&mut self);
}

impl SetParamScope for TsAst {
    fn set_param_scope(&mut self, params: &[String]) {
        self.param_scope_set(params);
    }
    fn clear_param_scope(&mut self) {
        self.param_scope_clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cure_ts_parser::parse;
    use cure_ts_print::print_module;

    fn run(src: &str) -> (String, Report) {
        let mut out = parse(src);
        assert!(out.errors.is_empty(), "{:#?}", out.errors);
        let report = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
        (print_module(&out.ast, &out.unit), report)
    }

    #[test]
    fn const_propagation_and_fold() {
        // 寄存器噪声：var 传播 + 常量折叠 + 死存储
        let (out, r) = run(
            "function f() {\n  var a = 2;\n  var b = 3;\n  var c = a * b;\n  return c + 0;\n}\n",
        );
        assert!(r.edits >= 2, "edits={}", r.edits);
        assert!(out.contains("return 6"), "{out}");
    }

    #[test]
    fn dead_store_removal() {
        // 死存储 + 传播链全折叠：a=1 死 → a=2 → b=a → return b ⇒ return 2
        let (out, r) = run("function f() {\n  var a = 1;\n  a = 2;\n  var b = a;\n  return b;\n}\n");
        assert!(r.edits >= 2, "edits={}", r.edits);
        assert!(out.contains("return 2"), "{out}");
    }

    #[test]
    fn var_hoisting_trap_respected() {
        // var 块外引用：声明不可删（var_trap 不透明）
        let (out, _r) = run(
            "function f(c) {\n  if (c) { var x = 1; }\n  return x;\n}\n",
        );
        assert!(out.contains("var x = 1"), "{out}");
    }

    #[test]
    fn if_to_ternary() {
        // if-else 赋值 → 三元 + 单使用传播 ⇒ return c ? 1 : 2
        let (out, r) = run(
            "function f(c) {\n  var r;\n  if (c) { r = 1; } else { r = 2; }\n  return r;\n}\n",
        );
        assert!(r.edits >= 1, "edits={}", r.edits);
        assert!(out.contains("return c ? 1 : 2"), "{out}");
    }

    #[test]
    fn semantics_js_safe() {
        // 2 ** 53 边界外的整数字面量不折
        let (out, _r) = run("function f() { return 9007199254740993 + 1; }\n");
        assert!(out.contains("9007199254740993 + 1"), "{out}");
    }

    #[test]
    fn semantics_undefined_not_touched() {
        // undefined 是全局名——不得当字面量处理
        let (out, _r) = run("function f(u) { return u ?? undefined; }\n");
        assert!(out.contains("undefined"), "{out}");
    }

    #[test]
    fn loop_body_simplify() {
        // 循环体内的常量折叠与声明合并照常发生
        let (out, r) = run(
            "function f(n) {\n  var i = 0;\n  while (i < n) { var t = 1 + 1; use(t); i = i + 1; }\n  return i;\n}\n",
        );
        assert!(r.edits >= 1, "edits={}", r.edits);
        assert!(out.contains("use(2)"), "{out}");
    }

    #[test]
    fn idempotent() {
        let src = "function f(n) {\n  var a = 2;\n  var b = a * 3;\n  if (b > 5) { return b + 1; }\n  return b - 1;\n}\n";
        let (once, _) = run(src);
        let mut out2 = parse(&once);
        assert!(out2.errors.is_empty(), "{:#?}\n== once ==\n{once}", out2.errors);
        let _ = simplify_unit(&mut out2.ast, &mut out2.unit, &Config::default());
        let twice = print_module(&out2.ast, &out2.unit);
        assert_eq!(once, twice, "\n== once ==\n{once}\n== twice ==\n{twice}");
    }

    #[test]
    fn class_method_bodies() {
        let (out, r) = run(
            "class A {\n  m(n) {\n    var t = n * 2;\n    var u = t + 1;\n    return u + 0;\n  }\n}\n",
        );
        assert!(r.edits >= 1, "edits={}", r.edits);
        assert!(out.contains("return n * 2 + 1"), "{out}");
    }

    #[test]
    fn arrow_bodies() {
        let (out, r) = run("const f = (a, b) => { var c = a + b; return c + 0; };\n");
        assert!(r.edits >= 1, "edits={}", r.edits);
        assert!(out.contains("return a + b"), "{out}");
    }
}
