//! 最新 TS/JS 语法回归：干净解析 → 简化 → 打印 → 再解析 → 幂等。
//! 只断言语法保真与稳定，不执行（部分特性当前 node 不支持）。

use cure_engine::Config;
use cure_ts_parser::{parse, parse_module};
use cure_ts_print::print_module;
use cure_ts_simplify::simplify_unit;

fn roundtrip(src: &str, module: bool) {
    let mut out = if module { parse_module(src) } else { parse(src) };
    assert!(out.errors.is_empty(), "{src}\n{:?}", out.errors);
    let _ = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
    let once = print_module(&out.ast, &out.unit);
    let mut out2 = if module { parse_module(&once) } else { parse(&once) };
    assert!(out2.errors.is_empty(), "printed:\n{once}\n{:?}", out2.errors);
    let _ = simplify_unit(&mut out2.ast, &mut out2.unit, &Config::default());
    let twice = print_module(&out2.ast, &out2.unit);
    assert_eq!(once, twice, "not idempotent:\n{once}\n---\n{twice}");
}

#[test]
fn explicit_resource_management() {
    roundtrip("using x = acquire();\n", false);
    roundtrip("async function f() { await using x = acquire(); }\n", false);
    roundtrip("await using x = acquire();\n", true);
    roundtrip("for (using x of xs) { use(x); }\n", false);
}

#[test]
fn import_attributes_and_phase_imports() {
    roundtrip(
        "import data from './x.json' with { type: 'json' };\nexport * as ns from './m.js';\n",
        true,
    );
    roundtrip("import defer * as ns from './m.js';\n", true);
    roundtrip("import source mod from './m.wasm';\n", true);
    roundtrip("import('./x.json', { with: { type: 'json' } });\n", false);
}

#[test]
fn decorators_accessors_private_and_static_blocks() {
    roundtrip(
        "@dec class C {\n  @dec accessor x = 1;\n  static { this.y = 1; }\n  #p = 2;\n  #m() { return #p in this; }\n}\n",
        false,
    );
}

#[test]
fn typescript_modern_operators() {
    roundtrip(
        "let x = y satisfies T;\nlet z = a as const;\nlet n = 1_000n;\n",
        false,
    );
    roundtrip(
        "function f<const T extends string>(x: T): T { return x; }\nconsole.log(f('x'));\n",
        false,
    );
    roundtrip("const g = <T,>(x: T): T => x;\nconsole.log(g(1));\n", false);
    roundtrip(
        "const h = async <T extends object>(x: T): Promise<T> => x;\nconsole.log(h);\n",
        false,
    );
    roundtrip(
        "const k = function <T>(x: T) { return x; };\nconsole.log(k(1));\n",
        false,
    );
    roundtrip(
        "class C<T extends string> { m<U>(x: U): U { return x; } }\nconsole.log(C);\n",
        false,
    );
}

#[test]
fn modern_regex_and_optional_chains() {
    roundtrip("const r = /[\\p{L}--[a]]/v;\n", false);
    roundtrip("const u = obj?.a?.[b]?.(c) ?? d;\nconst t = tag`x${y}`;\n", false);
}
