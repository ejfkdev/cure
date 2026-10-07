class AnnotatedMethodSelectorTest {
    @interface A {
    }
    static public void main(String... args) {
        java.util.@A Arrays.stream(args);
    }
}
