# cure-dart-simplify

Dart simplification over the lossless token document from
`cure-dart-ast`: safe integer/bool/string constant folding, constant
`if`/`else` branch selection, if→ternary for matching return/assignment
branches, else-after-terminal removal, unreachable-statement removal, loop-tail `continue` and empty
`finally` cleanup, adjacent string literal merging, atom parenthesis removal,
empty-else cleanup and `${identifier}` → `$identifier` interpolation
compaction. Rules are version-independent and never rewrite unmodelled Dart;
`new` is preserved because Dart 1 required it, and if-case patterns are never
folded into ternaries.

Part of the cure workspace: a fault-tolerant, semantic-preserving simplifier
and formatter for Java, TypeScript/JavaScript, Python and Dart.
