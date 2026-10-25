//! Python 简化规则回归（字面量/字符串/调用/下标折叠 + 结构规则）。

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use cure_engine::Config;
use cure_python_parser::parse;
use cure_python_print::print_module;
use cure_python_simplify::simplify_module;

fn cured(src: &str) -> String {
    let mut out = parse(src);
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    let _ = simplify_module(&mut out.ast, out.module, &Config::default());
    print_module(&out.ast, out.module)
}

fn assert_idempotent(src: &str) {
    let once = cured(src);
    let twice = cured(&once);
    assert_eq!(once, twice, "二次简化漂移；一次输出：\n{once}");
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

fn run(py: &std::path::Path, path: &std::path::Path) -> (String, String, Option<i32>) {
    let out = Command::new(py).arg(path).output().expect("run python");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.code(),
    )
}

fn differential(py: &std::path::Path, src: &str, tag: &str) {
    let cur = cured(src);
    let base = std::env::temp_dir().join(format!("cure_pysimp_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let a = base.join("orig.py");
    let b = base.join("cured.py");
    fs::write(&a, src).unwrap();
    fs::write(&b, &cur).unwrap();
    let (o1, e1, c1) = run(py, &a);
    let (o2, e2, c2) = run(py, &b);
    assert_eq!(o1, o2, "stdout 不一致；cured:\n{cur}\nstderr:\n{e2}");
    assert_eq!(c1, c2, "退出码不一致；stderr:\n{e1}\n---\n{e2}");
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn string_and_builtin_folding() {
    let src = r#"x = "a" + "b"
y = "abc".upper()
z = "a-b-a".replace("-", "+")
w = "".join(["a", "b"])
n = len("abcd")
c = chr(65)
o = ord("A")
i = int("42")
s = str(123)
b = bool("")
m = max(1, 5)
v = "abcdef"[1:4]
q = "abcdef"[-1]
t = (1, 2, 3)[1]
sl = [1, 2, 3][1:]
lt = "a" < "b"
"#;
    let out = cured(src);
    let expect = [
        "x = 'ab'",
        "y = 'ABC'",
        "z = 'a+b+a'",
        "w = 'ab'",
        "n = 4",
        "c = 'A'",
        "o = 65",
        "i = 42",
        "s = '123'",
        "b = False",
        "m = 5",
        "v = 'bcd'",
        "q = 'f'",
        "t = 2",
        "sl = [2, 3]",
        "lt = True",
    ];
    let got: Vec<&str> = out.lines().collect();
    assert_eq!(got, expect, "输出：\n{out}");
    assert_idempotent(src);
}

#[test]
fn unsafe_literal_shapes_are_not_folded() {
    let src = "a = \"a\\n\" + \"b\"\nb = \"é\".upper()\nc = b\"ab\".upper()\nd = \"a\" * 1000000\n";
    let out = cured(src);
    assert!(out.contains('+'), "{out}");
    assert!(out.contains(".upper()"), "{out}");
    assert!(out.contains("\"a\\n\"") || out.contains("'a\\n'"), "{out}");
    assert_eq!(out, cured(&out));
}

#[test]
fn structural_simplifications() {
    let src = r#"if cond:
    r = 1
else:
    r = 2
def f():
    if cond:
        return 1
    else:
        return 2
if cond:
    raise ValueError("x")
else:
    r = 3
if a:
    if b:
        r = 4
p = 9
p = 10
def g():
    return None
1 + 2
if c:
    return 1
elif d:
    return 2
else:
    return 3
"#;
    let out = cured(src);
    assert!(out.contains("r = 1 if cond else 2"), "{out}");
    assert!(out.contains("return 1 if cond else 2"), "{out}");
    assert!(out.contains("raise ValueError(\"x\")\nr = 3"), "{out}");
    assert!(out.contains("if a and b:"), "{out}");
    assert!(!out.contains("p = 9"), "{out}");
    assert!(out.contains("p = 10"), "{out}");
    assert!(out.contains("def g():\n    return"), "{out}");
    assert!(!out.contains("1 + 2"), "{out}");
    // elif 链在固定点后可以折成嵌套条件表达式（语义相同）。
    assert!(out.contains("return 1 if c else 2 if d else 3"), "{out}");
    assert_idempotent(src);
}

#[test]
fn dead_store_only_removes_pure_overwrites() {
    let src = r#"x = 1
x = 2
y = f()
y = 3
z = 1
z = f()
a = [1, 2]
a = 3
"#;
    let out = cured(src);
    assert!(out.contains("x = 2"), "{out}");
    assert!(!out.contains("x = 1"), "{out}");
    assert!(out.contains("y = f()\ny = 3"), "{out}");
    assert!(out.contains("z = 1\nz = f()"), "{out}");
    assert!(out.contains("a = 3"), "{out}");
    assert!(!out.contains("[1, 2]"), "{out}");
    assert_idempotent(src);
}

#[test]
fn shadowed_builtins_are_not_folded() {
    let src = r#"len = lambda x: 99
str = lambda x: "s"
a = len("abcd")
b = str(123)
def f(len):
    return len("abcd")
"#;
    let out = cured(src);
    assert!(out.contains("a = len(\"abcd\")"), "{out}");
    assert!(out.contains("b = str(123)"), "{out}");
    assert!(out.contains("return len(\"abcd\")"), "{out}");

    let out = cured("a = len(\"abcd\")\nb = str(123)\nc = bool(\"\")\n");
    assert!(out.contains("a = 4"), "{out}");
    assert!(out.contains("b = '123'"), "{out}");
    assert!(out.contains("c = False"), "{out}");

    // import * / builtins 标识符同样保守关闭。
    let out = cured("from math import *\na = len(\"abcd\")\n");
    assert!(out.contains("len(\"abcd\")"), "{out}");
    let out = cured("import builtins\na = len(\"abcd\")\n");
    assert!(out.contains("len(\"abcd\")"), "{out}");
}

#[test]
fn docstrings_are_not_created_by_folding_or_inlining() {
    let src = r#"if True:
    "a" + "b"
1
"not doc"
def f():
    "real doc"
    return "x" + "y"
class C:
    if True:
        "not class doc"
    def m(self):
        return "m" + "n"
"#;
    let out = cured(src);
    assert!(out.starts_with("pass\n'ab'"), "module 首条被错误处理：\n{out}");
    assert!(out.contains("def f():\n    \"real doc\""), "{out}");
    assert!(out.contains("class C:\n    pass\n    \"not class doc\""), "{out}");
    assert_idempotent(src);
}

#[test]
fn py3_simplify_semantics() {
    let Some(py) = python3() else {
        eprintln!("skip python3 differential");
        return;
    };
    let src = r#"s = "a" + "b"
t = "abc".upper()
u = "a-b".replace("-", "+")
v = "".join(["x", "y"])
n = len("abcd")
c = chr(66)
o = ord("B")
i = int("12")
w = str(34)
b1 = bool("")
b2 = bool("x")
m = max(2, 7)
sl = "abcdef"[1:5:2]
sl2 = "abcdef"[::-1]
sl3 = "abcdef"[4:1:-1]
sl4 = "abcdef"[::2]
sl5 = "abcdef"[-100:100]
sl6 = "abcdef"[100:]
sl7 = [0, 1, 2, 3, 4][::-1]
sl8 = [0, 1, 2, 3, 4][4:1:-1]
sl9 = (0, 1, 2, 3, 4)[::2]
rep2 = "a-a-a".replace("-", "+", 2)
st = "  AbC  ".strip()
st2 = "xxabcxx".strip("x")
sw = "aBc".swapcase()
ca = "aBC".capitalize()
lo = "ABC".lower()
ss = "abcdef".startswith("abc")
se = "abcdef".endswith("def")
ln = len([1, 2, 3])
idx = [10, 20, 30][-1]
tup = (1, 2, 3)[1:]
if 1:
    q = 5
else:
    q = 6
if 0:
    if 1:
        r = 7
else:
    r = 8
x = 1
x = 2
def h():
    return None
def ft(c):
    if c:
        return 1
    else:
        return 2
def gu(c):
    if c:
        raise ValueError("x")
    else:
        return 3
def hn(a, b):
    if a:
        if b:
            return 4
    return 5
outs = []
for cc in (0, 1):
    outs.append(ft(cc))
    try:
        outs.append(gu(cc))
    except Exception as e:
        outs.append(type(e).__name__)
    outs.append(hn(cc, 1))
    rr = 0
    if cc:
        rr = 1
    else:
        rr = 2
    outs.append(rr)
print(s, t, u, v, n, c, o, i, w, b1, b2, m, sl, sl2, sl3, sl4, sl5, sl6,
      sl7, sl8, sl9, rep2, st, st2, sw, ca, lo, ss, se, ln, idx, tup, q, r, x, h(), outs)
"#;
    differential(&PathBuf::from(py), src, "py3");
}

#[test]
fn py2_simplify_semantics_if_available() {
    let Some(py) = python2() else {
        eprintln!("skip python2 differential: set CURE_PY2");
        return;
    };
    let src = r#"s = "a" + "b"
print s, "abc".upper(), "a-b".replace("-", "+"), "".join(["x", "y"])
print len("abcd"), chr(65), ord("A"), int("42"), str(12), bool(""), max(1, 5)
print "abcdef"[1:4], [1, 2, 3][1], (1, 2, 3)[1:]
print "abcdef"[::-1], "abcdef"[4:1:-1], [0, 1, 2][::-1], (0, 1, 2)[::2]
print "a-a-a".replace("-", "+", 2), "  AbC  ".strip(), "xxabcxx".strip("x")
print "aBc".swapcase(), "aBC".capitalize(), "ABC".lower()
print "abcdef".startswith("abc"), "abcdef".endswith("def"), len([1, 2, 3])
if 1:
    x = 1
else:
    x = 2
print x
if 0:
    y = 1
else:
    y = 2
print y
q = 1
q = 2
print q
def h():
    return None
print h()
def ft(c):
    if c:
        return 1
    else:
        return 2
def gu(c):
    if c:
        raise ValueError("x")
    else:
        return 3
def hn(a, b):
    if a:
        if b:
            return 4
    return 5
for cc in (0, 1):
    print ft(cc)
    try:
        print gu(cc)
    except Exception as e:
        print type(e).__name__
    print hn(cc, 1)
    rr = 0
    if cc:
        rr = 1
    else:
        rr = 2
    print rr
"#;
    differential(&py, src, "py2");
}
#[test]
fn deep_nested_ifs_converge() {
    let mut src = String::from("def f(c):\n");
    for i in 0..20 {
        src.push_str(&format!("{}if c:\n", "    ".repeat(i + 1)));
    }
    src.push_str(&format!("{}return 1\n", "    ".repeat(21)));
    let once = cured(&src);
    let twice = cured(&once);
    assert_eq!(once, twice, "深层嵌套 if 未收敛");
    assert!(once.contains("if c and c"), "{once}");
}

#[test]
fn exec_disables_bool_constant_folding() {
    let src = "exec \"True = 0\"\nif True:\n    x = 1\nelse:\n    x = 2\n";
    let out = cured(src);
    assert!(out.contains("if True"), "exec 可能改写 True 时仍折叠了：\n{out}");
    let _ = parse(src); // 解析层不做版本判断，仅确认可解析
}

#[test]
fn py2_unicode_literals_keeps_str_and_chr_types() {
    let Some(py) = python2() else {
        eprintln!("skip python2 unicode_literals: set CURE_PY2");
        return;
    };
    let src = r#"from __future__ import unicode_literals
print type(str(123)).__name__, str(123)
print type(chr(65)).__name__, chr(65)
print type("a" + "b").__name__, "a" + "b"
"#;
    let out = cured(src);
    assert!(out.contains("str(123)"), "{out}");
    assert!(out.contains("chr(65)"), "{out}");
    differential(&py, src, "py2_unicode");
}

#[test]
fn docstrings_survive_at_runtime_under_python3() {
    let Some(py) = python3() else {
        eprintln!("skip python3 docstring differential");
        return;
    };
    let src = r#"if True:
    "a" + "b"
1
"not doc"
class C:
    if True:
        "c" + "d"
    def m(self):
        "method doc"
print(__doc__, C.__doc__, C.m.__doc__)
"#;
    differential(&PathBuf::from(py), src, "py3_doc");
}
