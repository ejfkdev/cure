import java.lang.annotation.ElementType;
import java.lang.annotation.Target;

public class TypeAnnotationPositionTest {
    TypeAnnotationPositionTest(char[] bar) {}
    @Target({ElementType.TYPE_USE}) @interface MyTest {
    }
    TypeAnnotationPositionTest test() {
        return new TypeAnnotationPositionTest(new char[] {'1'});
    }
}
