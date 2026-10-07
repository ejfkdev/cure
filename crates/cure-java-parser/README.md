# cure-java-parser

[![crates.io](https://img.shields.io/crates/v/cure-java-parser.svg)](https://crates.io/crates/cure-java-parser)
[![Documentation](https://docs.rs/cure-java-parser/badge.svg)](https://docs.rs/cure-java-parser)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/ejfkdev/cure/blob/main/LICENSE)

Fault-tolerant Java source parser for the [cure](https://github.com/ejfkdev/cure)
pipeline: produces a [`cure-java-ast`](https://crates.io/crates/cure-java-ast)
arena tree plus diagnostics — a parse error never aborts the file.

## Why fault tolerance matters

Real-world Java (especially *decompiled* Java) is full of constructs a strict
parser rejects: novel preview syntax, broken regions from partial
decompilation, encoding oddities. This parser keeps going: unparseable
regions become `Raw` nodes that carry the original text verbatim, and
everything around them still lands in the AST. Downstream consumers decide
what to do with the errors.

```rust
use cure_java_parser::parse;

let src = "class A { void m() { int x = 1 + ; return x; } }"; // note the stray `;`
let out = parse(src);

// The file is never lost:
assert!(!out.errors.is_empty()); // …but the parse diagnostics are reported
// …and the recoverable parts are still a real AST:
let printed = cure_java_print::print_unit(&out.ast, &out.unit);
```

## Properties

- **One pass, byte-borrowed.** Tokens borrow the source (`&str` slices);
  the common no-escape case does zero allocation in the lexer. Unicode
  escapes (`\uXXXX`, JLS 3.3, including `\uu+` forms) are decoded in a
  zero-copy preprocessing step exactly like `javac` does — before lexing.
- **Error recovery.** Diagnostics carry line:column; broken regions become
  `Raw { text }` nodes preserved into output.
- **Coverage.** Modern syntax through current preview features: records,
  sealed types, pattern matching (incl. record deconstruction patterns),
  switch expressions/arrow labels, text blocks, `var`, instanceof binding,
  lambda/method references, modules (`module-info`), labeled breaks,
  annotations in all JSR-308 positions.
- **Battle-tested.** The full OpenJDK source corpus (371,674 files, JDK 6–28)
  parses with zero hard failures; 12 vendored real-world corpora (~4,000
  files: google-java-format, checkstyle, javaparser, spoon, PMD test suites)
  run in CI.

## Output

```rust
pub struct ParseOutcome {
    pub ast: JavaAst,             // arena: executable bodies
    pub unit: CompilationUnit,    // signatures: package/imports/types/members
    pub errors: Vec<ParseError>,  // recoverable diagnostics (line:column)
}
```

Print the result with
[`cure-java-print`](https://crates.io/crates/cure-java-print), rewrite it
with [`cure-java-simplify`](https://crates.io/crates/cure-java-simplify) —
see the [pipeline example](https://github.com/ejfkdev/cure#usage) in the
repository README.

License: MIT. Zero runtime dependencies.
