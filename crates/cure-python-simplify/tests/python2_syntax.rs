//! Python 2 语法回归 + 双版本语法边界（不需要 Python 2 解释器也能跑）。
//! 设置 `CURE_PY2=/path/to/python2.7` 可额外执行语义差分。

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use cure_engine::Config;
use cure_python_parser::parse;
use cure_python_print::print_module;
use cure_python_simplify::simplify_module;

fn format_only(src: &str) -> String {
    let out = parse(src);
    assert!(
        out.errors.is_empty(),
        "解析诊断：{:?}\n---\n{src}",
        out.errors
    );
    print_module(&out.ast, out.module)
}

fn cured(src: &str) -> String {
    let mut out = parse(src);
    assert!(
        out.errors.is_empty(),
        "解析诊断：{:?}\n---\n{src}",
        out.errors
    );
    let _ = simplify_module(&mut out.ast, out.module, &Config::default());
    print_module(&out.ast, out.module)
}

fn assert_idempotent(src: &str) -> String {
    let once = cured(src);
    let twice = cured(&once);
    assert_eq!(once, twice, "二次 cure 不幂等");
    once
}

fn python2() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("CURE_PY2") {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Some(p);
        }
    }
    for c in ["python2.7", "python2"] {
        if Command::new(c)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(PathBuf::from(c));
        }
    }
    None
}

fn python3() -> Option<&'static str> {
    for c in ["python3.15", "python3"] {
        if Command::new(c)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(c);
        }
    }
    None
}

#[test]
fn py2_print_statement_forms() {
    let src = "\
print
print x
print x,
print x, y
print >>f
print >>f, x
print >>f, x,
print (x)
print (x), y
print[1, 2]
";
    let out = format_only(src);
    let expect = [
        "print",
        "print x",
        "print x,",
        "print x, y",
        "print >>f",
        "print >>f, x",
        "print >>f, x,",
        "print(x)",
        "print (x), y",
        "print[1, 2]",
    ];
    let got: Vec<&str> = out.lines().collect();
    assert_eq!(got, expect, "输出：\n{out}");
    assert_eq!(out, format_only(&out), "print 语句二次格式化漂移");
}

#[test]
fn py2_long_arithmetic_is_not_folded() {
    let src = "\
x = 1L + 1
y = -1L
z = ~1L
w = 1L << 3
v = 1L and 2L
";
    let out = cured(src);
    // 值相同的折叠会改变 Python 2 的 int/long 类型；必须原样保留。
    for s in ["1L + 1", "-1L", "~1L", "1L << 3", "2L"] {
        assert!(out.contains(s), "missing `{s}`:\n{out}");
    }
    assert_idempotent(src);
}

#[test]
fn py2_shadowed_true_false_are_not_folded() {
    let src = "True = 0\nFalse = 1\nif True:\n    x = 1\nelse:\n    x = 2\n";
    let out = cured(src);
    // 分支可以合并成条件表达式，但 True 不能被当成常量折叠。
    assert!(
        out.contains("1 if True else 2"),
        "被遮蔽的 True 被折成常量：\n{out}"
    );
    assert!(!out.contains("x = 1\n"), "{out}");
    assert_idempotent(src);

    // 形参/for 目标/lambda 参数同样是绑定。
    let out = cured("def f(True):\n    if True:\n        return 1\n    return 2\n");
    assert!(out.contains("if True:"), "形参 True 未识别：\n{out}");
    let out = cured("g = lambda True: 1 if True else 2\n");
    assert!(out.contains("if True"), "lambda 形参 True 未识别：\n{out}");

    // 未遮蔽时仍正常折叠；调用里的关键字参数不算绑定。
    let out = cured("f(True, x=1)\nif True:\n    x = 1\nelse:\n    x = 2\n");
    assert_eq!(out, "f(True, x=1)\nx = 1\n");
}

#[test]
fn print_followed_by_unary_sign_is_raw() {
    let src = "print -1 + 2\nprint +1 + 2\n";
    let out = cured(src);
    // Python 3 下这两行是合法的二元表达式，折叠会变成语法错误。
    assert_eq!(out, src);
    assert_eq!(out, cured(&out));
}

#[test]
fn encoding_cookie_after_blank_first_line_is_preserved() {
    let src = "\n# -*- coding: utf-8 -*-\nprint \"x\"\n";
    let out = cured(src);
    assert!(out.starts_with("\n# -*- coding: utf-8 -*-\n"), "{out}");
}

#[test]
fn py2_literals_and_operators() {
    let src = "\
a = 0777 + 255 + 10
b = `(1, 'a')`
c = 3 <> 4
d = ur'raw\\n'
e = 0xFFL
";
    let out = cured(src);
    assert!(out.contains("a = 776"), "{out}");
    assert!(out.contains("b = `(1, 'a')`"), "{out}");
    assert!(out.contains("c = True"), "{out}");
    assert!(out.contains("d = ur'raw\\n'"), "{out}");
    assert!(out.contains("e = 0xFFL"), "{out}");
    assert_idempotent(src);
}

