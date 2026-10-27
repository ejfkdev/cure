# cure-dart-print

Comment-preserving Dart source printer for
[cure](https://github.com/ejfkdev/cure). Formats every token stream produced by
`cure-dart-parser` with two-space indentation, block/collection-aware layout,
switch-case indentation, generic type spacing and original blank-line
paragraphing. Comments and string literal contents are emitted byte-for-byte.

Part of the cure workspace: a fault-tolerant, semantic-preserving simplifier
and formatter for Java, TypeScript/JavaScript, Python and Dart.
