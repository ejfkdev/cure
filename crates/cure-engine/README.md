# cure-engine

[![crates.io](https://img.shields.io/crates/v/cure-engine.svg)](https://crates.io/crates/cure-engine)
[![Documentation](https://docs.rs/cure-engine/badge.svg)](https://docs.rs/cure-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/ejfkdev/cure/blob/main/LICENSE)

The language-agnostic core of [cure](https://github.com/ejfkdev/cure): a
semantic-preserving code simplification engine — pattern matching, tree
rewrites, effect analysis, and a fixed-point convergence driver.

This crate knows **nothing about Java**. It operates on any tree whose
language implements the [`Lang`] trait: you provide an arena of nodes with
stable ids, name resolution, and effect queries; the engine provides rule
application, work invalidation, and iteration to a fixed point.

## When to use it standalone

You are building a simplifier / normalizer / deobfuscator for **your own
language or AST** and want the engine machinery (not the Java rules):

- rule dispatch + fixed-point driver (`simplify` — re-runs until no rule
  applies; pings-pongs are bounded)
- edit invalidation: every accepted edit invalidates exactly the affected
  ancestor chains and re-analysis caches
- region event index: use/write/shadow events over stable name keys, with
  binary-search queries for kill-analysis rules (dead stores, dominating-path
  logic lives here)
- cost model: rewrites only apply when they provably reduce node cost
- effect analysis (pure / read / write / opaque) gating every transform

The reference implementation of the trait is
[`cure-java-ast`](https://crates.io/crates/cure-java-ast) (arena AST with
u32 node ids, interned name keys).

## Entry points

```rust
use cure_engine::{Config, Lang, Report, Rule, simplify};

/// Run a rule set over `root` until fixed point.
pub fn simplify<L: Lang>(
    lang: &mut L,
    root: L::Id,
    rules: &[Box<dyn Rule<L>>],
    cfg: &Config,
) -> Report;
```

Rules are pattern → rewrite functions receiving a `RewriteCtx` (node access,
children, effect queries). `Config` toggles individual rules and the
optional/dead-code tiers.

The Java front end lives in the sibling crates — see the
[pipeline example](https://github.com/ejfkdev/cure#usage) in the repository
README:

| Crate | Role |
|---|---|
| [cure-java-ast](https://crates.io/crates/cure-java-ast) | arena AST, `Lang` implementation |
| [cure-java-parser](https://crates.io/crates/cure-java-parser) | fault-tolerant Java parser |
| [cure-java-print](https://crates.io/crates/cure-java-print) | canonical printer/formatter |
| [cure-java-simplify](https://crates.io/crates/cure-java-simplify) | the Java rule pack |

License: MIT. Zero runtime dependencies.
