package net.sourceforge.pmd.lang.java.symbols.testdata.sealed;

sealed interface SealedTypesTestData permits A, B, C {
}

sealed interface A permits X extends SealedTypesTestData {
}

non-sealed interface B extends SealedTypesTestData {
}

final class C implements SealedTypesTestData {
}

final class X implements A {
}
