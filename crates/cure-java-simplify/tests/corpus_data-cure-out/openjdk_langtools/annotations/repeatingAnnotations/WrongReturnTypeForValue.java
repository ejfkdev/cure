import java.lang.annotation.Repeatable;

@Repeatable(FooContainer.class) @interface Foo {
    int getNumbers();
}

@interface FooContainer {
    Foo value();     // wrong return type
}

@Foo @Foo
public class WrongReturnTypeForValue {
}
