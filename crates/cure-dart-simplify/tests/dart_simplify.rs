use std::path::PathBuf;
use std::process::Command;

use cure_dart_parser::parse;
use cure_dart_print::print_module;
use cure_dart_simplify::simplify_module;
use cure_engine::Config;

fn cured(src: &str) -> String {
    cured_with(src, &Config::default())
}

fn cured_with(src: &str, cfg: &Config) -> String {
    let mut out = parse(src);
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    simplify_module(&mut out.ast, cfg);
    print_module(&out.ast)
}

fn assert_idempotent(src: &str) {
    let once = cured(src);
    let twice = cured(&once);
    assert_eq!(once, twice, "not idempotent:\n{once}\n---\n{twice}");
}

#[test]
fn constant_folding_domains() {
    let out = cured(
        r#"void main() {
  var a = 1 + 2 * 3;
  var b = -5 % 3;
  var c = 5 % -3;
  var d = -5 ~/ 3;
  var e = 1 << 30;
  var f = true && false;
  var g = 'a' + 'b';
  var h = !true;
  var i = 1 << 31;
  var j = 1 / 2;
  var k = 'a' + x;
  var l = 2147483647 + 1;
  var m = -1;
}
"#,
    );
    for needle in [
        "var a = 7;",
        "var b = 1;",
        "var c = 2;",
        "var d = -1;",
        "var e = 1073741824;",
        "var f = false;",
        "var g = 'ab';",
        "var h = false;",
        "var m = -1;",
    ] {
        assert!(out.contains(needle), "missing {needle:?}\n{out}");
    }
    for needle in ["var i = 1 << 31;", "var j = 1 / 2;", "var k = 'a' + x;", "var l = 2147483647 + 1;"] {
        assert!(out.contains(needle), "should stay: {needle:?}\n{out}");
    }
    assert_idempotent(&out);
}

