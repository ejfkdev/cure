# cure-java-ast

[![crates.io](https://img.shields.io/crates/v/cure-java-ast.svg)](https://crates.io/crates/cure-java-ast)
[![Documentation](https://docs.rs/cure-java-ast/badge.svg)](https://docs.rs/cure-java-ast)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/ejfkdev/cure/blob/main/LICENSE)

Java AST for the [cure](https://github.com/ejfkdev/cure) pipeline: a
`u32`-indexed arena with copy-increment ids, interned name symbols, and the
reference implementation of the engine's [`Lang`] trait.

## Design

- **Arena, not boxes.** Every node is a `Node { data: NodeData, children }`
  slot in one `Vec`; a node handle is a `JavaId(u32)` — `Copy`, comparable,
  sortable. Tree rewrites become cheap arena edits instead of pointer surgery.
- **Child list inline storage.** Children ≤ 3 are stored inline in the node
  (no heap allocation); larger lists spill to the heap. Java grammar is
  fan-out-heavy with tiny arity, so the common case is allocation-free.
- **Interned name symbols.** Names live in a unit-level symbol table
  (`Sym(u32)`); nodes reference symbols, not `String`s — 87 % of identifier
  occurrences in real corpora are repeats.
- **Derived analysis caches.** The AST carries invalidation-tracked caches:
  per-node aggregate effects, variable types from scope resolution, and a
  region-event index (use/write/shadow events over name keys) that the
  engine binary-searches for kill analysis. Caches rebuild lazily and only
  for invalidated subtrees.
- **Signatures vs. bodies.** Class/member signatures are kept in plain
  structs (`CompilationUnit`, `TypeDecl`, `Member`, …) for fidelity;
  executable bodies live in the arena.

## Navigating

```rust
use cure_java_ast::{JavaAst, JavaId, NodeData};

/// Collect every reference to the variable `name` under `root`.
fn find_refs(ast: &JavaAst, root: JavaId, name: &str, out: &mut Vec<JavaId>) {
    if let NodeData::VarRef { name: sym } = ast.node(root).data {
        if ast.sn(sym) == name {
            out.push(root);
        }
    }
    for &child in ast.children(root) {
        find_refs(ast, child, name, out);
    }
}
```

Construction normally happens via
[`cure-java-parser`](https://crates.io/crates/cure-java-parser); printing via
[`cure-java-print`](https://crates.io/crates/cure-java-print); the rewrite
rules over this AST live in
[`cure-java-simplify`](https://crates.io/crates/cure-java-simplify) — see the
[pipeline example](https://github.com/ejfkdev/cure#usage) in the repository
README.

License: MIT. Zero runtime dependencies.
