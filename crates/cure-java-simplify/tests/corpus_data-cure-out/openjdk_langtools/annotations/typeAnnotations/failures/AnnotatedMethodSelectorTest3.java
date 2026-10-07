class AnnotatedMethodSelectorTest3 {
    @interface A {
    }
    static <T> AnnotatedMethodSelectorTest3 id() {
        return null;
    }
    static public void main(String... args) {
        AnnotatedMethodSelectorTest3.id().id().id().id().id();
    }
}
