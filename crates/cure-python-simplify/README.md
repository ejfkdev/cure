# cure-python-simplify

Python simplification facade over `cure-engine`: literal/builtin/subscript
folding with Python int/long and truthiness semantics, control-flow cleanup,
function-scope dead-assignment elimination, and docstring/shadowing guards.
25 rules are enabled by default (`cure rules` lists them all).

Part of the cure workspace: a fault-tolerant, semantic-preserving simplifier
and formatter for Java, TypeScript/JavaScript and Python.