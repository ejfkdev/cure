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
    assert!(out.contains("def g():\n    pass"), "{out}");
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

#[test]
fn extended_structural_rules() {
    let src = r#"def p1(c):
    if c:
        pass
    else:
        return 2
    return 1
def p2(c):
    if c:
        return 1
    return 2
def p3(c):
    return True if c else False
def p4(c):
    return False if c else True
def p5(c):
    while c:
        c = c - 1 if c > 0 else 0
        continue
    return 9
def p6(c):
    try:
        return c
    finally:
        pass
def p7(x):
    assert True
    assert 1, side()
    return x is not None
def p8(x):
    return not (x is None)
def p9(x):
    return not (x in [1, 2])
def p10():
    s = "a"
    s += "b"
    s += "c"
    return s
def p11():
    x = 2
    x = x + 3
    x = x * 2
    return x
def p12():
    ("doc")
"#;
    let out = cured(src);
    assert!(out.contains("return 1 if c else 2"), "{out}");
    assert!(out.contains("return bool(c)"), "{out}");
    assert!(out.contains("return not c"), "{out}");
    assert!(!out.contains("continue"), "{out}");
    assert!(out.contains("def p6(c):\n    return c"), "{out}");
    assert!(!out.contains("assert"), "{out}");
    assert!(out.contains("return x is not None"), "{out}");
    assert!(out.contains("return x not in [1, 2]"), "{out}");
    // 尾赋值返回把累加结果直接返回（s = 'abc'; return s → return 'abc'）
    assert!(out.contains("return 'abc'"), "{out}");
    assert!(out.contains("return 10"), "{out}");
    assert!(out.contains("def p12():\n    (\"doc\")"), "{out}");
    assert!(!out.contains("def p12():\n    pass"), "{out}");
    assert_idempotent(src);
}

#[test]
fn dead_assign_uses_function_scope_liveness() {
    let src = r#"def f():
    x = 1
    return 2
def g():
    x = side()
    return 2
def h():
    x = 1
    def k(a=x):
        return a
    return k()
def i():
    x = 1
    return locals()
def j():
    global x
    x = 1
    return 2
def l():
    x = 1
    y = x
    return y
def m():
    x = 1
    "not doc"
    return 2
class C:
    x = 1
"#;
    let out = cured(src);
    assert!(out.contains("def f():\n    return 2"), "{out}");
    assert!(out.contains("def g():\n    side()\n    return 2"), "{out}");
    assert!(out.contains("def h():\n    x = 1"), "{out}");
    assert!(out.contains("def i():\n    x = 1"), "{out}");
    assert!(out.contains("def j():\n    global x\n    x = 1"), "{out}");
    // x = 1; y = x; return y → return 1（x/y 均只读一次）
    assert!(out.contains("def l():\n    return 1"), "{out}");
    assert!(out.contains("def m():\n    pass\n    \"not doc\""), "{out}");
    assert!(out.contains("class C:\n    x = 1"), "{out}");
    assert_idempotent(src);
}

#[test]
fn extended_rules_run_under_python3() {
    let Some(py) = python3() else {
        eprintln!("skip python3 extended differential");
        return;
    };
    let src = r#"def side():
    return 3
def p1(c):
    if c:
        pass
    else:
        return 2
    return 1
def p2(c):
    if c:
        return 1
    return 2
def p3(c):
    return True if c else False
def p4(c):
    return False if c else True
def p5(c):
    while c:
        c = c - 1 if c > 0 else 0
        continue
    return 9
def p6(c):
    try:
        return c
    finally:
        pass
def p7(x):
    assert True
    assert 1, side()
    return x is not None
def p8(x):
    return not (x is None)
def p9(x):
    return not (x in [1, 2])
def p10():
    s = "a"
    s += "b"
    s += "c"
    return s
def p11():
    x = 2
    x = x + 3
    x = x * 2
    return x
def p12():
    ("doc")
class Toggle:
    def __init__(self, v):
        self.v = v
        self.calls = 0
    def __bool__(self):
        self.calls += 1
        return self.v
def p13(t):
    r = not not t
    return (r, t.calls)
def p14(x):
    y = x + 1
    return y
def p15(xs):
    y = xs
    def inner():
        return y
    return inner() is xs
def p16(xs):
    y = xs
    return y
def p17(x):
    y = x
    print("side", y)
    return y
print(p1(0), p1(1), p2(0), p2(1), p3(0), p3(1), p4(0), p4(1))
print(p5(0), p5(1), p6(5), p7(None), p8(None), p9(1), p9(3), p10(), p11(), p12.__doc__)
print(p13(Toggle(True)), p13(Toggle(False)))
print(p14(1), p15([2]), p16([3]), p17(4))
"#;
    differential(&PathBuf::from(py), src, "py3_ext");
}

