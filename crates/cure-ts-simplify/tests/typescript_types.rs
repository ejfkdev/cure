//! 复杂 TS 类型语法：类型别名/映射/条件类型、ambient 声明、明确赋值、
//! 可选成员、泛型调用/new、返回类型、类型化函数表达式/箭头。
//! 这些形态曾在真实语料（zod/minisearch/undici）里被拆散或吞到 EOF。

use cure_engine::Config;
use cure_ts_parser::{parse_js, parse_with_options, ParseOptions};
use cure_ts_print::print_module;
use cure_ts_simplify::simplify_unit;

fn cure_ts(src: &str) -> String {
    let mut out = parse_with_options(src, ParseOptions::default());
    assert!(out.errors.is_empty(), "解析失败: {:?}", out.errors);
    let _ = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
    let once = print_module(&out.ast, &out.unit);
    let mut out2 = parse_with_options(&once, ParseOptions::default());
    assert!(out2.errors.is_empty(), "输出不可再解析: {:?}\n{once}", out2.errors);
    let _ = simplify_unit(&mut out2.ast, &mut out2.unit, &Config::default());
    let twice = print_module(&out2.ast, &out2.unit);
    assert_eq!(once, twice, "二次 cure 不幂等");
    once
}

#[test]
fn complex_type_declarations_are_preserved() {
    let src = r#"export type U =
  | "a"
  | "b"
  | { x: string; y?: number };

type M<T> = T extends [any, ...any[]]
  ? { [K in keyof T]?: T[K] }
  : T extends object
    ? { [K in keyof T]: T[K] }
    : unknown;

export interface I<T> extends Base {
  x: T;
  m<U>(this: I<U>): U;
}

export const enum E { A, B }

declare type D<T> = { [K in keyof T]: T[K] };

export const utils: {
  a(x: string): void
  b(y: number): void
}

declare const loc: BrowserLocators

type Re = `a${string}b`;

const n: number = 1 + 2;
export function f<T, K extends keyof T>(x: T): Promise<{ [p in keyof T]: T[p] }> {
  return x as any;
}
console.log(n);
"#;
    let out = cure_ts(src);
    assert!(out.contains("type M<T> = T extends [any, ...any[]]"), "{out}");
    assert!(out.contains("[K in keyof T]"), "{out}");
    assert!(out.contains("declare type D<T>"), "{out}");
    assert!(out.contains("export const utils: {"), "{out}");
    assert!(out.contains("declare const loc: BrowserLocators"), "{out}");
    // 值级代码仍简化
    assert!(out.contains("const n = 3") || out.contains("console.log(3)"), "{out}");
}

#[test]
fn generic_calls_and_new_preserve_type_args() {
    let src = r#"const a = new WeakMap<object, Set<string>>();
const b = new Map<string, Array<number>>();
const c = f<number>(1);
const d = o.m<string>(2);
const e = new Foo<string>;
const g = new ns.Foo<A, B<C>>(3);
const h = f<string>;
console.log(a, b, c, d, e, g, h);
"#;
    let out = cure_ts(src);
    // 类型实参不参与运行时求值，但**类型层可见**：必须原文保留
    //（`useState<T|null>(null)` 丢弃后会推断成 `null`）。
    assert!(out.contains("new WeakMap<object, Set<string>>()"), "{out}");
    assert!(out.contains("new Map<string, Array<number>>()"), "{out}");
    assert!(out.contains("f<number>(1)"), "{out}");
    assert!(out.contains("o.m<string>(2)"), "{out}");
    assert!(out.contains("new Foo<string>"), "{out}");
    assert!(out.contains("new ns.Foo<A, B<C>>(3)"), "{out}");
    assert!(out.contains("const h = f<string>;"), "{out}");
}

