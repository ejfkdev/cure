use cure_dart_ast::TokKind;
use cure_dart_parser::lexer::tokenize;

fn texts(src: &str) -> Vec<String> {
    let (toks, errs) = tokenize(src);
    assert!(errs.is_empty(), "{errs:?}");
    toks.into_iter().map(|t| t.text.to_string()).collect()
}

#[test]
fn lexes_dart_versions_forms() {
    let src = r#"#!/usr/bin/env dart
library sample;
part 'x.dart';
import 'dart:io' as io show File;
@deprecated
class A<T extends Object> with M implements I {
  final int x;
  A(this.x);
  factory A.zero() => A(0);
  external void ext();
  int get value => x;
  set value(int v) {}
  void operator []=(String k, Object v) {}
  Future<void> run() async* {
    await Future<void>.value();
    yield* Stream<int>.empty();
  }
}
enum E { a, b }
extension type ET(int i) { int get v => i; }
typedef Old = int Function(int);
typedef int Legacy(int x);
"#;
    let t = texts(src);
    for expected in [
        "class", "factory", "operator", "extension", "type", "enum", "typedef", "Legacy",
        "async", "yield", "await", "external",
    ] {
        assert!(
            t.iter().any(|x| x == expected),
            "missing {expected:?} in {t:?}"
        );
    }
}

#[test]
fn lexes_all_operator_forms() {
    let t = texts("a >>>= b; c >>= d; e <<= f; g ~/= h; i ??= j; k ... l; m ...? n; o ?..p; q >> r; s >>> t;");
    for expected in [">>>=", ">>=", "<<=", "~/=", "??=", "...?", "?..", ">>", ">>>", "..."] {
        assert!(t.iter().any(|x| x == expected), "missing {expected:?}");
    }
}

#[test]
fn lexes_numbers_and_strings() {
    let t = texts(
        r#"var a = 0x1F + 0b101 + 1_000 + 1.5e-3 + .5;
var b = 'plain';
var c = "double";
var d = r'${raw}';
var e = '''multi
line''';
var f = '${'--$x\r\n'}';
var g = '$_body${'inner'}';
"#,
    );
    for expected in ["0x1F", "0b101", "1_000", "1.5e-3", ".5"] {
        assert!(t.iter().any(|x| x == expected), "missing {expected:?}");
    }
    assert!(t.iter().any(|x| x == "r'${raw}'"));
    assert!(t.iter().any(|x| x.contains("multi")));
    assert!(t.iter().any(|x| x == "'${'--$x\\r\\n'}'"));
    assert!(t.iter().any(|x| x == "'$_body${'inner'}'"));
}

#[test]
fn nested_block_comments_and_blank_lines() {
    let (toks, errs) = tokenize("/* a /* b */ c */\n\nvoid main() {}\n");
    assert!(errs.is_empty(), "{errs:?}");
    assert_eq!(toks[0].kind, TokKind::BlockComment);
    assert_eq!(toks[0].text.as_ref(), "/* a /* b */ c */");
    assert!(toks[1].newlines_before >= 2);
}

#[test]
fn unterminated_literals_are_reported_not_fatal() {
    let (toks, errs) = tokenize("void main() { var s = 'oops; }");
    assert!(!toks.is_empty());
    assert!(errs.iter().any(|e| e.message.contains("unterminated")));
    let (_, errs) = tokenize("/* never closed");
    assert!(errs.iter().any(|e| e.message.contains("unterminated block comment")));
}

#[test]
fn bom_and_shebang() {
    let (toks, errs) = tokenize("\u{feff}#!/usr/bin/env dart\nvoid main() {}\n");
    assert!(errs.is_empty(), "{errs:?}");
    assert_eq!(toks[0].kind, TokKind::LineComment);
    assert!(toks[0].text.starts_with("#!"));
    assert_eq!(toks[1].text.as_ref(), "void");
}

#[test]
fn positions_advance() {
    let (toks, _) = tokenize("void main() {\n  int x = 1;\n}\n");
    let int = toks.iter().find(|t| t.text.as_ref() == "int").unwrap();
    assert_eq!(int.line, 2);
    assert_eq!(int.col, 3);
    assert_eq!(int.newlines_before, 1);
}