#[test]
fn py2_exec_raise_except_are_preserved() {
    let src = "\
try:
    exec \"x = 1\" in g, l
    raise ValueError, \"bad\", None
except (ValueError, TypeError), e:
    pass
";
    let out = format_only(src);
    assert!(out.contains("exec \"x = 1\" in g, l"), "{out}");
    assert!(out.contains("raise ValueError, \"bad\", None"), "{out}");
    assert!(out.contains("except (ValueError, TypeError), e:"), "{out}");
    assert_eq!(out, format_only(&out));
}

#[test]
fn py2_tuple_params_and_comprehension_iterables() {
    let src = "\
def f((a, b), c=1L):
    return a + b + c
g = [x for x in a, b]
h = [x for x in lambda: True, lambda: False if x()]
i = lambda (m, n), k=2: m + n + k
";
    let out = format_only(src);
    assert!(out.contains("def f((a, b), c=1L):"), "{out}");
    assert!(out.contains("[x for x in (a, b)]"), "{out}");
    assert!(out.contains("if x()"), "{out}");
    assert!(out.contains("lambda: True"), "{out}");
    assert!(out.contains("lambda (m, n), k=2:"), "{out}");
    assert_eq!(out, format_only(&out));
}

#[test]
fn py3_annotations_may_contain_equals_in_brackets() {
    let src = "\
class C:
    enum: Annotated[list[str] | None, Field(min_length=1)] = None
    cb: Callable[[int], int] = lambda y=1: y
";
    let out = format_only(src);
    assert!(out.contains("Field(min_length=1)"), "{out}");
    assert!(out.contains("lambda y=1: y"), "{out}");
    assert_eq!(out, format_only(&out));
}

