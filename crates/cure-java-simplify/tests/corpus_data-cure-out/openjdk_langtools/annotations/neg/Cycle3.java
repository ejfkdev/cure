package cycle3;

import java.lang.annotation.Retention;
import static java.lang.annotation.RetentionPolicy.*;

@Retention(RUNTIME) @interface A {
    A[] values() default { @A };
}

@A class Main {
}
