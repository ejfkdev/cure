class AnnotationCommaArrayInit {
    @Foo({,}) void b() {}
    @interface Foo {
        int[] value();
    }
}
