import java.lang.annotation.*;

@Target(ElementType.TYPE_USE) @interface TC {
    T[] value();
    int foo();
}
