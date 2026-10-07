package p;

class DeprecatedAnnotationTest implements AutoCloseable {
    void foo(@Deprecated int p) {
        @Deprecated int l;
        try (DeprecatedAnnotationTest r = new DeprecatedAnnotationTest()) {} catch (@Deprecated Exception e) {}
    }
    @Override
    public void close() throws Exception {
        @SuppressWarnings("deprecation")  // verify that we are able to suppress.
                @Deprecated int x;
    }
}