#[test]
fn py3_fstring_braces_after_backslash() {
    let src = "\
a = rf'({name})(\\{{)'
b = f\"{x:#x}\"
c = rf'\\{{x}}'
";
    let out = format_only(src);
    assert!(out.contains(r"rf'({name})(\{{)'"), "{out}");
    assert!(out.contains(r#"f"{x:#x}""#), "{out}");
    assert_eq!(out, format_only(&out));
}

#[test]
fn decorator_multiline_string_content_is_preserved() {
    let src = "class C:\n  @decorate(\n    \"\"\"\n    line\n\n    trailing  \n    \"\"\"\n  )\n  def f(self):\n    pass\n";
    let out = format_only(src);
    assert!(
        out.contains("    line\n\n    trailing  \n"),
        "字符串内容被改写：\n{out}"
    );
    // 代码缩进按打印器 4 空格重排，装饰器仍合法。
    assert!(out.contains("@decorate("), "{out}");
    assert_eq!(out, format_only(&out), "装饰器二次格式化漂移");
}

#[test]
fn raw_statement_multiline_string_content_is_preserved() {
    let src = "class C:\n  def f(self):\n    raise ValueError, \"\"\"msg\n  next\n\"\"\"\n";
    let out = format_only(src);
    assert!(out.contains("\"\"\"msg\n  next\n\"\"\""), "原始字符串被改写：\n{out}");
    assert_eq!(out, format_only(&out), "raw 语句二次格式化漂移");
}

#[test]
fn slice_trailing_colon_and_subscript_tuples() {
    let src = "\
a = b[0:2:]
c = d[..., ...]
e = f[1, 2]
g = h[()]
i = j[(1, 2), 3]
";
    let out = format_only(src);
    assert!(out.contains("b[0:2:]"), "{out}");
    assert!(out.contains("d[..., ...]"), "{out}");
    assert!(out.contains("f[1, 2]"), "{out}");
    assert!(out.contains("h[()]"), "{out}");
    assert!(out.contains("j[(1, 2), 3]"), "{out}");
    assert_eq!(out, format_only(&out));
}

#[test]
fn exponent_tokenization_stops_before_non_digits() {
    let src = "x = 1 if 1else 0\ny = 1 if 0else 0\n";
    let out = format_only(src);
    assert!(out.contains("1 if 1 else 0"), "{out}");
    assert!(out.contains("1 if 0 else 0"), "{out}");
    assert_idempotent(src);
}

#[test]
fn shebang_and_encoding_cookie_are_preserved() {
    let src = "#!/usr/bin/env python\n# -*- coding: utf-8 -*-\nprint \"x\"\n";
    let out = cured(src);
    assert!(out.starts_with("#!/usr/bin/env python\n# -*- coding: utf-8 -*-\n"), "{out}");
    assert_eq!(out, cured(&out), "头部二次格式化漂移");
}

#[test]
fn utf8_bom_is_stripped() {
    let src = "\u{feff}x = 1\n";
    let out = cured(src);
    assert_eq!(out, "x = 1\n");
}

fn run_py(py: &PathBuf, path: &std::path::Path) -> (String, String, Option<i32>) {
    let out = Command::new(py).arg(path).output().expect("run python");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.code(),
    )
}

#[test]
fn py2_semantics_differential_if_available() {
    let Some(py) = python2() else {
        eprintln!("skip python2 differential: set CURE_PY2 to a Python 2.7 binary");
        return;
    };
    let src = r#"# -*- coding: utf-8 -*-
import sys
print "a", 1 + 2
print -1 + 2
print >>sys.stdout, "b", 2 + 3
print "c",
print "d"
x = `(1, 'z')`
print x
print 0777, 0xFFL, 10L
print 1L + 1, -1L, ~1L
print type(1L + 1).__name__, type(-1L).__name__, type(~1L).__name__
print 3 <> 4
def f((a, b), c=1L):
    return a + b + c
print f((2, 3))
print (lambda (m, n): m * n)((4, 5))
try:
    raise ValueError, "boom"
except ValueError, e:
    print "caught", e
exec "q = 6 * 7" in globals()
print q
print [i for i in (1, 2)]
print (i for i in (1, 2)).next()
class C:
  @staticmethod
  def f():
    return """
    a

    b
    """
print repr(C.f())
"#;
    let cur = cured(src);
    let base = std::env::temp_dir().join(format!("cure_py2_sem_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let a = base.join("orig.py");
    let b = base.join("cured.py");
    fs::write(&a, src).unwrap();
    fs::write(&b, &cur).unwrap();
    let (o1, e1, c1) = run_py(&py, &a);
    let (o2, e2, c2) = run_py(&py, &b);
    assert_eq!(o1, o2, "stdout 不一致；cured:\n{cur}\nstderr:\n{e2}");
    assert_eq!(c1, c2, "退出码不一致；stderr:\n{e1}\n---\n{e2}");
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn py3_decorator_and_annotation_semantics() {
    let Some(py) = python3() else {
        eprintln!("skip python3 differential: python3 missing");
        return;
    };
    let src = r#"class C:
  @staticmethod
  def f():
    return """
    a

    b
    """
x: "dict[str, int]" = {}
y: int = 1
print(repr(C.f()))
print(x, y)
"#;
    let cur = cured(src);
    let base = std::env::temp_dir().join(format!("cure_py3_dec_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let a = base.join("orig.py");
    let b = base.join("cured.py");
    fs::write(&a, src).unwrap();
    fs::write(&b, &cur).unwrap();
    let (o1, e1, c1) = run_py(&PathBuf::from(py), &a);
    let (o2, e2, c2) = run_py(&PathBuf::from(py), &b);
    assert_eq!(o1, o2, "stdout 不一致；cured:\n{cur}\nstderr:\n{e2}");
    assert_eq!(c1, c2, "退出码不一致；stderr:\n{e1}\n---\n{e2}");
    let _ = fs::remove_dir_all(&base);
}
#[test]
fn based_numbers_stop_at_the_right_alphabet() {
    let src = "a = 0b1and 1\nb = 0o17or 0\nc = 0xFFor 0\n";
    let out = cured(src);
    assert!(out.contains("a = 1"), "{out}");
    assert!(out.contains("b = 0o17"), "{out}");
    assert!(out.contains("c = 0xFF"), "{out}");
    assert!(!out.contains("0b1and") && !out.contains("0o17or"), "{out}");
}

#[test]
fn py2_true_false_shadowing_differential_if_available() {
    let Some(py) = python2() else {
        eprintln!("skip python2 shadow differential: set CURE_PY2");
        return;
    };
    let src = r#"# -*- coding: utf-8 -*-
True = 0
False = 1
print True, False
if True:
    print "bad"
else:
    print "good"
def f(True):
    if True:
        return 1
    return 2
print f(0)
for x, True in [(3, 4)]:
    print True
"#;
    let cur = cured(src);
    let base = std::env::temp_dir().join(format!("cure_py2_shadow_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let a = base.join("orig.py");
    let b = base.join("cured.py");
    fs::write(&a, src).unwrap();
    fs::write(&b, &cur).unwrap();
    let (o1, e1, c1) = run_py(&py, &a);
    let (o2, e2, c2) = run_py(&py, &b);
    assert_eq!(o1, o2, "stdout 不一致；cured:\n{cur}\nstderr:\n{e2}");
    assert_eq!(c1, c2, "退出码不一致；stderr:\n{e1}\n---\n{e2}");
    let _ = fs::remove_dir_all(&base);
}
