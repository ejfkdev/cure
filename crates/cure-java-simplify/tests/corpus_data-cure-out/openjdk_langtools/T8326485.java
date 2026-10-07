import java.lang.annotation.ElementType;
import java.lang.annotation.Target;

public class T8326485 {
    @Ann not.java.lang.String f;
}

@Target({ElementType.TYPE_USE, ElementType.FIELD}) @interface Ann {
}
