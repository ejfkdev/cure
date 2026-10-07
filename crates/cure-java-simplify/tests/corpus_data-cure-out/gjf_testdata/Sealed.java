class T {
    sealed interface I permits C, B extends A {
    }
    final class C implements I {
    }
    sealed private interface A permits I {
    }
    non-sealed private interface B extends I {
    }
}
