//! Python 语义差分（CPython 3.15）：原版 vs cure 版 stdout/退出码一致；
//! 二次 cure 逐字节幂等。无 python3 时跳过。

use std::fs;
use std::path::Path;
use std::process::Command;

use cure_engine::Config;
use cure_python_parser::parse;
use cure_python_print::print_module;
use cure_python_simplify::simplify_module;

fn python_bin() -> Option<&'static str> {
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

fn cured(src: &str) -> (String, usize) {
    let mut out = parse(src);
    let errs = out.errors.len();
    assert!(
        out.errors.is_empty(),
        "测试源码必须干净解析：{:?}",
        out.errors
    );
    let _ = simplify_module(&mut out.ast, out.module, &Config::default());
    (print_module(&out.ast, out.module), errs)
}

fn differential(name: &str, src: &str) {
    let (once, _) = cured(src);
    let (twice, _) = cured(&once);
    assert_eq!(once, twice, "{name}: 二次 cure 不幂等");
    let Some(py) = python_bin() else {
        eprintln!("skip python differential: python3 missing");
        return;
    };
    let base = std::env::temp_dir().join(format!("cure_py_{name}"));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let orig = base.join("orig.py");
    let cur = base.join("cured.py");
    fs::write(&orig, src).unwrap();
    fs::write(&cur, &once).unwrap();
    let run = |f: &Path| {
        let out = Command::new(py).arg(f).output().expect("run python");
        (
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
            out.status.code(),
        )
    };
    let (oo, oe, oc) = run(&orig);
    let (co, ce, cc) = run(&cur);
    assert_eq!(
        oc, cc,
        "{name}: 退出码改变\n--stderr orig--\n{oe}\n--cured--\n{once}\n--stderr cured--\n{ce}"
    );
    assert_eq!(
        oo, co,
        "{name}: stdout 改变\n--orig--\n{oo}\n--cured--\n{co}\n--src--\n{once}"
    );
}

#[test]
fn py_constants_and_flow() {
    differential(
        "consts",
        r#"print(7//-2, 7%-2, -7//2, -7%2)
print(2**10, 1<<5, -5>>1, ~3)
print(1+2*3-4, (1+2)*3)
print(not 0, not 1, not None)
print(True and 7, False and 7, 0 or 9, 1 or 9)
if True:
    print("t")
else:
    print("f")
if False:
    print("bad")
elif 1:
    print("elif")
while False:
    print("bad")
else:
    print("welse")
"#,
    );
}

#[test]
fn py_side_effects_short_circuit() {
    differential(
        "effects",
        r#"def f(x):
    print("call", x)
    return x
print(True and f("a"), False and f("b"))
print(True or f("c"), False or f("d"))
print(0 and f("e"), 1 or f("f"))
print(f("g") and True, f("h") or False)
"#,
    );
}

#[test]
fn py_generator_yield_markers_survive() {
    differential(
        "generators",
        r#"def baditer():
    raise TypeError
    yield True
def gen():
    raise StopIteration
    yield
def whoo():
    if False:
        yield
def g(n):
    x = 0
    while x < n:
        y = x * 2
        y = yield y
        x += 1
try:
    set(baditer())
except TypeError:
    print("type-error")
for fn in (gen, whoo):
    print(type(fn()).__name__)
print(list(g(2)))
"#,
    );
}

#[test]
fn py_classes_exceptions_match() {
    differential(
        "classes",
        r#"class A:
    def __init__(self, v): self.v = v
    @property
    def double(self): return self.v * 2
class B(A):
    def __init__(self, v): super().__init__(v + 1)
    def double(self): return super().double + 1
b = B(3)
print(b.v, b.double())
def f(mode):
    try:
        if mode: raise ValueError("v")
        return "ok"
    except ValueError as e:
        return ("caught", str(e))
    finally:
        print("finally")
print(f(0), f(1))
def m(v):
    match v:
        case [1, *rest] if rest: return ("list", rest)
        case {"a": a, **kw}: return ("dict", a, sorted(kw))
        case 1 | 2 | 3: return "small"
        case _: return None
for v in ([1,2], {"a":5,"b":6}, 2, 9):
    print(m(v))
"#,
    );
}

