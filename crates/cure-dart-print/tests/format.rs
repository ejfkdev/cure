use cure_dart_ast::{DartAst, TokKind};
use cure_dart_parser::parse;
use cure_dart_print::print_module;

fn fmt(src: &str) -> String {
    let out = parse(src);
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    print_module(&out.ast)
}

fn tokens(ast: &DartAst) -> Vec<(TokKind, String)> {
    ast.tokens
        .iter()
        .map(|t| (t.kind, t.text.to_string()))
        .collect()
}

#[test]
fn formatting_is_token_preserving() {
    let src = r#"
class A<T> {
  Map<String,List<int>> m = {'a': [1,2]};
  void f(int x) { if (x > 0) { print('a'); } else { print('b'); } }
  void operator []=(String k, Object v) {}
  List<int> g() => [1, 2, 3];
}
"#;
    let out = parse(src);
    assert!(out.errors.is_empty());
    let once = print_module(&out.ast);
    let out2 = parse(&once);
    assert!(out2.errors.is_empty(), "{once}\n{:?}", out2.errors);
    assert_eq!(tokens(&out.ast), tokens(&out2.ast));
    assert_eq!(once, print_module(&out2.ast), "format not idempotent");
}

#[test]
fn comments_and_strings_survive_verbatim() {
    let src = r#"// leading
/* nested /* block */ comment */
void main() {
  var s = 'a$b ${c}';
  var raw = r'${keep}';
  var t = '''multi
  line''';
  print(s); // tail
}
"#;
    let printed = fmt(src);
    for needle in [
        "// leading",
        "/* nested /* block */ comment */",
        "'a$b ${c}'",
        "r'${keep}'",
        "'''multi\n  line'''",
        "// tail",
    ] {
        assert!(printed.contains(needle), "missing {needle:?}\n{printed}");
    }
}

#[test]
fn switch_cases_are_indented() {
    let printed = fmt("void main(){switch(x){case 1:foo();break;default:bar();}}");
    let expected = "void main() {\n  switch (x) {\n    case 1:\n      foo();\n      break;\n    default:\n      bar();\n  }\n}";
    assert!(printed.contains(expected), "{printed}");
}

#[test]
fn generic_spacing_is_tight() {
    let printed = fmt("Map<String,List<int>> m = <String,List<int>>{};");
    assert!(printed.contains("Map<String, List<int>>"), "{printed}");
    assert!(printed.contains("<String, List<int>>{}"), "{printed}");
}

#[test]
fn operator_bracket_assign_stays_lexically_joined() {
    let printed = fmt("void operator[]=(String k, Object v) {}");
    assert!(printed.contains("operator[]=("), "{printed}");
    let out = parse(&printed);
    assert!(out.errors.is_empty());
    assert!(out.ast.tokens.windows(3).any(|w| {
        w[0].text.as_ref() == "[" && w[1].text.as_ref() == "]" && w[2].text.as_ref() == "="
    }));
}

#[test]
fn nullable_type_and_cascade_do_not_merge() {
    let printed = fmt("void f(){ GlobalKey<S>?\n ..x = 1; }");
    assert!(printed.contains("? ..x"), "{printed}");
    let out = parse(&printed);
    assert!(out
        .ast
        .tokens
        .windows(2)
        .any(|w| w[0].text.as_ref() == "?" && w[1].text.as_ref() == ".."));
    assert!(!out.ast.tokens.iter().any(|t| t.text.as_ref() == "?.."));
}

#[test]
fn annotation_newline_before_block_comment() {
    let printed = fmt("@Foo(1)\n/* member */\nvoid f() {}");
    assert!(printed.contains("@Foo(1)\n/* member */"), "{printed}");
    assert_eq!(printed, fmt(&printed));
}

