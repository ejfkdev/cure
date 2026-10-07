import java.lang.annotation.Repeatable;

@Repeatable(FooContainer.class) @interface Foo {
}

@interface FooContainer {
    Foo[] values();  // wrong method name
}

@Foo @Foo
public class MissingValueMethod {
}