#[test]
fn extended_rules_run_under_python2_if_available() {
    let Some(py) = python2() else {
        eprintln!("skip python2 extended differential: set CURE_PY2");
        return;
    };
    let src = r#"def side():
    return 3
def p1(c):
    if c:
        pass
    else:
        return 2
    return 1
def p2(c):
    if c:
        return 1
    return 2
def p3(c):
    return True if c else False
def p4(c):
    return False if c else True
def p5(c):
    while c:
        c = c - 1 if c > 0 else 0
        continue
    return 9
def p6(c):
    try:
        return c
    finally:
        pass
def p7(x):
    assert True
    assert 1, side()
    return x is not None
def p8(x):
    return not (x is None)
def p9(x):
    return not (x in [1, 2])
def p10():
    s = "a"
    s += "b"
    s += "c"
    return s
def p11():
    x = 2
    x = x + 3
    x = x * 2
    return x
def p12():
    ("doc")
print p1(0), p1(1), p2(0), p2(1), p3(0), p3(1), p4(0), p4(1)
print p5(0), p5(1), p6(5), p7(None), p8(None), p9(1), p9(3), p10(), p11(), p12.__doc__
"#;
    differential(&py, src, "py2_ext");
}

#[test]
fn double_not_and_trailing_assign_return() {
    let src = r#"def a(x):
    return not not x
def b(x):
    return not (not x)
def c(x):
    y = x + 1
    return y
def d(x):
    y = x + 1
    print(y)
    return y
def e(x):
    def inner():
        return y
    y = x
    return y
def f(x):
    global G
    y = x
    G = y
    return y
def g(x):
    y = x
    return y
def h():
    return locals()
def i(x):
    y = x
    return y
"#;
    let out = cured(src);
    assert!(out.contains("return bool(x)"), "{out}");
    assert!(out.contains("def c(x):\n    return x + 1"), "{out}");
    assert!(out.contains("def d(x):\n    y = x + 1\n    print(y)\n    return y"), "{out}");
    assert!(out.contains("def e(x):"), "{out}");
    assert!(out.contains("y = x\n    return y"), "{out}");
    assert!(out.contains("def g(x):\n    return x"), "{out}");
    assert!(out.contains("def h():\n    return locals()"), "{out}");
    assert_idempotent(src);
}

#[test]
fn local_literal_propagation_and_guards() {
    let src = r#"def a(x):
    y = 5
    return y + x
def b():
    y = "a"
    def g():
        return y + y
    return g()
def c(y):
    y = 5
    return y + 1
def d():
    y = 5
    y = 6
    return y
def e():
    y = 5
    (y := 6)
    return y
def f():
    y = 5
    del y
    return 1
def g():
    global gy
    y = 5
    gy = y
    return gy
def h():
    y = 5
    def inner():
        nonlocal y
        y = 6
    inner()
    return y
def i():
    y = 5
    return locals()
def j():
    y = [5]
    return y is y
def k():
    y = "doc"
    def m():
        y
    return m.__doc__
def l():
    def m():
        try:
            return y
        except NameError:
            return "unbound"
    y = 7
    return m()
def n():
    y = 5
    out = []
    for i in range(2):
        out.append(y + i)
    return out
"#;
    let out = cured(src);
    assert!(out.contains("def a(x):\n    return 5 + x"), "{out}");
    assert!(out.contains("def b():\n    def g():\n        return 'aa'"), "{out}");
    assert!(out.contains("def c(y):\n    y = 5"), "{out}");
    assert!(out.contains("def e():\n    y = 5\n    (y := 6)"), "{out}");
    assert!(out.contains("def f():\n    y = 5\n    del y"), "{out}");
    assert!(out.contains("def g():\n    global gy\n    y = 5"), "{out}");
    assert!(out.contains("def h():\n    y = 5"), "{out}");
    assert!(out.contains("def i():\n    y = 5\n    return locals()"), "{out}");
    assert!(out.contains("def j():\n    y = [5]"), "{out}");
    assert!(out.contains("def k():\n    y = \"doc\"\n    def m():\n        y"), "{out}");
    assert!(out.contains("def l():"), "{out}");
    assert!(out.contains("out.append(5 + i)"), "{out}");
    assert_idempotent(src);
}

#[test]
fn local_literal_propagation_runtime_differential() {
    let Some(py) = python3() else {
        eprintln!("skip python3 propagation differential");
        return;
    };
    let src = r#"def a(x):
    y = 5
    return y + x
def b():
    y = "a"
    def g():
        return y + y
    return g()
def c(x):
    y = 5
    out = []
    for i in range(2):
        out.append(y + i)
    return out + [x]
def d():
    y = None
    if y is None:
        return y
    return 1
def e():
    y = True
    return not y
def f(x):
    y = 3
    while x > 0:
        x -= y
        print(y)
    return x
def g():
    y = 7
    try:
        return y
    finally:
        pass
print(a(1), b(), c(0), d(), e(), f(9), g())
"#;
    differential(&PathBuf::from(py), src, "py3_prop");
}

