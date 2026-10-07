package syntax1;

import java.lang.annotation.*;
import java.util.*;

@Documented
@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.METHOD, ElementType.TYPE)
public @interface Syntax1 {
    String elementName();
}
