# cure-cli

[![crates.io](https://img.shields.io/crates/v/cure-cli.svg)](https://crates.io/crates/cure-cli)
[![CI](https://github.com/ejfkdev/cure/actions/workflows/ci.yml/badge.svg)](https://github.com/ejfkdev/cure/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/ejfkdev/cure/blob/main/LICENSE)

`cure` — the command-line front end of the
[cure](https://github.com/ejfkdev/cure) simplification engine: a
fault-tolerant, semantic-preserving code simplifier/formatter that turns
decompiled code back into something a human would have written. The CLI is
language-agnostic (**Java, TypeScript/JavaScript and Python are the backends
implemented today**) — a new language registers in `LANG_BACKENDS`
(extensions + pipeline function) and inherits traversal, parallelism,
`--check`/`--diff`/`--stats`, and the cross-platform batched I/O for free.

```bash
cargo install cure-cli
```

```bash
echo 'class A{int m(){int a=foo();int b=a;return b;}}' | cure -
# class A {
#     int m() {
#         return foo();
#     }
# }

echo 'const x = 1 + 2; console.log(x);' | cure --lang ts -
# const x = 3;
# console.log(3);

printf 'x = 1 + 2\nprint(x)\n' | cure --lang py -
# x = 3
# print(3)
```

Highlights:

- **Fault-tolerant** — parse errors never abort a run; broken regions are
  preserved verbatim while everything around them is simplified
  (`--strict` turns them into an error exit for CI).
- **Python 2.7 + 3.x** — `.py`/`.pyi` back end: one indentation-aware
  fault-tolerant parser for both grammars (Python 2 print/exec/raise
  statements, backticks, `<>`, long/old-octal literals, `ur` strings, tuple
  parameters; PEP 701 f-strings, t-strings, `match`, walrus, `except*`, PEP
  695, `lazy import`), Python-semantics constant folding that preserves
  Python 2 int/long types and `True`/`False` rebinding, and generator/scope
  guards. A 12-rule pack includes literal string/builtin/subscript folding,
  if/else-to-conditional, nested-if merging, else-after-terminal removal,
  dead stores/expressions and `return None` cleanup. Verified over 35,548
  stdlib/site-package files with 0 diagnostics, 0 idempotency drift and 0
  compile regressions.
- **Directory mode** — recursive, parallel (dynamic work queue, threads =
  CPU cores × 1.5), mirrors the tree into `<dir>-cure-out/`; the 371,674-file OpenJDK
  corpus (4.70 GB) in ~15 s / ~430 MiB peak RSS on an 18-core laptop
  (`--check`), 0 failures.
- **Cross-platform batch I/O** — optional Linux `io_uring` backend
  (`--features io-uring`; read syscalls 6,977 → 10 on a 3,483-file trace);
  macOS/Windows thread pool. Default build: zero runtime dependencies.
- Dry runs (`--check`), diffs (`--diff`), stats (`--stats`/`--report`),
  rule toggles (`--disable`), in-place mode (`-w`), thread control (`-j`).
- **Simplification-quality metrics** (`--stats`): AST nodes, decision
  points, max nesting — before vs. after. Untouched by formatting, they
  separate real simplification from reformatting; per-file classification
  (structurally simplified / format-only / untouched) surfaces files no
  rule reached.

Full documentation — rule catalog, verification methodology, performance
notes, and the 中文版 — in the
[repository README](https://github.com/ejfkdev/cure).

License: MIT.
