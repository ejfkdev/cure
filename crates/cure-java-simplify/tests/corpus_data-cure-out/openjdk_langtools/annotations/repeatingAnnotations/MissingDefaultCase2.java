import java.lang.annotation.Repeatable;

@Repeatable(FooContainer.class) @interface Foo {
}

@interface FooContainer {
    Foo[] value();
    Foo other();  // missing default clause and return type is an annotation
}

@Foo @Foo
public class MissingDefaultCase2 {
}
