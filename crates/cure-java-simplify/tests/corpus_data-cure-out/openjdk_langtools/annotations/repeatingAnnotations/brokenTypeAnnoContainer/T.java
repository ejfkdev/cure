import java.lang.annotation.*;

@Target(ElementType.TYPE_USE)
@Repeatable(TC.class) @interface T {
    int value();
}