#[test]
fn constant_if_selects_block_and_removes_empty_else() {
    let out = cured(
        r#"void main() {
  if (true) { print(1); } else { print(2); }
  if (false) { print(3); } else { print(4); }
  if (1 < 2) { print(5); }
  if (x) { print(6); } else { }
}
"#,
    );
    assert!(!out.contains("if (true)"), "{out}");
    assert!(!out.contains("if (false)"), "{out}");
    assert!(!out.contains("if (1 < 2)"), "{out}");
    assert!(!out.contains("print(2)"), "{out}");
    assert!(!out.contains("print(3)"), "{out}");
    assert!(out.contains("print(1)"), "{out}");
    assert!(out.contains("print(4)"), "{out}");
    assert!(out.contains("print(5)"), "{out}");
    assert!(out.contains("if (x)"), "{out}");
    assert!(!out.contains("else"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn constant_if_respects_comments() {
    let out = cured("void main(){ if (true) { /* keep */ print(1); } }");
    assert!(out.contains("if (true)"), "{out}");
    assert!(out.contains("/* keep */"), "{out}");
}

#[test]
fn atom_parens_and_new_is_preserved() {
    let out = cured(
        r#"class Foo { Foo(); }
void main() {
  var a = new Foo();
  var b = (a);
  var c = (a).toString();
  f((a));
  var d = (a) => a;
  var e = (a) { return a; };
}
"#,
    );
    // Dart 1 required `new`; keep it for cross-version soundness.
    assert!(out.contains("var a = new Foo();"), "{out}");
    assert!(out.contains("var b = a;"), "{out}");
    assert!(out.contains("var c = a.toString();"), "{out}");
    assert!(out.contains("f(a);"), "{out}");
    assert!(out.contains("var d = (a) => a;"), "{out}");
    assert!(out.contains("var e = (a) {"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn adjacent_string_literals_merge() {
    let out = cured("void main(){ print('a' 'b'); print(r'c' \"d\"); print('e' '$f'); }");
    assert!(out.contains("print('ab');"), "{out}");
    // raw/插值字符串不在安全域内。
    assert!(out.contains("r'c' \"d\""), "{out}");
    assert!(out.contains("'e' '$f'"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn control_and_parameter_parens_stay() {
    let out = cured("void main(){ if ((x)) { print(x); } }");
    assert!(out.contains("if (x)"), "{out}");
    let out = cured("void f(int x) { print(x); }");
    assert!(out.contains("void f(int x)"), "{out}");
}

#[test]
fn interpolation_compaction_is_safe() {
    let out = cured(
        r#"void main() {
  var a = '${x}';
  var b = '${x}y';
  var c = r'${x}';
  var d = '${x + 1}';
  var e = '${this}';
  var f = '${f('x')}';
  var g = '$${x}';
}
"#,
    );
    assert!(out.contains("var a = '$x';"), "{out}");
    for needle in ["var b = '${x}y';", "var c = r'${x}';", "var d = '${x + 1}';", "var e = '${this}';", "var f = '${f('x')}';", "var g = '$${x}';"] {
        assert!(out.contains(needle), "changed: {needle:?}\n{out}");
    }
    // 组合符号是 Dart 的 ID_Continue；`${x}` 后紧跟组合符号时不能压缩成
    // `$x́`（会变成另一个标识符）。
    let mark = cured("void main() { var x = 1; var h = '${x}\u{0301}'; }");
    assert!(mark.contains("'${x}\u{0301}'"), "{mark}");
}

#[test]
fn disable_rules_is_honored() {
    let mut cfg = Config::default();
    cfg.disabled_rules.insert("dart_const_fold".into());
    cfg.disabled_rules.insert("dart_adjacent_strings".into());
    let out = cured_with("void main(){ var a = 1 + 2; print('x' 'y'); }", &cfg);
    assert!(out.contains("1 + 2"), "{out}");
    assert!(out.contains("'x' 'y'"), "{out}");
}

#[test]
fn dart_versions_parse_and_stay_idempotent() {
    let src = r#"typedef int Legacy(int x);
typedef Old = int Function(int);
library legacy;
part of foo;
class A<T> with M implements I {
  final int x;
  A(this.x);
  factory A.zero() => new A(0);
  int get v => x;
  set v(int value) {}
  void operator []=(String k, Object v) {}
}
extension type ET(int i) { int get v => i; }
sealed class S {}
final class F extends S {}
void main() {
  var (a, b) = (1, 2);
  switch (a) { case 1: print(b); break; default: print(0); }
  var r = (x: 1, y: 'z');
  var s = switch (a) { 1 => 'one', _ => 'other' };
  print('$r $s ${a + b}');
}
"#;
    let out = cured(src);
    assert_idempotent(&out);
    assert!(out.contains("typedef int Legacy(int x);"), "{out}");
    assert!(out.contains("extension type ET(int i)"), "{out}");
    assert!(out.contains("sealed class S"), "{out}");
}

#[test]
fn dart_runtime_differential_if_available() {
    if Command::new("dart").arg("--version").output().is_err() {
        return;
    }
    let src = r#"void main() {
  print(1 + 2 * 3);
  print(-5 % 3);
  print(5 % -3);
  print(-5 ~/ 3);
  print(1 << 30);
  print(true && false);
  print('a' + 'b');
  var xs = new List<int>.filled(2, 0);
  if (true) { print(xs.length); } else { print(99); }
  print('${xs.length}');
}
"#;
    let cured = cured(src);
    let a = run_dart(src);
    let b = run_dart(&cured);
    assert_eq!(a, b, "runtime mismatch; cured:\n{cured}");
}

#[test]
fn cured_output_is_valid_dart_if_available() {
    if Command::new("dart").arg("--version").output().is_err() {
        return;
    }
    let src = r#"sealed class A {}
final class B extends A {}
extension type E(int i) {}
void main() {
  var (a, b) = (1, 2);
  switch (a) { case 1: print(b); break; default: print(0); }
  var r = (x: 1, y: 'z');
  print('$r ${1 + 2}');
}
"#;
    let cured = cured(src);
    let path = temp_path("dart_syntax", "dart");
    std::fs::write(&path, &cured).unwrap();
    let out = Command::new("dart")
        .arg("format")
        .arg("--output=none")
        .arg(&path)
        .output()
        .expect("run dart format");
    let _ = std::fs::remove_file(&path);
    assert!(
        out.status.success(),
        "dart format rejected output:\n{cured}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn run_dart(src: &str) -> (Option<i32>, Vec<u8>, Vec<u8>) {
    let path = temp_path("dart_run", "dart");
    std::fs::write(&path, src).unwrap();
    let out = Command::new("dart")
        .arg("run")
        .arg(&path)
        .output()
        .expect("run dart");
    let _ = std::fs::remove_file(&path);
    (out.status.code(), out.stdout, out.stderr)
}

fn temp_path(prefix: &str, ext: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "{prefix}_{}_{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    p
}
#[test]
fn constant_conditions_keep_flow_sensitive_branches() {
    let out = cured("void f() { late int v; if (1 > 2) { v = 1; } else { } v; }");
    // Dart 分析器不会把 1 > 2 当常量；改成 false 会改变 late 变量的
    // definite assignment 诊断。
    assert!(out.contains("if (1 > 2)"), "{out}");

    let out2 = cured("void g() { if (1 < 2) { print(1); } }");
    assert!(!out2.contains("if"), "{out2}");

    let out3 = cured(
        "void h() { late int v; if (1 < 2 ? false : true) { v = 1; } v; }",
    );
    assert!(out3.contains("1 < 2 ?"), "{out3}");
    assert_idempotent(&out3);

    let out4 = cured(
        "void j() { late int v; var x = (1 > 2) ? (v = 1) : 0; v; }",
    );
    assert!(out4.contains("(1 > 2) ?"), "{out4}");
}

#[test]
fn else_after_terminal_respects_statement_position() {
    let f = cured(
        "int f(bool a, bool b) {\n\
           var x = 0;\n\
           if (a) { x = 1; } else if (b) { throw 'e'; } else { x = 2; }\n\
           return x;\n\
         }\n",
    );
    // 内层 else-if 是外层 else 的无括号子语句：去掉它的 else 会让裸块
    // 在 a 为 true 时也执行。
    assert!(f.contains("else if (b)"), "{f}");
    assert!(f.contains("} else {"), "{f}");

    let g = cured(
        "int g(bool a, bool b) {\n\
           var x = 0;\n\
           while (a) if (b) { break; } else { x = 1; }\n\
           return x;\n\
         }\n",
    );
    // while 的无括号 body 同理：else 块不能漏到循环外。
    assert!(g.contains("} else {"), "{g}");

    let h = cured(
        "int h(bool a) {\n\
           if (a) { return 1; } else { print(2); }\n\
           return 3;\n\
         }\n",
    );
    assert!(!h.contains("else"), "{h}");
    assert!(h.contains("print(2)"), "{h}");
    assert_idempotent(&h);

    let lab = cured(
        "int lab(bool a, bool b) {\n\
           label: if (a) { return 1; } else { if (b) { break label; } return 2; }\n\
           return 3;\n\
         }\n",
    );
    assert!(lab.contains("break label;"), "{lab}");
    assert!(lab.contains("} else {"), "{lab}");
}

#[test]
fn if_return_does_not_cross_block_boundaries() {
    let out = cured(
        "int f(bool unordered, Object o) {\n\
           if (!unordered) {\n\
             if (o is List) { return 1; }\n\
           } else if (o is Iterable) {\n\
             return 2;\n\
           }\n\
           return 3;\n\
         }\n\
         int g(bool a, bool b) {\n\
           if (a) { if (b) { return 1; } }\n\
           return 2;\n\
         }\n",
    );
    // 外层 if 之后的 return 不能被 else-if 分支吞掉。
    assert!(out.contains("return 3;"), "{out}");
    assert!(!out.contains("else return"), "{out}");
    assert!(out.contains("return 2;"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn if_return_and_if_assign_become_ternaries() {
    let out = cured(
        "int f(int c) {\n\
           if (c > 0) { return 1; } else { return 2; }\n\
         }\n\
         int g(int c) {\n\
           var x = 0;\n\
           if (c > 0) { x = 5; } else { x = 6; }\n\
           return x;\n\
         }\n",
    );
    assert!(out.contains("return (c > 0) ? 1 : 2;"), "{out}");
    assert!(out.contains("x = (c > 0) ? 5 : 6;"), "{out}");
    assert!(!out.contains("if (c > 0)"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn if_case_patterns_are_never_rewritten() {
    let src = "int f(Object x) {\n\
                 if (x case int y) { return y; }\n\
                 return 0;\n\
               }\n";
    let out = cured(src);
    assert!(out.contains("x case int y"), "{out}");
    assert!(out.contains("return y;"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn collection_if_elements_are_never_rewritten() {
    let src = "void main() {\n\
                 var x = 1;\n\
                 var xs = [if (true) x = 1 else x = 2];\n\
                 print(xs);\n\
               }\n";
    let out = cured(src);
    assert!(out.contains("if (true) x = 1 else x = 2"), "{out}");
}

#[test]
fn unreachable_statements_are_removed_conservatively() {
    let out = cured(
        "void a() {\n\
           print('x');\n\
           return;\n\
           print('dead');\n\
           cleanup();\n\
         }\n\
         void b() {\n\
           throw Exception('x');\n\
           print('dead');\n\
         }\n\
         int c() {\n\
           return 1;\n\
           return 2;\n\
         }\n\
         void d() {\n\
           late int v;\n\
           for (var _ in [0]) { break; v = 1; }\n\
           v;\n\
         }\n",
    );
    assert!(!out.contains("print('dead')"), "{out}");
    assert!(!out.contains("cleanup()"), "{out}");
    assert!(out.contains("return 1;"), "{out}");
    // 删除 return/yield 会影响 Dart 的函数/生成器返回类型推断，必须保留。
    assert!(out.contains("return 2;"), "{out}");
    // 不可达赋值会影响 late 变量的 definite assignment 分析，必须保留。
    assert!(out.contains("v = 1;"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn trailing_return_with_body_comments_is_kept() {
    let out = cured(
        "void f() {\n\
           // 刻意保留作 dart2js 内联屏障\n\
           print(1);\n\
           return;\n\
         }\n\
         void g() {\n\
           print(2);\n\
           return;\n\
         }\n",
    );
    assert!(out.contains("return;"), "{out}");
    assert_eq!(out.matches("return;").count(), 1, "{out}");
    assert_idempotent(&out);
}

#[test]
fn trailing_continue_and_empty_finally_are_cleaned() {
    let out = cured(
        "void f(int c) {\n\
           while (c > 0) {\n\
             print(c);\n\
             continue;\n\
           }\n\
           for (var i = 0; i < c; i++) {\n\
             print(i);\n\
             continue;\n\
           }\n\
           do {\n\
             print(c);\n\
             continue;\n\
           } while (c > 0);\n\
           try {\n\
             print(c);\n\
           } catch (e) {\n\
             print(e);\n\
           } finally {\n\
           }\n\
         }\n",
    );
    assert!(!out.contains("continue"), "{out}");
    assert!(!out.contains("finally"), "{out}");
    assert!(out.contains("while (c > 0)"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn empty_finally_without_catch_is_unwrapped() {
    let out = cured("void f() { try { print(1); } finally { } }");
    assert!(!out.contains("finally"), "{out}");
    assert!(out.contains("print(1);"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn ternary_rules_do_not_cross_assignment_boundaries() {
    let out = cured(
        "bool f(bool? x, bool c) { x ??= c ? false : true; return x; }\n\
         int g(int x, bool c) { x += c ? 1 : 2; return x; }\n\
         int h(int x, bool c) { x = c ? 1 : 2; return x; }\n\
         bool i(bool c) { var x = false; x = c ? false : true; return x; }\n",
    );
    assert!(out.contains("x ??= !c;"), "{out}");
    assert!(out.contains("x += c ? 1 : 2;"), "{out}");
    assert!(out.contains("x = c ? 1 : 2;"), "{out}");
    assert!(out.contains("x = !c;"), "{out}");
    assert!(!out.contains("!(x ??="), "{out}");
    assert!(!out.contains("!(x +="), "{out}");
    assert_idempotent(&out);
}

#[test]
fn ternary_and_literal_logic_fold() {
    let out = cured(
        "bool a(bool c) { return c ? true : false; }\n\
         bool b(bool c) { return c ? false : true; }\n\
         bool c1(bool x) { return true ? x : !x; }\n\
         bool d(bool x) { return x ? true : (false && x); }\n\
         bool e(bool x) { return true && x; }\n\
         bool f(bool x) { return false || x; }\n\
         bool g(bool x) { return false && x; }\n\
         bool h(bool x) { return true || x; }\n\
         bool i(bool x) { return x && true; }\n\
         bool j(bool x) { return x || false; }\n\
         bool k(bool x) { return x && false; }\n\
         bool l(bool x) { return !x ? false : true; }\n\
         bool m(bool x, bool y) { return true && (x == y); }\n\
         bool n(bool x, bool y) { return false || (x != y); }\n\
         bool o(bool x, bool y) { return (x == y) && true; }\n\
         bool p(bool x, bool y) { return (x != y) || false; }\n\
         bool q(bool x) { return true && !x; }\n",
    );
    for needle in [
        "bool a(bool c) {\n  return c ? true : false;\n}",
        "bool b(bool c) {\n  return !c;\n}",
        "bool c1(bool x) {\n  return true ? x : !x;\n}",
        "bool d(bool x) {\n  return x ? true : false;\n}",
        "bool e(bool x) {\n  return true && x;\n}",
        "bool f(bool x) {\n  return false || x;\n}",
        "bool g(bool x) {\n  return false;\n}",
        "bool h(bool x) {\n  return true;\n}",
        "bool i(bool x) {\n  return x && true;\n}",
        "bool j(bool x) {\n  return x || false;\n}",
        "bool k(bool x) {\n  return x && false;\n}",
        "bool l(bool x) {\n  return !!x;\n}",
        "bool m(bool x, bool y) {\n  return (x == y);\n}",
        "bool n(bool x, bool y) {\n  return (x != y);\n}",
        "bool o(bool x, bool y) {\n  return (x == y);\n}",
        "bool p(bool x, bool y) {\n  return (x != y);\n}",
        "bool q(bool x) {\n  return !x;\n}",
    ] {
        assert!(out.contains(needle), "missing {needle:?}\n{out}");
    }
    assert_idempotent(&out);
}

#[test]
fn bitwise_precedence_matches_dart() {
    let out = cured(
        "bool a(int x) { return x & 0x60 == 0; }\n\
         bool b() { return 1 & 2 == 2; }\n\
         bool c() { return 0x60 & 0x60 == 0x60; }\n\
         int d() { return 1 | 2 & 3; }\n\
         int e() { return 8 >> 1 & 3; }\n",
    );
    // Dart 里 `&` 比 `==` 紧：`x & 0x60 == 0` 是 `(x & 0x60) == 0`。
    assert!(out.contains("x & 0x60 == 0"), "{out}");
    assert!(out.contains("return false;"), "{out}");
    assert!(out.contains("return true;"), "{out}");
    assert!(out.contains("return 3;"), "{out}");
    assert!(out.contains("return 0;"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn const_fold_respects_unary_and_associativity() {
    let out = cured(
        "int a(int x) { return x - (1 + 2); }\n\
         int b() { return -1 * 2; }\n\
         int c() { return -(1 + 2); }\n\
         bool d(bool x) { return !true && x; }\n\
         int e() { return 1 + 2 * 3; }\n\
         int f(int x) { return x - 1 + 2; }\n\
         int g() { return 5 % 3; }\n\
         int h() { return -5 % 3; }\n",
    );
    assert!(out.contains("return x - 3;"), "{out}");
    assert!(out.contains("return -2;"), "{out}");
    assert!(out.contains("return -3;"), "{out}");
    assert!(out.contains("!true && x"), "{out}");
    assert!(out.contains("return 7;"), "{out}");
    assert!(out.contains("x - 1 + 2"), "{out}");
    assert!(out.contains("return 2;"), "{out}");
    assert!(out.contains("return 1;"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn literal_folding_respects_operator_grouping() {
    let out = cured(
        "bool a(bool x) { return x == false || x == true; }\n\
         int b(int x) { return x - 1 + 2; }\n\
         int c(int x) { return x + 1 * 2; }\n\
         bool d(bool x) { return x && true || true; }\n\
         bool e(bool x) { return x == true && x; }\n\
         bool f(int x) { return x == 1 + 2; }\n",
    );
    assert!(out.contains("x == false || x == true"), "{out}");
    assert!(out.contains("x - 1 + 2"), "{out}");
    assert!(out.contains("x + 1 * 2"), "{out}");
    // 未知 bool 变量上的 `&& true` 不再删（动态非 bool 原本会抛错）。
    assert!(out.contains("x && true || true"), "{out}");
    assert!(out.contains("x == true && x"), "{out}");
    assert!(out.contains("x == 3"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn switch_expression_patterns_are_never_rewritten() {
    let src = "String f(Object v) {\n\
                 return switch (v) {\n\
                   true && true => 'a',\n\
                   true || false => 'b',\n\
                   int x when true && x > 2 => 'c',\n\
                   _ => 'd',\n\
                 };\n\
               }\n\
               String g(int v) => switch (v) { _ => true ? 'x' : 'y' };\n";
    let out = cured(src);
    assert!(out.contains("true && true =>"), "{out}");
    assert!(out.contains("true || false =>"), "{out}");
    assert!(out.contains("when true && x > 2"), "{out}");
    // 分支体是普通表达式，仍可安全折叠常量三元。
    assert!(out.contains("_ => 'x'"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn bool_literal_rules_skip_patterns() {
    let src = "String f(Object v) {\n\
                 switch (v) {\n\
                   case true && true:\n\
                     return 'y';\n\
                   default:\n\
                     return 'n';\n\
                 }\n\
               }\n\
               bool g(Object v) {\n\
                 if (v case true && true) { return true; }\n\
                 return false;\n\
               }\n";
    let out = cured(src);
    assert!(out.contains("case true && true:"), "{out}");
    assert!(out.contains("v case true && true"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn negated_comparison_and_bang_forms() {
    let out = cured(
        "void h(void Function(bool)? fn, bool b) { fn!(b == true); }\n\
         bool a(Object x) { return !(x is String); }\n\
         bool b(Object x) { return !(x == null); }\n\
         bool c(Object x) { return !(x != null); }\n\
         bool d(bool x) { return !!(x == true); }\n\
         bool e(bool? x) { return x!; }\n\
         bool f(bool? x) { return !x!; }\n\
         bool g(bool x) { return !!x; }\n",
    );
    assert!(out.contains("fn!(b == true);"), "{out}");
    assert!(out.contains("return x is! String;"), "{out}");
    assert!(out.contains("return x != null;"), "{out}");
    assert!(out.contains("return x == null;"), "{out}");
    assert!(out.contains("return (x == true);"), "{out}");
    assert!(out.contains("return x!;"), "{out}");
    assert!(out.contains("return !x!;"), "{out}");
    assert!(out.contains("return !!x;"), "{out}");
    assert!(!out.contains("return!!"), "{out}");
    assert!(!out.contains("is!String"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn assign_compound_keeps_rhs_grouping() {
    let out = cured(
        "int a(int x, int d) { x = x * 10 + d; return x; }\n\
         int b(int x, int y) { x = x - 1 + y; return x; }\n\
         int c(int x, int y) { x = x + y * 2; return x; }\n\
         int d(int x, int y) { x = x + (y + 1); return x; }\n\
         int e(int x, int y) { x = x << y + 1; return x; }\n\
         int f(int x, int y) { x = x << y << 1; return x; }\n",
    );
    assert!(out.contains("x = x * 10 + d;"), "{out}");
    assert!(out.contains("x = x - 1 + y;"), "{out}");
    assert!(out.contains("x += y * 2;"), "{out}");
    assert!(out.contains("x += (y + 1);"), "{out}");
    assert!(out.contains("x <<= y + 1;"), "{out}");
    assert!(out.contains("x = x << y << 1;"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn assign_compound_nested_if_and_empty_then() {
    let out = cured(
        "int _v = 1;\n\
         int get x => _v;\n\
         set x(int v) { _v = v; }\n\
         int a(int x, int y) { x = x + y; x = x * 2; return x; }\n\
         int b(int x, int y) { x = x ?? y; return x; }\n\
         int top(int y) { x = x ?? y; return x; }\n\
         void c(bool a, bool b) { if (a) { if (b) { print(1); } } }\n\
         void d(bool a) { if (a) { } else { print(2); } }\n\
         void e(bool a) { if (a) { print(3); } }\n\
         void f(bool a, bool b) { if (a) { if (b) { print(4); } else { print(5); } } }\n\
         void g(bool a) { if (a) { } else if (a) { print(6); } }\n",
    );
    assert!(out.contains("x += y;"), "{out}");
    assert!(out.contains("x *= 2;"), "{out}");
    // `??=` 在 x 非空时不会调用 setter，`x = x ?? y` 会；不能合并。
    assert!(out.contains("x = x ?? y;"), "{out}");
    assert!(!out.contains("x ??= y;"), "{out}");
    assert!(out.contains("if (a && b) {"), "{out}");
    assert!(out.contains("if (!a) {"), "{out}");
    assert!(out.contains("void e(bool a) {\n  if (a) {\n    print(3);\n  }\n}"), "{out}");
    // 内层 if 带 else 时不能合并条件。
    assert!(out.contains("if (a) {\n    if (b) {"), "{out}");
    assert!(out.contains("if (!a) if (a) {"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn try_unwrap_removes_rethrow_and_empty_finally() {
    let out = cured(
        "void a() { try { print(1); } catch (e) { rethrow; } }\n\
         void b() { try { print(2); } finally { } }\n\
         void c() { try { print(3); } catch (e) { rethrow; } finally { print(4); } }\n\
         void d() { try { print(5); } catch (e) { print(e); } }\n\
         void e() { try { print(6); } on Exception catch (e) { rethrow; } finally { } }\n\
         void f() { try { print(7); } catch (e) { rethrow; } catch (e) { rethrow; } }\n\
         void g() { return; }\n",
    );
    assert!(!out.contains("catch (e) {\n    rethrow;"), "{out}");
    assert_eq!(out.matches("finally").count(), 1, "{out}");
    assert!(out.contains("print(1);"), "{out}");
    assert!(out.contains("print(2);"), "{out}");
    assert!(out.contains("print(3);"), "{out}");
    assert!(out.contains("print(4);"), "{out}");
    assert!(out.contains("print(5);"), "{out}");
    assert!(out.contains("print(e);"), "{out}");
    assert!(out.contains("print(6);"), "{out}");
    assert!(out.contains("print(7);"), "{out}");
    assert!(!out.contains("void g() {\n  return;\n}"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn literal_member_calls_fold() {
    let out = cured(
        "void main() {\n\
           print('abc'.length);\n\
           print('abc'.isEmpty);\n\
           print('abc'.isNotEmpty);\n\
           print('abc'.codeUnitAt(1));\n\
           print('abc'.contains('bc'));\n\
           print('abc'.startsWith('ab'));\n\
           print('abc'.endsWith('bc'));\n\
           print('abc'.indexOf('b'));\n\
           print('a😀'.length);\n\
           print(255.toRadixString(16));\n\
           print((-7).abs());\n\
           print((-3).isNegative);\n\
           print((4).isEven);\n\
           print((5).isOdd);\n\
           print((8).bitLength);\n\
           print((3).sign);\n\
           print((42).toString());\n\
           print(true.toString());\n\
         }\n",
    );
    for needle in [
        "print(3);",
        "print(false);",
        "print(true);",
        "print(98);",
        "print(1);",
        "print(3);",
        "print('ff');",
        "print(7);",
        "print(4);",
        "print('42');",
        "print('true');",
    ] {
        assert!(out.contains(needle), "missing {needle:?}\n{out}");
    }
    assert_idempotent(&out);
}

#[test]
fn edge_semantics_runtime_differential_if_available() {
    if Command::new("dart").arg("--version").output().is_err() {
        return;
    }
    let src = r#"dynamic dyn = 42;
String dyn_bool() {
  var out = '';
  try { var a = dyn ? true : false; out = out + 'a'; } catch (e) { out = out + 'A'; }
  try { var b = dyn ? false : true; out = out + 'b'; } catch (e) { out = out + 'B'; }
  try { var c = !!dyn; out = out + 'c'; } catch (e) { out = out + 'C'; }
  try { var d = !dyn ? false : true; out = out + 'd'; } catch (e) { out = out + 'D'; }
  try { var e = true && dyn; out = out + 'e'; } catch (e) { out = out + 'E'; }
  try { var f = false || dyn; out = out + 'f'; } catch (e) { out = out + 'F'; }
  try { var g = dyn && true; out = out + 'g'; } catch (e) { out = out + 'G'; }
  try { var h = dyn || false; out = out + 'h'; } catch (e) { out = out + 'H'; }
  var i = false && dyn;
  var j = true || dyn;
  out = out + (i ? '1' : '0') + (j ? '1' : '0');
  return out;
}
String dangling(bool a, bool b) {
  var s = '';
  if (a) { } else if (b) { s = 'b'; } else { s = 'c'; }
  if (a) { s = s + 'A'; } else { s = s + 'E'; }
  return s;
}
String try_finally(int c) {
  var log = '';
  for (var i = 0; i < 3; i++) {
    try {
      if (i == c) { continue; }
      log = log + 'i$i';
    } catch (e) {
      rethrow;
    } finally {
      log = log + 'f$i';
    }
  }
  try {
    return log + 'r';
  } finally {
    log = log + 'x';
  }
}
String catch_loop(int c) {
  var out = '';
  for (var i = 0; i < 3; i++) {
    try {
      if (i == c) { throw 'E$i'; }
      out = out + 'i$i';
    } catch (e) {
      out = out + 'c$i';
    } finally {
      out = out + 'f$i';
    }
  }
  return out;
}
String nested(int c) {
  var out = '';
  try {
    try {
      if (c == 0) { throw 'X'; }
      out = out + 'a';
    } catch (e) {
      rethrow;
    } finally {
      out = out + 'f';
    }
    out = out + 'b';
  } catch (e) {
    out = out + 'C$e';
  }
  return out;
}
String compound(int a, int b) {
  var x = a;
  x = x + b;
  x = x - 3;
  x = x * 2;
  x = x % 7;
  x = x ~/ 2;
  var s = 'q';
  s = s + 'r';
  return '$x,$s';
}
String bang(int? v) {
  var a = v!;
  var b = (v as int?)!;
  var c = [v][0]!;
  return '$a,$b,$c';
}
String spread(int c) {
  var m = {'k': c > 0 ? 'p' : 'n', 't': true ? 'a' : 'b'};
  var l = [c > 0 ? 1 : 2, true ? 3 : 4];
  return '${m['k']},${m['t']},${l[0]},${l[1]}';
}
String patterns(Object v) {
  var s = switch (v) {
    true && true => 'tt',
    true || false => 'tf',
    int x when x > 5 => 'big',
    int x => 'i',
    _ => 'z',
  };
  if (v case true && true) { return 'a:$s'; }
  return 'b:$s';
}
String map_pattern(Object v) {
  if (v case {'a': true && true}) { return 'ma'; }
  if (v case {'a': int x}) { return 'mi$x'; }
  return 'mz';
}
String guard(int v) {
  if (v case int x when true && x > 2) { return 'g$x'; }
  return 'n';
}
int promote(int? x) {
  if (x != null) { if (x > 0) { return x; } }
  if (x == null) { } else { return x + 1; }
  return 0;
}
int ispromote(Object x) {
  if (x is! int) { } else { return x + 1; }
  return 0;
}
bool bit1() => 1 & 2 == 2;
bool bit2() => 0x60 & 0x60 == 0x60;
int bit3() => 1 | 2 & 3;
int bit4() => 8 >> 1 & 3;
void main() {
  print(dyn_bool());
  print(dangling(true, true) + '|' + dangling(true, false) + '|' + dangling(false, true) + '|' + dangling(false, false));
  for (var c = -1; c < 4; c++) { print(try_finally(c)); }
  for (var c = -1; c < 4; c++) { print(catch_loop(c)); }
  print(nested(0));
  print(nested(1));
  print(compound(5, 7));
  print(compound(-5, -7));
  print(bang(5));
  print(spread(1));
  print(spread(-1));
  print(patterns(true));
  print(patterns(false));
  print(patterns(3));
  print(patterns(9));
  print(patterns('x'));
  print(map_pattern({'a': true}));
  print(map_pattern({'a': 3}));
  print(map_pattern({'a': false}));
  print(guard(3));
  print(guard(1));
  print(promote(null));
  print(promote(2));
  print(promote(-2));
  print(ispromote('x'));
  print(ispromote(4));
  print(bit1());
  print(bit2());
  print(bit3());
  print(bit4());
}
"#;
    let cured = cured(src);
    let a = run_dart(src);
    let b = run_dart(&cured);
    assert_eq!(a, b, "runtime mismatch; cured:\n{cured}");
}

#[test]
fn new_rules_runtime_differential_if_available() {
    if Command::new("dart").arg("--version").output().is_err() {
        return;
    }
    let src = r#"bool t1(bool c) { return c ? true : false; }
bool t2(bool c) { return c ? false : true; }
bool t3(bool x) { return !x ? false : true; }
bool t4(bool x) { return true && x; }
bool t5(bool x) { return false && x; }
bool t6(bool x) { return x && true; }
bool t7(Object? x) { return !(x == null); }
bool t8(Object? x) { return !(x is String); }
bool t9(bool x) { return !!x; }
int t10(int x, int y) { x = x + y; x = x * 2; return x; }
int t11(int x) { return true ? x : -x; }
void t12(bool c) { if (c) { } else { print('else'); } }
void t13(bool a, bool b) { if (a) { if (b) { print('both'); } } }
void t14() { try { print('try'); } catch (e) { rethrow; } }
void t15() { try { print('f'); } finally { } }
void t16() { try { print('g'); } catch (e) { rethrow; } finally { print('fin'); } }
String t17() { try { throw 'boom'; } catch (e) { rethrow; } }
String t18() { try { return 'ret'; } finally { } }
String t19() { return 'abc'.length.toString() + ':' + 'abc'.indexOf('b').toString() + ':' + (255).toRadixString(16); }
int t20(int x) { final y = x + 1; return y; }
int t21(int x) { var y = 42; return y; }
List<int> t22(int x) { final y = [x]; return y; }
int t23(int x) { final y = x; return y + 1; }
int t24(int x) { final y = x; int inner() => y; return inner(); }
int t25(int x) { final y = x; print(y); return y; }
void main() {
  print(t1(true)); print(t1(false));
  print(t2(true)); print(t2(false));
  print(t3(true)); print(t3(false));
  print(t4(true)); print(t4(false));
  print(t5(true)); print(t5(false));
  print(t6(true)); print(t6(false));
  print(t7(null)); print(t7('x'));
  print(t8('x')); print(t8(1));
  print(t9(true)); print(t9(false));
  print(t10(2, 3)); print(t11(9));
  t12(true); t12(false);
  t13(true, true); t13(true, false); t13(false, true);
  t14(); t15(); t16();
  try { t17(); } catch (e) { print(e); }
  print(t18());
  print(t19());
  print(t20(1)); print(t21(2)); print(t22(3)); print(t23(4)); print(t24(5)); print(t25(6));
  t14();
}
"#;
    let cured = cured(src);
    let a = run_dart(src);
    let b = run_dart(&cured);
    assert_eq!(a, b, "runtime mismatch; cured:\n{cured}");
}

#[test]
fn dart313_primary_constructors_and_dot_shorthands() {
    let src = "class Point(var int x, var int y);\n\
               class User({required var String _name});\n\
               class Classic {\n\
                 double x, y;\n\
                 new(this.x, this.y);\n\
                 new origin() : x = 0, y = 0;\n\
                 factory clone(Classic other) => Classic(other.x, other.y);\n\
               }\n\
               enum Status { none, running }\n\
               void main() {\n\
                 var p = Point(1, 2);\n\
                 Status s = .running;\n\
                 Point q = .new(3, 4);\n\
                 print('$p $s $q');\n\
               }\n";
    let out = cured(src);
    assert!(out.contains("class Point(var int x, var int y);"), "{out}");
    assert!(out.contains("class User({required var String _name});"), "{out}");
    assert!(out.contains("new(this.x, this.y);"), "{out}");
    assert!(out.contains("factory clone"), "{out}");
    assert!(out.contains(".running"), "{out}");
    assert!(out.contains(".new(3, 4)"), "{out}");
    assert_idempotent(&out);
}

fn legacy_bin(env: &str, default: &str) -> Option<PathBuf> {
    let p = std::env::var(env).map(PathBuf::from).ok().unwrap_or_else(|| PathBuf::from(default));
    p.exists().then_some(p)
}

fn run_bin(bin: &PathBuf, args: &[&str]) -> std::process::Output {
    Command::new(bin)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("run {} failed: {e}", bin.display()))
}

#[test]
fn dart1_legacy_syntax_and_runtime_if_available() {
    let Some(dart) = legacy_bin("DART1_BIN", "/tmp/dartsdk/dart1/dart-sdk/bin/dart") else {
        return;
    };
    let dartfmt = dart.parent().unwrap().join("dartfmt");
    let src = "library legacy;\n\
               typedef int Comparison(int a, int b);\n\
               class Base {\n\
                 final int value;\n\
                 const Base(this.value);\n\
                 factory Base.none() = Base._;\n\
                 Base._() : value = 0;\n\
               }\n\
               class Impl extends Base {\n\
                 Impl() : super(1);\n\
                 int add(final int a, [final int b = 2]) => a + b;\n\
               }\n\
               bool b1(bool c) { return c ? true : false; }\n\
               bool b2(bool c) { return !c ? false : true; }\n\
               bool b3(bool c) { return true && c; }\n\
               bool b4(bool c) { return c && true; }\n\
               bool b5(Object x) { return !(x is String); }\n\
               bool b6(bool x) { return !!x; }\n\
               int b7(int x, int y) { x = x + y; return x; }\n\
               void b8(bool a) { if (a) { } else { print('neg'); } }\n\
               void b9(bool a, bool b) { if (a) { if (b) { print('both'); } } }\n\
               void b10() { try { print('t2'); } catch (e) { rethrow; } }\n\
               void b11() { try { print('u'); } finally { } }\n\
               void b12() { return; }\n\
               int b13() { return 'abc'.length + 'ab'.indexOf('b'); }\n\
               void main() {\n\
                 var x = new Impl();\n\
                 print(x.add(1));\n\
                 print(1 + 2 * 3);\n\
                 print(-5 % 3);\n\
                 print('a' + 'b');\n\
                 if (true) { print(x.value); } else { print(0); }\n\
                 while (x.value > 0) { print('once'); break; }\n\
                 try { print('t'); } catch (e) { print(e); } finally { }\n\
                 print(b1(true)); print(b1(false));\n\
                 print(b2(true)); print(b2(false));\n\
                 print(b3(true)); print(b4(true));\n\
                 print(b5('x')); print(b5(1));\n\
                 print(b6(true));\n\
                 print(b7(2, 3));\n\
                 b8(true); b8(false);\n\
                 b9(true, true); b9(true, false);\n\
                 b10(); b11(); b12();\n\
                 print(b13());\n\
               }\n";
    let cured = cured(src);
    let path = temp_path("dart1_legacy", "dart");
    std::fs::write(&path, &cured).unwrap();
    let fmt = run_bin(&dartfmt, &["-n", path.to_str().unwrap()]);
    let orig_path = temp_path("dart1_legacy_orig", "dart");
    std::fs::write(&orig_path, src).unwrap();
    let orig_fmt = run_bin(&dartfmt, &["-n", orig_path.to_str().unwrap()]);
    assert!(orig_fmt.status.success(), "original Dart 1 formatter failed:\n{src}");
    assert!(
        fmt.status.success(),
        "Dart 1 formatter rejected cured output:\n{cured}\n{}",
        String::from_utf8_lossy(&fmt.stderr)
    );
    let a = run_bin(&dart, &[orig_path.to_str().unwrap()]);
    let b = run_bin(&dart, &[path.to_str().unwrap()]);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&orig_path);
    assert_eq!(a.stdout, b.stdout, "Dart 1 runtime mismatch:\n{cured}");
}

#[test]
fn dart2_modern_syntax_and_runtime_if_available() {
    let Some(dart) = legacy_bin("DART2_BIN", "/tmp/dartsdk/dart2/dart-sdk/bin/dart") else {
        return;
    };
    let src = "// @dart=2.19\n\
               bool b1(bool c) { return c ? true : false; }\n\
               bool b2(bool c) { return c ? false : true; }\n\
               bool b3(Object? x) { return !(x is String); }\n\
               bool b4(bool x) { return !!x; }\n\
               int b5(int x, int y) { x = x + y; return x; }\n\
               int b6(int? x, int y) { x ??= y; return x!; }\n\
               void b7(bool a) { if (a) { } else { print('neg2'); } }\n\
               void b8(bool a, bool b) { if (a) { if (b) { print('both2'); } } }\n\
               void b9() { try { print('t2'); } catch (e) { rethrow; } }\n\
               void b10() { try { print('u2'); } finally { } }\n\
               void main() {\n\
                 var xs = <int?>[1, null, 2];\n\
                 var ys = [for (var x in xs) if (x != null) x * 2];\n\
                 print(ys);\n\
                 print('a' + 'b');\n\
                 print(-5 % 3);\n\
                 print(5 % -3);\n\
                 int f(int c) { if (c > 0) { return 1; } else { return 2; } }\n\
                 print(f(1));\n\
                 var x = new Object();\n\
                 print(x != null);\n\
                 print(b1(true)); print(b1(false));\n\
                 print(b2(true)); print(b2(false));\n\
                 print(b3('x')); print(b3(1));\n\
                 print(b4(true));\n\
                 print(b5(2, 3)); print(b6(null, 5));\n\
                 b7(true); b7(false);\n\
                 b8(true, true); b8(true, false);\n\
                 b9(); b10();\n\
               }\n";
    let cured = cured(src);
    let path = temp_path("dart2_modern", "dart");
    std::fs::write(&path, &cured).unwrap();
    let fmt = run_bin(&dart, &["format", "--output=none", path.to_str().unwrap()]);
    assert!(
        fmt.status.success(),
        "Dart 2 formatter rejected cured output:\n{cured}\n{}",
        String::from_utf8_lossy(&fmt.stderr)
    );
    let orig_path = temp_path("dart2_modern_orig", "dart");
    std::fs::write(&orig_path, src).unwrap();
    let a = run_bin(&dart, &["run", orig_path.to_str().unwrap()]);
    let b = run_bin(&dart, &["run", path.to_str().unwrap()]);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&orig_path);
    assert_eq!(a.stdout, b.stdout, "Dart 2 runtime mismatch:\n{cured}");
}

#[test]
fn statement_rules_runtime_differential_if_available() {
    if Command::new("dart").arg("--version").output().is_err() {
        return;
    }
    let src = r#"int choose(int c) {
  if (c > 0) { return 1; } else { return 2; }
}
int choose2(int c) {
  if (c > 0) return 3;
  return 4;
}
int assign(int c) {
  var x = 0;
  if (c > 0) { x = 5; } else { x = 6; }
  return x;
}
int loops(int c) {
  var n = 0;
  for (var i = 0; i < c; i++) { n += i; continue; }
  while (c > 0) { n += c; c--; continue; }
  do { n++; continue; } while (false);
  return n;
}
void dead() {
  print('a');
  return;
  print('b');
}
int fin(int c) {
  try { return c; } catch (e) { return -1; } finally { }
}
void main() {
  print(choose(-1));
  print(choose2(0));
  print(assign(1));
  print(loops(3));
  dead();
  print(fin(7));
}
"#;
    let cured = cured(src);
    let a = run_dart(src);
    let b = run_dart(&cured);
    assert_eq!(a, b, "runtime mismatch; cured:\n{cured}");
}

#[test]
fn else_after_terminal_is_removed() {
    let out = cured(
        "int f(int c) {\n\
           if (c > 0) { return 1; } else { print('neg'); return 2; }\n\
         }\n\
         void g(int c) {\n\
           if (c > 0) { print('p'); return; } else { print('n'); }\n\
         }\n\
         int h(int c) {\n\
           if (c > 0) { return 1; } else if (c < 0) { print(-1); } else { print(0); }\n\
         }\n",
    );
    assert!(!out.contains("} else {\n    print('neg')"), "{out}");
    assert!(!out.contains("} else {\n    print('n')"), "{out}");
    assert!(out.contains("if (c > 0) {"), "{out}");
    assert!(out.contains("print('neg');"), "{out}");
    assert!(out.contains("else if (c < 0)"), "{out}");
    assert_idempotent(&out);
}

#[test]
fn version_syntax_fixtures_are_token_stable() {
    let fixtures: &[(&str, &str)] = &[
        ("dart1", include_str!("fixtures/dart1_syntax.dart")),
        ("dart1_part", include_str!("fixtures/dart1_part.dart")),
        ("dart2", include_str!("fixtures/dart2_syntax.dart")),
        ("dart3", include_str!("fixtures/dart3_syntax.dart")),
        ("dart313", include_str!("fixtures/dart313_syntax.dart")),
    ];
    for (name, src) in fixtures {
        let out = parse(src);
        assert!(out.errors.is_empty(), "{name}: {:?}", out.errors);
        let printed = print_module(&out.ast);
        let out2 = parse(&printed);
        assert!(out2.errors.is_empty(), "{name} printed: {:?}", out2.errors);
        assert_eq!(out.ast.tokens.len(), out2.ast.tokens.len(), "{name}");
        for (a, b) in out.ast.tokens.iter().zip(&out2.ast.tokens) {
            assert_eq!((a.kind, &*a.text), (b.kind, &*b.text), "{name}");
        }
        let cured = cured(src);
        assert!(!cured.trim().is_empty(), "{name}");
        assert_idempotent(&cured);
    }
}

#[test]
fn trailing_assign_return_folds_single_use_locals() {
    let out = cured(
        r#"int f(int x) {
  final y = x;
  return y;
}
int g(int x) {
  var y = 42;
  return y;
}
int h(int x) {
  final y = x + 1;
  return y;
}
int i(int x) {
  final y = foo();
  return y;
}
int j(int x) {
  final y = x;
  print(y);
  return y;
}
"#,
    );
    assert!(out.contains("int f(int x) {\n  return x;\n}"), "{out}");
    assert!(out.contains("int g(int x) {\n  return 42;\n}"), "{out}");
    assert!(out.contains("int h(int x) {\n  return x + 1;\n}"), "{out}");
    // 调用结果类型受返回位置上下文影响（泛型推断），保留局部变量。
    assert!(out.contains("final y = foo();"), "{out}");
    assert!(out.contains("print(y);"), "{out}");
    assert_idempotent(r#"int f(int x) {
  final y = x;
  return y;
}
int g(int x) {
  var y = 42;
  return y;
}
"#);
}
