# cure-python-parser

Fault-tolerant, indentation-aware Python parser for
[cure](https://github.com/ejfkdev/cure). Accepts Python 2.7 and Python 3.x in
one grammar, preserves version-ambiguous forms byte-for-byte, and keeps
unparseable regions verbatim so surrounding code can still be simplified.

Part of the cure workspace: a fault-tolerant, semantic-preserving simplifier
and formatter for Java, TypeScript/JavaScript and Python.