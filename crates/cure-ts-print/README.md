# cure-ts-print

Source printer for cure's TypeScript/JavaScript backend. Emits the
`cure-ts-ast` tree with precedence-aware parentheses, and supports
byte-for-byte function/class/object-literal output for
`Function.prototype.toString` fidelity.

Part of the cure workspace: a fault-tolerant, semantic-preserving simplifier
and formatter for Java, TypeScript/JavaScript and Python.