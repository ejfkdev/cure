# cure-dart-parser

Fault-tolerant Dart lexer/token-document parser for
[cure](https://github.com/ejfkdev/cure). Covers Dart 1/2/3 lexical syntax:
nested block comments, raw/triple/interpolated strings (including nested
strings inside `${...}`), binary/hex numbers with separators, all operator
forms, shebangs and annotations. Unterminated literals or unbalanced brackets
are reported but never abort the file.

Part of the cure workspace: a fault-tolerant, semantic-preserving simplifier
and formatter for Java, TypeScript/JavaScript, Python and Dart.