#[test]
fn async_and_sync_bodies_are_blocks() {
    let printed = fmt("class A{ Future<void> f() async { await g(); } Stream<int> s() async* { yield 1; } }");
    assert!(printed.contains("async {\n"), "{printed}");
    assert!(printed.contains("async* {"), "{printed}");
    assert_eq!(printed, fmt(&printed));
}

#[test]
fn trailing_ignore_comments_stay_on_statement_line() {
    let printed = fmt(
        "void main(){foo(); // ignore: unnecessary_statements\nbar(); test('a',(){baz(); // ignore: unnecessary_statements\nqux();});}",
    );
    assert!(
        printed
            .lines()
            .any(|l| l.trim_start().starts_with("foo(); // ignore:")),
        "{printed}"
    );
    assert!(
        printed
            .lines()
            .any(|l| l.trim_start().starts_with("baz(); // ignore:")),
        "{printed}"
    );
    assert_eq!(printed, fmt(&printed));
}

#[test]
fn standalone_ignore_comments_are_not_absorbed() {
    let printed = fmt(
        "void main(){test('a',(){// ignore: unnecessary_statements\nx;\n// ignore: unnecessary_statements\ny;\n});}",
    );
    let comments: Vec<&str> = printed
        .lines()
        .filter(|l| l.trim_start().starts_with("// ignore:"))
        .collect();
    assert_eq!(comments.len(), 2, "{printed}");
    assert!(
        !printed.lines().any(|l| l.contains("; // ignore:")),
        "ignore comment was absorbed by a statement:\n{printed}"
    );
    // 闭包（作为调用实参）里的语句也要逐行输出。
    assert!(
        !printed.lines().any(|l| l.contains("x;") && l.contains("y;")),
        "{printed}"
    );
    assert_eq!(printed, fmt(&printed));
}

#[test]
fn for_header_semicolons_stay_inline() {
    let printed = fmt("void main(){for(var i=0;i<2;i++){print(i);}}");
    assert!(printed.contains("for (var i = 0; i < 2; i++)"), "{printed}");
    assert_eq!(printed, fmt(&printed));
}

#[test]
fn statements_in_closure_arguments_break_lines() {
    let printed = fmt("void main(){test('a',(){var x=1;var y=2;print(x+y);});}");
    assert!(
        !printed
            .lines()
            .any(|l| l.contains("var x = 1;") && l.contains("var y = 2;")),
        "{printed}"
    );
    assert_eq!(printed, fmt(&printed));
}
#[test]
fn record_return_type_starts_on_new_line_after_class() {
    // `class A {}` 后接记录返回类型 `(int, String) f()`：`}` 与 `(` 不能粘。
    let printed = fmt("class A {}\n(int, String) f() => (1, 'x');\n");
    assert!(printed.contains("}\n(int, String) f()"), "{printed}");
    assert_eq!(printed, fmt(&printed));
}

#[test]
fn keyword_before_paren_and_bracket_gets_space() {
    let printed = fmt(
        "void g(Object o, List<int?> xs) {\n  for (final (i, v) in xs.indexed) { print(i); }\n  if (xs case [_, ...]) { print(o); }\n  var r = switch (o) { 1 => 'a', _ => 'b' };\n}\n",
    );
    assert!(printed.contains("final (i, v)"), "{printed}");
    assert!(printed.contains("case [_, ...]"), "{printed}");
    // switch 表达式每个分支独占一行
    assert!(printed.contains("1 => 'a',\n"), "{printed}");
    assert_eq!(printed, fmt(&printed));
}

#[test]
fn yield_star_and_pattern_shorthand_are_tight() {
    let printed = fmt(
        "Iterable<int> h() sync* { yield 1; yield* [2, 3]; }\nString p(Object o) => switch (o) {\n  Box<int>(:final value) => '$value',\n  _ => '',\n};\n",
    );
    assert!(printed.contains("yield* [2, 3]"), "{printed}");
    assert!(printed.contains("(:final value)"), "{printed}");
    assert_eq!(printed, fmt(&printed));
}
