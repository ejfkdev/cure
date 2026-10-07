# cure

**[English](README.md) | [简体中文](README.zh-CN.md)**

[![CI](https://github.com/ejfkdev/cure/actions/workflows/ci.yml/badge.svg)](https://github.com/ejfkdev/cure/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/cure-cli.svg)](https://crates.io/crates/cure-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Rust MSRV](https://img.shields.io/badge/rust-1.74%2B-orange)

**cure** is a fault-tolerant, semantic-preserving code simplifier and
formatter that turns decompiled code back into something a human would have
written. The engine and CLI are language-agnostic — **Java is the backend
implemented today**; the pipeline below shows the Java experience.

```java
// decompiled input                          // after cure
class B {                                     class B {
    void n() {                                    void n() {
        int x = 1 + 2;                                System.out.println("y1");
        boolean c = (x > 2) && true;              }
        if (c) { System.out.println("y"+1); }  }
        return;
    }
}
```

It is **not** dead-code elimination and **not** a minifier. It normalizes
local expression and control-flow *shape*: folds constants and opaque
predicates, inlines copies, dissolves compiler artifacts (register noise,
builder chains, state-machine control-flow flattening), and prints the result
in clean idiomatic formatting — while guaranteeing the program still means
the same thing.

## Highlights

- **Fault-tolerant by design** — syntax errors never abort a run. Broken
  regions are preserved verbatim; everything around them is still simplified
  (`--strict` flips this to an error exit for CI use).
- **Semantic preservation, verified in three layers** — property testing
  against a toy-language interpreter, differential testing against real
  `javac`/`java` runs, and whole-corpus re-parse/idempotency checks (details
  below). These layers caught real bugs during development; they are not
  decorative.
- **55 simplification rules** (32 language-agnostic + 23 Java-specific),
  including control-flow flattening recovery, statement-level
  `StringBuilder` chain recovery, XOR-noise removal, and partial evaluation
  ("virtual execution") of literal-only JDK calls.
- **Scales to real codebases** — the full OpenJDK source corpus
  (371,674 files, 4.7 GB) processes in ~15 s on an 18-core laptop, with
  0 parse failures and 0 panics.
- **Cross-platform I/O backends** — Linux builds can use batched
  `io_uring` (3,483-file syscall trace: read calls 6,977 → 10, syscall time
  −53%); macOS/Windows use a tuned thread pool. The default build stays
  zero-runtime-dependency.
- **Zero runtime dependencies** — only `std` (one optional feature-gated
  Linux-only crate).

## Install

```bash
# from crates.io
cargo install cure-cli

# or a prebuilt binary from GitHub Releases
curl -L https://github.com/ejfkdev/cure/releases/latest/download/cure-<target>.tar.gz | tar xz

# or from source
git clone https://github.com/ejfkdev/cure
cargo install --path crates/cure-cli
```

## Usage

```bash
echo 'class A{int m(){int a=foo();int b=a;return b;}}' | cure -
```
```java
class A {
    int m() {
        return foo();
    }
}
```

```text
cure [选项] <文件.java>... | <目录> | -
```

| Option | Meaning |
|--------|---------|
| `-o, --output <path>` | output file (single input) or output root (directory) |
| `-w, --write` | rewrite inputs in place |
| `--check` | dry run: exit code 2 if anything is rewritable, write nothing |
| `-j, --threads <N>` | worker threads (default: CPU cores × 1.5) |
| `--format-only` | format only, no simplification |
| `--ext <list>` | restrict to given extensions (comma/space separated) |
| `--copy-other` | directory mode: copy non-source files verbatim |
| `--diff` | print unified diffs of every rewritten file |
| `--stats` / `--report` | aggregate / per-file rewrite statistics |
| `--disable <rule>` | disable a named rule (repeatable) |
| `--dead-code` | opt-in: drop private methods referenced nowhere in the unit |
| `--strict` | exit 1 on any parse error |

Directory mode recurses, processes files in parallel (dynamic work queue,
naturally load-balanced against clustered large files), and mirrors the tree
into `<dir>-cure-out/` by default.

`cure` with no arguments prints the help; `cure rules` lists every rule
name (what `--disable` takes); help is bilingual — `CURE_LANG=zh|en` forces,
otherwise the locale is auto-detected. Full option reference: `cure --help`.

## How correctness is verified

| Layer | Method | Catches |
|---|---|---|
| 1. Property testing | Random toy programs (80 seeds); an interpreter compares return values **and** call/side-effect sequences** before vs. after | engine-level semantics: evaluation order, short-circuiting, side effects; fixed-point idempotency |
| 2. Differential testing | Original and simplified versions each compiled by `javac` and run; stdout / exit code / exception signature must match byte-for-byte (12 families: overflow, NaN, division by zero, StringBuilder, iterators, boxing, short-circuit side effects, …) | full Java-language semantics |
| 3. Corpora | 3,999 vendored real-world files in 12 auto-discovered suites + 5,596-file external mega-corpus + the 371k-file OpenJDK source corpus | robustness (no panics), self-consistency (output re-parses cleanly), idempotency (second pass = 0 rewrites) |

`cargo test` runs 170 tests. Layers 1–2 caught three real semantic/parse bugs
during development (`x && true` mis-fold, `((Cast) x).method()` postfix loss,
`new` treated as a type name).

### Adversarial case studies

All of the following round-trip through `javac` differential testing with
byte-identical output and preserved side-effect call sequences:

- **Combined obfuscation gauntlet** — cross-method string decrypter
  (string-array + Base64 helper) × control-flow flattening state machine ×
  register noise × opaque predicates × StringBuilder statement chains: 24
  rewrites, non-blank lines 87 → 32 (−64%).
- **Control-flow flattening recovery** (`cff_recover`) —
  obfuscator.io/Allatori-style `while(true){switch(s)}` state machines
  recovered into structured control flow: linear chains, diamonds, loops,
  `default: return` exits. Unsafe shapes (state variable used after the
  loop, unreachable cases) are conservatively rejected.
- **Hard obfuscation** — multi-layer constant hiding, nested opaque
  predicates, SB × `new String` × `valueOf` × scattered constants: 44
  rewrites, non-blank lines 76 → 33 (−57%).
- **Double toolchain** — obfuscated source → `javac` → d8 → ddc (register
  artifacts) → cure: output matches the ddc and original versions.

## Rule catalog

**Engine (language-agnostic, 32):** `paren_removal`, `const_condition`,
`boolean_return`, `if_to_ternary`, `if_assign_ternary`, `if_else_empty`,
`bool_compare`, `double_not`, `bool_not_fold`, `bool_short_circuit`,
`not_compare`, `ternary_fold`, `ternary_bool`, `ternary_bool_op`,
`const_fold_bin` (Java wrap-around i32/i64 + shift masking), `cmp_const_fold`
(`1 < 2 → true`, breaks opaque predicates), `self_assign`, `arith_identity`,
`arith_zero`, `bit_identity`, `arith_reassoc` (xor/add-sub reassoc; string
concat constant merging `("a"+x)+"b"+"c" → "a"+x+"bc"`), `local_propagation`,
`decl_assign_merge`, `assign_propagation` (copy-assignment inlining with
block-scope declaration anchoring), `store_kill` (distant dead stores /
register pre-declarations killed on the dominating path only),
`multi_use_copy`, `trailing_return`, `trailing_continue` (label-aware),
`inverse_assign_pair` (`x+=K;x-=K` register-noise pairs), `dead_store`, `store_kill`, double_neg_fold (fold `-(-lit)`), block_flatten (unnecessary nested blocks collapse), opt-in `unreachable_after_terminal`.

**Java-specific (23):** `cast_simplify`, `self_compare`,
`string_builder_fold`, `box_unbox_chain`, `iterator_to_for_each`,
`while_iterator_to_for_each`, `new_string_fold`, `loop_head_break` (incl.
do-while shapes), `concat_value_of_drop`, `string_builder_statements`
(statement-level SB chain recovery across reassignments and fresh variables),
`xor_noise` (XOR operands are integral in Java, so side-effecting calls can
be unwrapped: `(mark(5) ^ 0x5A) ^ 0x5A → mark(5)`), `str_len_fold`,
`literal_eval` (partial evaluator: `"HelloWorld".substring(0,5)`,
`String.format`, `Integer.parseInt`, `Math.abs/max/min`,
`Character.isXxx/toXxx`, literal array indexing, cast literals — folds only
when evaluation succeeds, never on the exception path),
`base64_new_string_fold`, `cff_recover` (control-flow flattening recovery),
`twr_recover` + `string_switch_recover` (decompiler-shape restoration),
new_string_char_array_fold (`new String(CHAR_ARRAY)`), empty_finally_strip, try_unwrap_no_catch, static_array_index_fold (literal array indexing), const_method_inline (single-return helper inlining — string decrypters), trailing_continue (label-aware tail continue removal).

## Performance

Release build, 18-core Apple Silicon laptop, warm page cache, default
threads (cores × 1.5):

| Workload | Files | Time | Peak RSS |
|---|---|---|---|
| Single file (stdin → stdout) | 1 | <10 ms | ~2 MiB |
| Vendored test corpora | 3,999 | 0.6 s | ~110 MiB |
| OpenJDK full source corpus, `--check` | 371,674 / 4.70 GB | 14.8 s | ~430 MiB |
| OpenJDK full source corpus, output tree written | 371,674 / 4.70 GB | 27.5 s | ~430 MiB |

The full-corpus runs execute parse → simplify → print for every file with
**0 parse failures and 0 panics**. On the 3,999-file vendored corpus,
`--stats` classifies: 951 structurally simplified / 2,025 format-only /
47 untouched (nodes −4.7 %, decisions −5.0 %).

- Parallelism: dynamic work queue, default threads = cores × 1.5 (measured
  −7% vs 1:1 on heterogeneous P/E cores; CPU and RSS neutral).
- Linux batched I/O (`--features io-uring`): three-phase batched
  open→read→close; read syscalls 6,977 → 10 on a 3,483-file trace, syscall
  time −53%.
- macOS/Windows I/O: thread-pool per-file reads, measured ~1 GB/s aggregate
  (near the per-file syscall ceiling).

## Architecture

```text
cure-engine        language-agnostic core: patterns, rewrites, passes,
                   fixed-point driver, cost model, literal-folding hooks
cure-tree          generic arena-tree toolkit every language reuses:
                   inline ChildList, name interning + node keys, region-event
                   index (B3 lazy rebuild), effect table rebuild
cure-java-ast      Java NodeData + signatures; implements the Lang trait
                   on top of cure-tree
cure-java-parser   fault-tolerant Java lexer/parser (byte-borrowed tokens,
                   zero-copy source)
cure-java-print    canonical Java printer/formatter
cure-java-simplify Java rule pack + facade over cure-engine
cure-cli           the `cure` binary: parallel pipeline, cross-platform
                   batch I/O (io_uring / thread pool), directory mode
```

The `Lang` trait is the language boundary. Numeric/string folding semantics
live behind language hooks (`fold_lit_bin` / `fold_lit_cmp` / `fold_lit_neg` /
`reassoc_delta`) — the engine hardcodes no language's arithmetic. A second
language frontend provides its own AST + parser + printer + rule pack on top
of `cure-engine` + `cure-tree` (see the ~100-line toy language in
`cure-tree/tests/kit.rs`).

## Status & roadmap

- ✅ Java backend at production robustness: 371k-file corpus, 0 failures
- ✅ 12 vendored corpora, three-layer verification, 170 tests
- 🔜 Restoration rules V2 (reflective `addSuppressed` shapes, nested
  two-resource try-with-resources)
- 🔜 A second language to validate the `Lang` trait boundary

## License

MIT — see [LICENSE](LICENSE). Release history in [CHANGELOG.md](CHANGELOG.md).
