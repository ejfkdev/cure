@interface Anno {
    @Deprecated
        boolean b() default false;
}

@Anno(b = true) class Foo {
}
