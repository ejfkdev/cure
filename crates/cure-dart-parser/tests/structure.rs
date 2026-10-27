use cure_dart_ast::{BraceKind, TokKind};
use cure_dart_parser::parse;

#[test]
fn distinguishes_blocks_collections_and_switch() {
    let src = r#"
class A {
  void f(int x) {
    if (x > 0) { print(x); }
    var m = {'a': 1, 'b': 2};
    var s = {1, 2, 3};
    switch (x) { case 1: break; default: break; }
    var g = () { return 1; };
    var r = (x, name: 'n');
  }
}
"#;
    let out = parse(src);
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    let kinds: Vec<_> = out
        .ast
        .tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| t.text.as_ref() == "{")
        .map(|(i, _)| out.ast.brace_kind(i).unwrap())
        .collect();
    assert_eq!(
        kinds,
        vec![
            BraceKind::Block,       // class
            BraceKind::Block,       // method
            BraceKind::Block,       // if
            BraceKind::Collection,  // map
            BraceKind::Collection,  // set
            BraceKind::SwitchBlock, // switch
            BraceKind::Block,       // closure
        ]
    );
}

#[test]
fn pairs_and_metrics() {
    let out = parse("void main() { if (true && false) { print('x'); } else { print('y'); } }");
    assert!(out.errors.is_empty());
    let mut opens = 0;
    for (i, t) in out.ast.tokens.iter().enumerate() {
        if t.text.as_ref() == "(" || t.text.as_ref() == "{" {
            opens += 1;
            assert!(out.ast.matching(i).is_some());
        }
    }
    assert!(opens >= 4);
    let (nodes, decisions, depth) = cure_dart_ast::unit_metrics(&out.ast);
    assert!(nodes > 10);
    assert!(decisions >= 2);
    assert!(depth >= 2);
    assert!(out
        .ast
        .tokens
        .iter()
        .any(|t| t.kind == TokKind::Str && t.text.contains("x")));
}

#[test]
fn unmatched_brackets_are_reported() {
    let out = parse("void main() { if (true) { }");
    assert!(!out.errors.is_empty());
    assert!(out.errors.iter().any(|e| e.message.contains("unclosed")));
    let out = parse("void main() { } }");
    assert!(out.errors.iter().any(|e| e.message.contains("unmatched")));
}

#[test]
fn simple_string_domain() {
    use cure_dart_ast::{make_simple_string, simple_string};
    assert_eq!(simple_string("'abc'").as_deref(), Some("abc"));
    assert_eq!(simple_string(r#""a b""#).as_deref(), Some("a b"));
    assert_eq!(simple_string("'a\\n'"), None);
    assert_eq!(simple_string("'$x'"), None);
    assert_eq!(simple_string("'''x'''"), None);
    assert_eq!(make_simple_string("ab").as_deref(), Some("'ab'"));
    assert_eq!(make_simple_string("a'b"), None);
}