# cure

**[English](README.md) | [简体中文](README.zh-CN.md)**

[![CI](https://github.com/ejfkdev/cure/actions/workflows/ci.yml/badge.svg)](https://github.com/ejfkdev/cure/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/cure-cli.svg)](https://crates.io/crates/cure-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Rust MSRV](https://img.shields.io/badge/rust-1.74%2B-orange)

**cure** is a fault-tolerant, semantic-preserving code simplifier and
formatter that turns decompiled code back into something a human would have
written. The engine and CLI are language-agnostic — **Java**,
**TypeScript/JavaScript** (`ts`/`tsx`/`mts`/`cts`/`js`/`jsx`/`mjs`/`cjs`) and
**Python** (`py`/`pyi`, Python 2.7 and 3.x through 3.15) are the backends
implemented today; the pipeline below shows the Java experience. The TS/JS
backend ships the same engine with ECMAScript semantics (f64 / ToInt32
folding domains, ±2^53 exactness, per-file binding/scope resolution for
`var` hoisting and closure-capture guards, optional-chain and destructuring
opacity, ECMA number formatting). The Python backend accepts both grammars —
Python 2 `print`/`exec`/`raise` statements, backticks, `<>`, long and
old-style octal literals, `ur` strings, tuple parameters and
`except E, e`; plus PEP 701 f-strings/t-strings, `match`, walrus, `except*`,
PEP 695 type parameters and `lazy import` — with Python truthiness,
floor-division/power semantics and generator/scope preservation. Python 2
int/long distinctions and `True`/`False` rebinding are kept sound; forms
whose meaning differs between versions stay byte-preserved when ambiguous.

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
  regions are preserved verbatim; everything around them is still simplified.
  Deeply nested input beyond the parser recursion limit is preserved as a
  whole-file verbatim region instead of overflowing the stack
  (`--strict` flips diagnostics into an error exit for CI use).
- **Semantic preservation, verified in three layers** — property testing
  against a toy-language interpreter, differential testing against real
  `javac`/`java` runs, and whole-corpus re-parse/idempotency checks (details
  below). These layers caught real bugs during development; they are not
  decorative.
- **Opt-in source fidelity** — `--preserve-function-source` prints every
  function, class and object literal byte-for-byte (comments, whitespace and
  CRLF included), so `Function.prototype.toString()` matches the original.
  Bindings referenced from preserved regions are frozen; the rest of the
  module is still simplified.
- **Complex TypeScript types** — type aliases with generic/mapped/
  conditional/tuple/template-literal types, interfaces/namespaces/ambient
  declarations, definite assignment (`x!: T`), optional members, generic
  calls (`f<T>()`) and `new C<T>()`, typed function expressions and arrows
  (including object return types and rest parameters), `as`/`satisfies`
  with parenthesized types. Type declarations are preserved verbatim while
  value-level code is still simplified.
- **Modern TS/JS syntax** — decorators (class/member), `accessor`, private
  fields/methods, static blocks, `using`/`await using`, import
  attributes/`defer`/`source`, top-level await, generic functions/arrows
  (including `const` type parameters), `satisfies`/`as const`, `#x in o`,
  RegExp `v` flags and optional chains. Unsupported constructs stay verbatim
  instead of aborting.
- **Python backend (2.7 + 3.x)** — `.py`/`.pyi` parse → simplify → print
  with a fault-tolerant, indentation-aware parser for both grammars. Python 2
  statements and literals (`print`, `exec`, `raise E, V`, backticks, `<>`,
  `1L`, `0777`, `ur"..."`, tuple parameters, `except E, e`, unparenthesized
  comprehension tuples) coexist with modern syntax (PEP 701, t-strings,
  `match`, walrus, `except*`, PEP 695, `lazy import`). Constant folding
  follows Python semantics (exact integers in the i64 domain, floor
  division/modulo, `and`/`or` returning an operand, truthiness); Python 2
  int/long types and `True`/`False` rebinding are preserved, and
  version-dependent forms stay byte-preserved when ambiguous. Dead branches
  containing `yield`/`global`/`nonlocal` are never dropped.
- **92 simplification rules** (66 Java defaults + 1 Java opt-in + 25
  Python; 33 of the Java defaults are language-agnostic engine rules),
  including control-flow flattening recovery, statement-level
  `StringBuilder` chain recovery, XOR-noise removal, and partial evaluation
  ("virtual execution") of literal-only JDK calls.
- **Scales to real codebases** — the full OpenJDK source corpus
  (371,674 files, 4.7 GB) processes in ~15 s on an 18-core laptop, with
  0 parse failures and 0 panics. On the JS/TS side, a 13 MB minified
  bundle runs in ~5 s (was ~32 s before incremental effect caches), and
  4,358 npm files process in ~10.5 s; token positions are `u32`, keeping
  each token at 48 bytes.
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

# or a prebuilt binary via cargo-binstall
cargo binstall cure-cli

# or download a raw (uncompressed) binary from GitHub Releases:
# assets are named cure-vX.Y.Z-<target>, no archive wrapper
# https://github.com/ejfkdev/cure/releases

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

```bash
echo 'const x = 1 + 2; console.log(x);' | cure --lang ts -
# const x = 3;
# console.log(3);
```

```bash
printf 'x = 1 + 2\nprint(x)\n' | cure --lang py -
# x = 3
# print(3)
```

```text
cure [选项] <文件>... | <目录> | -
```

| Option | Meaning |
|--------|---------|
| `-o, --output <path>` | output file (single input) or output root (directory) |
| `-w, --write` | rewrite inputs in place |
| `--check` | dry run: exit code 2 if anything is rewritable, write nothing |
| `-j, --threads <N>` | worker threads (default: CPU cores × 1.5) |
| `--format-only` | format only, no simplification |
| `--ext <list>` | restrict to given extensions (comma/space separated) |
| `--lang <name>` | force a backend: `java`, `ts`/`js` (TS/JS) or `python`/`py`; by default inferred from the file extension |
| `--copy-other` | directory mode: copy non-source files verbatim |
| `--diff` | print unified diffs of every rewritten file |
| `--stats` / `--report` | aggregate / per-file rewrite statistics |
| `--disable <rule>` | disable a named rule (repeatable) |
| `--dead-code` | opt-in: drop private methods referenced nowhere in the unit |
| `--preserve-function-source` | TS/JS: print functions/classes/object literals byte-for-byte (`Function.prototype.toString`); preserved regions are not simplified |
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
| 4. Decompiled apps | 144,592 files from 5 real-world Android APKs run through [ddc](https://github.com/ejfkdev/ddc), then cure (full + `--dead-code`) | same invariants + javac differential: every package whose ddc output compiles must still compile after cure |
| 5. ECMAScript conformance | [Test262](https://github.com/tc39/test262), six layers: (a) sync original vs cured in fresh `node` vm contexts, exception status + stdout compared (34,007 executable cases); (b) **async** cases executed with `$DONE`/microtask/timer draining (5,423 executed); (c) **module** cases linked and evaluated as ESM with a filesystem fixture loader (358 executed); (d) **compile-only on all 53,910 files** including async/module/negative; (e) **print round-trip**: parse → print (no simplify) → parse, ASTs structurally equal ignoring `Paren`/literal spelling/`Group` expansion (49,159 checked); (f) **simplify idempotency**: cure twice over the same corpus, byte-identical output (49,159 checked) | ECMAScript semantics: f64/ToInt32 domains, ±2^53, TDZ, Annex B hoisting, eval scoping, destructuring, generator/`yield` grammar, sparse arrays, async/await ordering, ESM linking/live bindings; printer/paren/decorator drift; fixed-point convergence |
| 6. Real-world JS/TS | 4,358 `.js/.mjs/.cjs` files (~568 MB) plus 1,334 `.ts/.tsx/.mts/.cts` files from a global npm install (undici, zod, minisearch, pnpm, yarn, vite-plus, Lark CLI, bytenode, …): every valid file must still compile after cure; a second cure pass must produce byte-identical output. Both corpora run at **0 parser diagnostics** and **0 second-pass differences** in default and preserve modes, and every cured file parses cleanly with Oxc (`oxfmt`, 0 syntax errors) | real-package syntax (minified bundles, top-level await, import attributes), complex type syntax, idempotency under module-level constant propagation |
| 7. Randomized JS differential | Deterministic PRNG generates valid programs mixing declarations, shadowing, `var` hoisting, closures, loops, ternaries, short circuits and templates; original vs cured run under `node` (stdout + exit code) and a second cure must be byte-identical (`CURE_FUZZ_CASES` scales the sample) | rule-combination semantics and first/second-pass convergence; 5,000 cases used during development |
| 8. Python (2.7 + 3.13–3.15) | CPython 2.7 stdlib (1,609 UTF-8 sources), a 4,135-file Python 2 package corpus (Django 1.11, Twisted 20.3, pip 20.3, setuptools 44, requests, six, PyYAML), CPython 3.13/3.14/3.15 stdlibs (2,188 / 2,288 / 2,456 sources), 18,128 Python 3.14 site-packages files plus 1,083 / 422 files from the 3.13 / 3.15 sites: parse → print compared with that interpreter's `ast.dump`; a second cure must be byte-identical; every simplified file compiled by the same interpreter; curated programs executed | **0** parser diagnostics in all corpora; **32,289** AST-identical round-trips (5,736 Python 2 + 26,553 Python 3; 20 intentionally invalid/foreign fixtures excluded), **35,548/35,548** idempotent with 0 drift, **32,274** compile checks with **0 regressions** (35 originals were already uncompilable); curated stdout/exit codes match |

`cargo test` runs 523 tests. The Test262 semantic differential reaches
**33,963/34,007 (99.87%) identical runs** in default mode; the only 44
excluded cases compare `Function.prototype.toString` **source text** (cure
reformats function bodies) or use arrow functions as computed-property keys,
which relies on that same text. With `--preserve-function-source` the same
layer passes **34,007/34,007 (100%)** — function/class/object-literal source
is emitted byte-for-byte, so the excluded cases now compare equal too. The async layer executes **5,423/5,423 cases identically
(0 mismatch)**, the module layer **358/358 (0 mismatch)**, and the compile
layer keeps **49,165/49,165 valid files compiling after cure** (0 broken;
modules via `vm.SourceTextModule`); the print round-trip and simplify
idempotency layers both pass **49,159/49,159** files parameterized by their
Test262 script/module goal. Sampled versions of all six layers are standing
tests (`cargo test -p cure-ts-simplify --test
t262_sem|t262_async|t262_module|t262_compile|t262_roundtrip|t262_idem`); they skip automatically when
`/tmp/test262` or `node` is unavailable (`T262_ROOT` / `T262_SAMPLE` override). Layers 1–2 caught three real semantic/parse bugs
during development (`x && true` mis-fold, `((Cast) x).method()` postfix loss,
`new` treated as a type name).

The Python differential spans CPython 2.7 and 3.13–3.15. The 2.7 standard
library (1,609 UTF-8 sources) and a 4,135-file Python 2 package corpus
(Django 1.11, Twisted 20.3, pip, setuptools 44, requests, six, PyYAML) run at
**0 parser diagnostics**; 1,607 + 4,129 parse → print round-trips are
AST-identical under CPython 2.7 (the rest are deliberately invalid/foreign
fixtures), all 5,744 files are idempotent, and every simplified file that
CPython 2.7 could compile before still compiles (1,600 + 4,129; 15
originals were already invalid). The 3.13/3.14/3.15 stdlibs and the 3.14
site-packages tree add 26,553 AST-identical round-trips with 0 diagnostics,
0 idempotency drift and 0 compile regressions; the 3.6 and 3.8 stdlibs are
also clean (1,578 and 1,661 files). Across all corpora that is **32,289 AST
comparisons, 35,548 idempotent files and 32,274 compilations, with 0
regressions**; the simplifier performs **179,873 rewrites** (7,833 Python 2
stdlib, 6,763 Python 2 packages, 12,098 Python 3.15 stdlib, 111,340 Python
3.14 site-packages; the remaining 3.13/3.14/3.6/3.8 corpora add 41,839),
while docstrings and version-visible types stay intact.
Curated Python 2 and 3 programs (print/exec/raise forms,
backticks, `<>`, long/int types, tuple parameters, decorators with
triple-quoted arguments) execute side-by-side with identical stdout and exit
codes. Non-UTF-8 source files (PEP 263 encodings other than UTF-8) are
currently skipped by the CLI; the Python 2 stdlib corpus above was transcoded
to UTF-8 for testing.

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

**Engine (language-agnostic, 33):** `paren_removal`, `const_condition`,
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
`multi_use_copy`, `trailing_return`,
`inverse_assign_pair` (`x+=K;x-=K` register-noise pairs), `dead_store`,
`assign_back_fold` (`x=e; x=x+K` register accumulator folded into the decl), `dead_decl`, double_neg_fold (fold `-(-lit)`), block_flatten (unnecessary nested blocks collapse), opt-in `unreachable_after_terminal`.

**Java-specific (33):** `cast_simplify`, `self_compare`,
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
`try_unwrap_rethrow` (`catch (E e) { throw e; }` unwrapping — the most frequent
fernflower leftover, ≥23 sites in a 66-file corpus),
new_string_char_array_fold (`new String(CHAR_ARRAY)`), empty_finally_strip, try_unwrap_no_catch, static_array_index_fold (literal array indexing), const_method_inline (single-return helper inlining — string decrypters), trailing_continue (label-aware tail continue removal), static_exec, url_decode_fold, trailing_assign_return, dead_pure_call, ctor_delegation_hoist, trailing_else_flatten, trailing_if_continue, const_field_propagate, noop_private_call.

**Python (25):** `py_const_fold` (integer/boolean/string literals, string
`+`/`*`, comparisons, truthiness, `and`/`or` operand-return semantics;
long/float/exotic literals are never folded), `py_call_fold` (literal `str`
methods — upper/lower/swapcase/capitalize/strip/replace/startswith/endswith/
join — plus shadow-safe `len`/`ord`/`chr`/`str`/`int`/`abs`/`bool`/`min`/
`max`), `py_subscript_fold` (literal string/list/tuple index and slice),
`py_const_if`, `py_if_ternary` (single-assignment or single-return branches →
conditional expression), `py_if_return` (`if c: return a` followed by
`return b` → `return a if c else b`), `py_if_empty` (`else: pass` removal and
pass-only branch inversion), `py_nested_if` (`if a: if b:` → `if a and b:`),
`py_else_after_terminal` (drop `else` when the `if` body always terminates),
`py_bool_cond` (`True if c else False` → `bool(c)` and its inverse),
`py_ternary_not` (`a if not c else b` → `b if c else a`), `py_not_is_in`
(`not (x is y)` / `not (x in y)` → `x is not y` / `x not in y`),
`py_paren_removal` (redundant parentheses around atoms), `py_dead_store`
(adjacent pure-literal overwrite), `py_dead_assign` (function-scope liveness:
an unread local with a pure RHS is removed, an effectful RHS survives as an
expression), `py_dead_expr` (bare numeric/`None`/`...` expressions),
`py_str_accum` (`s = "a"; s += "b"` → `s = "ab"`), `py_assign_back_fold`
(`x = 2; x = x + 3` → `x = 5` for pure int/string chains), `py_return_none`
(`return None` → `return`), `py_trailing_return` (drop an implicit trailing
`return`), `py_trailing_continue` (drop a loop-tail `continue`),
`py_empty_finally` (`finally: pass`), `py_assert_true` (constant-true
assertions), `py_unreachable` and `py_pass_cleanup`. Docstrings, Python 2
int/long types, shadowed `True`/`False`/builtins (including `exec` and
`import *`), locals/vars/dir/eval introspection and version-ambiguous forms
are guarded.

## Performance

Release build, 18-core Apple Silicon laptop, warm page cache, default
threads (cores × 1.5):

| Workload | Files | Time | Peak RSS |
|---|---|---|---|
| Single file (stdin → stdout) | 1 | <10 ms | ~2 MiB |
| Vendored test corpora | 3,999 | 0.6 s | ~110 MiB |
| CPython 2.7 stdlib, output tree written | 1,609 | 0.18 s | ~85 MiB |
| Python 2 package corpus, output tree written | 4,135 | 0.38 s | ~75 MiB |
| CPython 3.15 stdlib, output tree written | 2,456 | 0.35 s | ~150 MiB |
| Python 3.14 site-packages, output tree written | 18,128 | 1.6 s | ~285 MiB |
| OpenJDK full source corpus, `--check` | 371,674 / 4.70 GB | 14.8 s | ~430 MiB |
| OpenJDK full source corpus, output tree written | 371,674 / 4.70 GB | 27.5 s | ~430 MiB |

The full-corpus runs execute parse → simplify → print for every file with
**0 parse failures and 0 panics**. On the 3,999-file vendored corpus,
`--stats` classifies: 1,032 structurally simplified / 1,943 format-only /
46 untouched (lines −53.4 %, nodes −10.3 %, decisions −13.9 %).

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
cure-python-ast    Python NodeData + PyId arena
cure-python-parser fault-tolerant indentation-aware parser (Python 2.7–3.15)
cure-python-print  canonical 4-space Python printer
cure-python-simplify Python constant/control-flow rules over cure-engine
cure-cli           the `cure` binary: parallel pipeline, cross-platform
                   batch I/O (io_uring / thread pool), directory mode
```

The `Lang` trait is the language boundary. Numeric/string folding semantics
live behind language hooks (`fold_lit_bin` / `fold_lit_cmp` / `fold_lit_neg` /
`reassoc_delta`) — the engine hardcodes no language's arithmetic. Each
language frontend provides its own AST + parser + printer + rule pack on top
of `cure-engine` + `cure-tree`; the TS/JS and Python backends are two such
frontends (see also the ~100-line toy language in `cure-tree/tests/kit.rs`).

## Status & roadmap

- ✅ Java backend at production robustness: 371k-file corpus, 0 failures
- ✅ TS/JS backend: Test262 six layers, 0 parser diagnostics on npm corpora
- ✅ Python 2.7 + 3.x backend: 35,548 stdlib/site-package files, 0
  diagnostics, 0 idempotency drift, 0 compile regressions
- ✅ 12 vendored Java corpora, three-layer verification, 523 tests
- 🔜 Restoration rules V2 (reflective `addSuppressed` shapes, nested
  two-resource try-with-resources)
- 🔜 Further Python rule-pack work (local propagation, decompiler-shape
  restoration)

## License

MIT — see [LICENSE](LICENSE). Release history in [CHANGELOG.md](CHANGELOG.md).