#[test]
fn typed_function_expressions_and_arrows() {
    let src = r#"const stub = function StubFunction(..._args: unknown[]): never {
  throw new Error("x");
};
const make = (entry: [string, number]): { ref: string; defId?: string } => {
  return { ref: entry[0] };
};
const obj = {
  get features(): string | undefined {
    return "x";
  },
  meta(...args: any[]): any {
    return args;
  },
};
class C {
  pick = (...args: any[]): any => args[0];
  field!: string[];
  optional?: number;
  get x(): number { return 1; }
  static lazycreate = <Shape extends object>(
    shape: () => Shape,
    params?: P
  ): ZodObject<Shape, "strip"> => {
    outer: for (let i = 0; i < 1; i++) { if (i) break outer; }
    return shape as any;
  };
}
console.log(stub, make, obj, C);
"#;
    let out = cure_ts(src);
    assert!(out.contains("function StubFunction(..._args: unknown[]): never"), "{out}");
    assert!(out.contains("): { ref: string; defId?: string } =>"), "{out}");
    assert!(out.contains("get features(): string | undefined"), "{out}");
    assert!(out.contains("meta(...args: any[]): any"), "{out}");
    assert!(out.contains("pick = (...args: any[]): any =>"), "{out}");
    assert!(out.contains("field!: string[]"), "{out}");
    assert!(out.contains("optional?: number"), "{out}");
    assert!(out.contains("outer: for"), "{out}");
}

#[test]
fn as_expression_stops_at_ternary_colon() {
    let src = r#"function f(params: any) {
  return typeof params.error === "string"
    ? ({ ...params, error: () => params.error }) as any
    : params;
}
function g(catchValue: any): any {
  return {
    catchValue: (typeof catchValue === "function" ? catchValue : () => catchValue) as (
      ctx: any
    ) => any,
  } as any;
}
console.log(f, g);
"#;
    let out = cure_ts(src);
    assert!(out.contains("as any : params"), "{out}");
    assert!(!out.contains(";;}") && !out.contains(": :"), "{out}");
}

#[test]
fn js_mode_keeps_comparison_chains() {
    // 纯 JS：`a < b > (c)` 是比较链，不是泛型调用 `a<b>(c)`。
    let src = "const k = a < b > (c); console.log(k);\n";
    let mut out = parse_js(src);
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    let _ = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
    let printed = print_module(&out.ast, &out.unit);
    assert!(!printed.contains("a(c)"), "{printed}");
    assert!(printed.contains('<') && printed.contains('>'), "{printed}");
}

#[test]
fn iife_inline_only_when_scope_neutral() {
    let src = r#"
const a = (() => 42)();
const b = (() => { return 7; })();
const c = (() => "x" + "y")();
const d = (function () { return 1; })();
function f(x: number) {
  const e = (() => x + 1)();
  return e;
}
class K {
  y = 1;
  m() { return (() => this.y)(); }
}
function h() { return (function () { return this; })(); }
function i() { return (function () { return arguments[0]; })(); }
const j = (() => { const z = 1; return z; })();
const l = (async () => 1)();
const m = (x: number) => x;
const n = ((x: number) => x)(1);
"#;
    let out = cure_ts(src);
    // 零参、单表达式 / 单 return 的 IIFE 内联
    assert!(out.contains("const a = 42"), "{out}");
    assert!(out.contains("const b = 7"), "{out}");
    assert!(out.contains("const c = 'xy'"), "{out}");
    assert!(out.contains("const d = 1"), "{out}");
    assert!(out.contains("return x + 1"), "{out}");
    // 箭头词法 this 保留内联；普通函数的 this/arguments 不内联
    assert!(out.contains("m() {\n        return this.y;"), "{out}");
    assert!(out.contains("(function () {\n        return this;"), "{out}");
    assert!(out.contains("return arguments[0];"), "{out}");
    // 有局部声明 / async / 有参：保持
    assert!(out.contains("const z = 1;"), "{out}");
    assert!(out.contains("(async () => 1)()"), "{out}");
    assert!(out.contains("((x: number) => x)(1)"), "{out}");
    let twice = cure_ts(&out);
    assert_eq!(out, twice, "not idempotent:\n{out}");

    // 模块里出现直接 eval：引擎的 eval 污染区冻结模块级改写（保守），
    // IIFE 保留；函数内部的非污染子树仍可简化。
    let eval_src = "const k = (() => eval(\"1\"))();\nfunction g() { return (() => 2)(); }\n";
    let out = cure_ts(eval_src);
    assert!(out.contains("eval('1')"), "{out}");
    assert!(out.contains("return 2"), "{out}");
}
