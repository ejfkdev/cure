# cure-dart-ast

Lossless Dart token document used by [cure](https://github.com/ejfkdev/cure):
tokens with source positions and newline trivia, bracket matching, block vs
collection brace classification, and token-based metrics. The model accepts
Dart 1, Dart 2 and Dart 3 syntax without a version switch, because unmodelled
constructs stay as ordinary tokens instead of being rejected.

Part of the cure workspace: a fault-tolerant, semantic-preserving simplifier
and formatter for Java, TypeScript/JavaScript, Python and Dart.