#[test]
fn py_del_unbinds_and_forbids_propagation() {
    differential(
        "del_scope",
        r#"def del_then_read():
    v = 5
    del v
    try:
        return v
    except UnboundLocalError:
        return 'unbound'

def del_then_bare_read():
    v = 1
    del v
    try:
        v
    except UnboundLocalError:
        return 'gone'

def del_then_rebind():
    v = 1
    del v
    v = 2
    return v

def del_item():
    box = {'k': 1}
    del box['k']
    return box

print(del_then_read(), del_then_bare_read(), del_then_rebind(), del_item())
"#,
    );
}

#[test]
fn py_comprehensions_and_walrus() {
    differential(
        "comp",
        r#"print([i*j for i in range(3) for j in range(2) if (i+j)%2])
print({k:v for k,v in zip("abc", range(3))})
print({i%3 for i in range(10)})
a = [1,2,3]
b = a[x:=0]
print(b, x)
data = [1,2]
if (n := len(data)) > 1:
    print(n)
print([y for v in range(5) if (y := v*v) % 2 == 0])
"#,
    );
}

#[test]
fn py_315_syntax_roundtrip() {
    let src = r#"lazy import os
lazy from json import dumps
t = t"value={1+1}"
def f[T](x: T) -> T:
    return x
class C[T]:
    pass
try:
    raise ExceptionGroup("g", [ValueError("v")])
except* ValueError as eg:
    print("eg", len(eg.exceptions))
print(t, f(1))
"#;
    let (out, errs) = cured(src);
    assert_eq!(errs, 0);
    assert!(out.contains("lazy import os"), "{out}");
    assert!(out.contains("t\"value={1+1}\""), "{out}");
    assert!(out.contains("def f[T]"), "{out}");
    assert!(out.contains("except* ValueError"), "{out}");
}

#[test]
fn py_roundtrip_ast_equivalence() {
    let Some(py) = python_bin() else {
        eprintln!("skip python ast check: python3 missing");
        return;
    };
    let src = r#"import os, sys
from typing import Any

def f(a, b=2, *args, c=3, **kw) -> dict[str, Any]:
    s = f"x={a!r:>4}"
    q = [i for i in range(3) if i]
    z = (a := b) + 1
    return {"a": a, "s": s, "q": q, "z": z}

class K[T]:
    x: int = 1
    def m(self):
        match self.x:
            case [1, *r] if r:
                return r
            case _:
                return None
"#;
    let out = parse(src);
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    let printed = print_module(&out.ast, out.module);
    let base = std::env::temp_dir().join("cure_py_ast");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    fs::write(base.join("a.py"), src).unwrap();
    fs::write(base.join("b.py"), &printed).unwrap();
    let script = r#"
import ast, sys
a = ast.dump(ast.parse(open(sys.argv[1]).read()), include_attributes=False)
b = ast.dump(ast.parse(open(sys.argv[2]).read()), include_attributes=False)
sys.exit(0 if a == b else 1)
"#;
    let st = Command::new(py)
        .arg("-c")
        .arg(script)
        .arg(base.join("a.py"))
        .arg(base.join("b.py"))
        .status()
        .expect("run python ast check");
    assert!(st.success(), "AST 不等价；输出：\n{printed}");
}

#[test]
fn negative_literal_canonicalization_is_not_a_reported_edit() {
    let mut out = parse("x = -1\ny = -1 + 2\n");
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    let report = simplify_module(&mut out.ast, out.module, &Config::default());
    let printed = print_module(&out.ast, out.module);
    assert_eq!(printed, "x = -1\ny = 1\n");
    assert_eq!(
        report.edits, 1,
        "only the parent fold is observable: {report:?}"
    );

    let mut again = parse(&printed);
    let report2 = simplify_module(&mut again.ast, again.module, &Config::default());
    assert_eq!(
        report2.edits, 0,
        "converged file still reports edits: {report2:?}"
    );
}
