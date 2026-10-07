import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

public class T8320144 {
    @Retention(RetentionPolicy.RUNTIME)
    @Target({ElementType.TYPE})
    public @interface TestAnnotation {
        public String[] excludeModules() default new String[0];
        public String[] value() default new String[] { 3 };
    }
}
