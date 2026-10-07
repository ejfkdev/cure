public @interface Foo {
    static final ThreadLocal<Interner<Integer>> interner =
            ThreadLocal.withInitial(Interners::newStrongInterner);
}