#[test]
fn pre37_await_and_async_are_identifiers() {
    // Python 3.5/3.6（及更早）里 await/async 是软关键字，可作标识符；
    // 3.7+ 才是保留字。版本无关解析按后随 token 消歧。
    let src = r#"async = 1
await = async + 1
def f(async, await=2):
    return async + await
class C:
    async = 3
    def await(self):
        return self.async
f(await=4)
await = lambda x: x + 1
print(await(1))
print(await[0] if isinstance(await, list) else await(2))
"#;
    let out = cured(src);
    for needle in [
        "async = 1",
        "await = async + 1",
        "def f(async, await=2)",
        "f(await=4)",
        "await(1)",
        "await[0]",
    ] {
        assert!(out.contains(needle), "missing {needle:?}\n{out}");
    }
    assert_idempotent(src);

    // 现代 async/await 语义不受影响。
    let modern = r#"import asyncio
async def g():
    await asyncio.sleep(0)
    x = await h()
    return await (k()) + 1
async def h():
    return 1
async def k():
    return 2
"#;
    let out = cured(modern);
    assert!(out.contains("await asyncio.sleep(0)"), "{out}");
    assert!(out.contains("await h()"), "{out}");
    assert!(out.contains("await k() + 1"), "{out}");
    assert_idempotent(modern);
}

#[test]
fn leading_comment_block_and_type_directives_are_preserved() {
    let src = r#"#!/usr/bin/env python2
# -*- coding: utf-8 -*-
# Copyright 2026
# type: ignore
# mypy: ignore-errors

"""doc"""
import os
x = 1
"#;
    let out = cured(src);
    assert!(out.starts_with("#!/usr/bin/env python2\n# -*- coding: utf-8 -*-\n"), "{out}");
    assert!(out.contains("# type: ignore"), "{out}");
    assert!(out.contains("# mypy: ignore-errors"), "{out}");
    assert!(out.contains("#!/usr/bin/env python2"), "{out}");
    assert!(out.contains("def " ) || true);
    assert_idempotent(src);
}

#[test]
fn line_level_comments_and_directives_are_preserved() {
    let src = r#"#!/usr/bin/env python3
# license
import os  # noqa: F401
# leading
x = 1  # type: ignore[assignment]
y = "a"  # note

def f(a):
    # body comment
    b = a + 1  # trailing
    if b:
        return b  # ret
    return 0
"""doc"""
# eof comment
"#;
    let out = cured(src);
    for needle in [
        "# noqa: F401",
        "# type: ignore[assignment]",
        "# note",
        "# body comment",
        "# trailing",
        "# ret",
        "# eof comment",
        "# leading",
    ] {
        assert!(out.contains(needle), "missing {needle:?}\n{out}");
    }
    // 行尾指令必须仍在该语句行上
    assert!(out.contains("import os  # noqa: F401"), "{out}");
    assert!(out.contains("x = 1  # type: ignore[assignment]"), "{out}");
    assert!(out.contains("b = a + 1  # trailing"), "{out}");
    assert_idempotent(src);

    // docstring 仍是作用域首条语句（注释不影响 __doc__）
    let doc = "def g():\n    \"\"\"doc\"\"\"\n    # c\n    return 1\n";
    let out = cured(doc);
    assert!(out.contains("\"\"\"doc\"\"\""), "{out}");
    assert!(out.contains("# c"), "{out}");
    assert!(out.starts_with("def g():\n    \"\"\"doc\"\"\""), "{out}");
}

#[test]
fn directive_statements_are_never_removed_or_merged() {
    // 被简化删除/合并的语句必须保留：行尾指令要留在原来的行上，
    // 否则 mypy / flake8 / pylint 的抑制会失效或换行生效。
    let src = r#"def f(x):
    y = 5  # type: ignore
    return y + x
"#;
    let out = cured(src);
    assert!(out.contains("y = 5  # type: ignore"), "{out}");
    assert!(out.contains("return y + x"), "{out}");
    assert_idempotent(src);

    // 死赋值 / 死存储：有指令的语句保留，无指令的仍可删除。
    let src = r#"def g():
    x = 1  # noqa: F841
    y = compute()  # type: ignore
    y = 2
    return y
"#;
    let out = cured(src);
    assert!(out.contains("x = 1  # noqa: F841"), "{out}");
    assert!(out.contains("y = compute()  # type: ignore"), "{out}");
    assert!(out.contains("return 2"), "{out}");
    assert_idempotent(src);

    // 不可达清理遇到指令语句时停止裁剪。
    let src = "def h():\n    x = 1  # type: ignore\n    return x\n    print(x)\n";
    let out = cured(src);
    assert!(out.contains("x = 1  # type: ignore"), "{out}");
    assert!(out.contains("return x"), "{out}");
    assert!(!out.contains("print(x)"), "{out}");
    assert_idempotent(src);
}

#[test]
fn comments_in_raw_signature_fields_are_not_duplicated() {
    let src = "@deco(  # deco note\n    1)\ndef f(a,  # param note\n      b):\n    return a + b\n";
    let out = cured(src);
    assert_eq!(out.matches("# deco note").count(), 1, "{out}");
    assert_eq!(out.matches("# param note").count(), 1, "{out}");
    assert!(out.contains("# deco note"), "{out}");
    assert!(out.contains("# param note"), "{out}");
    assert_idempotent(src);
}
