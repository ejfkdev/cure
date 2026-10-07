# cure-java-print

[![crates.io](https://img.shields.io/crates/v/cure-java-print.svg)](https://crates.io/crates/cure-java-print)
[![Documentation](https://docs.rs/cure-java-print/badge.svg)](https://docs.rs/cure-java-print)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/ejfkdev/cure/blob/main/LICENSE)

Canonical Java source printer for the [cure](https://github.com/ejfkdev/cure)
pipeline: renders a [`cure-java-ast`](https://crates.io/crates/cure-java-ast)
tree back to text in clean, idiomatic formatting.

```rust
use cure_java_parser::parse;      // sibling crate: build the AST
use cure_java_print::print_unit;

let out = parse("class A{int m(){int a=foo();int b=a;return b;}}");
println!("{}", print_unit(&out.ast, &out.unit));
// class A {
//     int m() {
//         return foo();
//     }
// }
```

## What it guarantees

- **Output re-parses.** Whatever the AST contains (including rules that moved
  nodes across the tree), the printed text is valid input for
  [`cure-java-parser`](https://crates.io/crates/cure-java-parser) — this
  round-trip is asserted over the full test corpus in CI.
- **Idempotent.** Parse → print → parse → print produces identical text.
- **Raw regions verbatim.** Unparseable input regions survive as `Raw` nodes
  and are printed exactly as they were read.
- **Java-aware precedence.** Minimal parentheses: keeps parens only where
  operator precedence, associativity, or cast/switch-expression receiver
  position requires them for correctness.

## Formatting style

4-space indent, K&R braces, single space after keywords, no line wrapping
(line breaking is a policy decision the host tool can apply afterwards).

Entry points: `print_unit(ast, unit)` for whole compilation units,
`print(ast, root)` for any subtree, `ty_str(ty)` for type rendering.

License: MIT. Zero runtime dependencies.
