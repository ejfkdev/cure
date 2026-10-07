package net.sourceforge.pmd.lang.java.symbols.testdata.sealed;

sealed class ImplicitPermitsClause {
}

sealed interface ImplicitPermitsClauseItf {
}

class Foo {
    non-sealed class Bar extends ImplicitPermitsClause implements ImplicitPermitsClauseItf {
    }
}

final class Qux extends ImplicitPermitsClause {
    static {
        new Foo().new Bar() {};
    }
}

non-sealed interface SubItf extends ImplicitPermitsClauseItf {
}

sealed interface SubItf2 extends SubItf {
}

record FooRecord() implements SubItf2 {
}

enum FooEnum implements SubItf2 {
    A { }, B
}
