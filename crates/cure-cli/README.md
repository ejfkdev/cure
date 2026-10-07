# cure-cli

[![crates.io](https://img.shields.io/crates/v/cure-cli.svg)](https://crates.io/crates/cure-cli)
[![CI](https://github.com/ejfkdev/cure/actions/workflows/ci.yml/badge.svg)](https://github.com/ejfkdev/cure/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/ejfkdev/cure/blob/main/LICENSE)

`cure` — the command-line front end of the
[cure](https://github.com/ejfkdev/cure) simplification engine: a
fault-tolerant, semantic-preserving Java simplifier/formatter that turns
decompiled code back into something a human would have written.

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
```

Highlights:

- **Fault-tolerant** — parse errors never abort a run; broken regions are
  preserved verbatim while everything around them is simplified
  (`--strict` turns them into an error exit for CI).
- **Directory mode** — recursive, parallel (dynamic work queue, threads =
  CPU cores × 1.5), mirrors the tree into `<dir>-cure-out/`; the 371,674-file OpenJDK
  corpus (4.70 GB) in ~15 s / ~430 MiB peak RSS on an 18-core laptop
  (`--check`), 0 failures.
- **Cross-platform batch I/O** — optional Linux `io_uring` backend
  (`--features io-uring`; read syscalls 6,977 → 10 on a 3,483-file trace);
  macOS/Windows thread pool. Default build: zero runtime dependencies.
- Dry runs (`--check`), diffs (`--diff`), stats (`--stats`/`--report`),
  rule toggles (`--disable`), in-place mode (`-w`), thread control (`-j`).

Full documentation — rule catalog, verification methodology, performance
notes, and the 中文版 — in the
[repository README](https://github.com/ejfkdev/cure).

License: MIT.
