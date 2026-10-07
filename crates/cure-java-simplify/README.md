# cure-java-simplify

[![crates.io](https://img.shields.io/crates/v/cure-java-simplify.svg)](https://crates.io/crates/cure-java-simplify)
[![Documentation](https://docs.rs/cure-java-simplify/badge.svg)](https://docs.rs/cure-java-simplify)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/ejfkdev/cure/blob/main/LICENSE)

The Java rule pack of [cure](https://github.com/ejfkdev/cure): 58
semantic-preserving simplification rules over
[`cure-java-ast`](https://crates.io/crates/cure-java-ast), driven to a fixed
point by [`cure-engine`](https://crates.io/cure-engine).

The point is not compression — it is making **decompiled** code read like
hand-written code: fold constants and opaque predicates, inline copies, kill
dead stores and compiler register noise, recover `StringBuilder` statement
chains, dissolve control-flow-flattening state machines, evaluate
literal-only JDK calls, and restore idiomatic shapes (try-with-resources,
string switch).

```rust
use cure_engine::Config;
use cure_java_parser::parse;
use cure_java_print::print_unit;
use cure_java_simplify::simplify_unit;

let src = "class B { void n() { int x = 1 + 2; boolean c = (x > 2) && true;
            if (c) { System.out.println(\"y\" + 1); } return; } }";
let mut out = parse(src);
let report = simplify_unit(&mut out.ast, &mut out.unit, &Config::default());
println!("{} edits", report.edits);
println!("{}", print_unit(&out.ast, &out.unit));
// class B {
//     void n() {
//         System.out.println("y1");
//     }
// }
```

## Correctness methodology

Every rule is gated by effect analysis (evaluation order, short-circuiting,
and side-effect sequences are preserved) and validated in three layers:
property testing against an interpreter, differential testing against real
`javac`/`java` runs (byte-identical stdout/exit codes/exception signatures),
and whole-corpus idempotency (second pass makes zero edits). See the
[verification section](https://github.com/ejfkdev/cure#how-correctness-is-verified)
of the repository README.

Notable rules: `cff_recover` (control-flow flattening recovery),
`literal_eval` (partial evaluation of literal-only JDK calls),
`string_builder_statements` (statement-level builder-chain recovery),
`store_kill` (dominating-path dead-store kill), `xor_noise`,
`twr_recover` / `string_switch_recover` (decompiler shape restoration).
Full catalog in the [repository README](https://github.com/ejfkdev/cure#rule-catalog).

License: MIT. Zero runtime dependencies.
