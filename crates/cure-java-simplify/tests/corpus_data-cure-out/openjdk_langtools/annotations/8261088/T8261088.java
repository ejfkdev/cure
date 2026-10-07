import java.lang.annotation.ElementType;
import java.lang.annotation.Repeatable;
import java.lang.annotation.Target;

@Target(ElementType.MODULE) @interface TC {
    T[] value() default {};
}

@Repeatable(TC.class) @interface T {
}
