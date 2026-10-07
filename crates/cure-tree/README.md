# cure-tree

[![crates.io](https://img.shields.io/crates/v/cure-tree.svg)](https://crates.io/crates/cure-tree)
[![Documentation](https://docs.rs/cure-tree/badge.svg)](https://docs.rs/cure-tree)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/ejfkdev/cure/blob/main/LICENSE)

Generic arena-tree toolkit for [cure](https://github.com/ejfkdev/cure)
language frontends: the infrastructure every language AST would otherwise
reimplement. **A new language frontend = this kit + its own NodeData /
parser / printer / rule pack.**

## What's in the kit

| Component | What it does |
|---|---|
| [`ChildList<Id>`] | children container with ≤3-child inline storage (zero heap allocation for the vast majority of AST nodes; spills to heap for Block/Call fan-outs) |
| [`NameTable`] | unit-level `name ↔ u32` interning (append-only) + per-node name-key table with incremental extension — makes `Lang::NameKey` comparisons integer equality |
| [`EventStore<Id, NameKey>`] | region-event index storage with **B3 lazy rebuild**: edits invalidate only affected statement slots, `prepare` rebuilds only empty slots |
| [`collect_region_events`] | **generic** event collector — dispatches purely on `Lang` hooks (`is_opaque` / `var_key` / `kind` / `children` / `un_op`), strictly matching the engine's `scan_region` recursion order |
| [`stmt_roots`] | statement-root enumeration (direct children of Blocks) |
| [`rebuild_effects`] | per-node aggregate effect table rebuild (fixed-point sweep for injected nodes whose index exceeds their parent's) |

## Conventions

The kit assumes (and documents in the crate docs):

- **Dense arena**: `Lang::node_index` and `Lang::id_of_index` are mutual
  inverses; `Id: Copy`.
- **u32 interned name keys**: your `Lang::NameKey` is a `u32` intern id from
  `NameTable` — region scans degenerate to integer comparisons.
- **Children layouts** follow the [`cure_engine::kind::NodeKind`] contract
  (e.g. `Assign = [target, value]`) — the collector traverses accordingly.

## Proof it works: the reference consumer + a toy language

- [`cure-java-ast`](https://crates.io/crates/cure-java-ast) is built on this
  kit (the Java reference implementation). After the extraction, the
  371,674-file OpenJDK corpus produced byte-identical output.
- `tests/kit.rs` builds a complete toy language in ~100 lines on top of the
  kit and verifies event semantics (Use/Write/Shadow/opaque), effect
  aggregation, invalidation, and lazy rebuild.

## License

MIT. Zero dependencies beyond [`cure-engine`](https://crates.io/crates/cure-engine).
