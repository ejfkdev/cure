package spoon.test.annotation.testclasses;

public class AnnotationIntrospection {
    @TestAnnotation
    public void m() throws NoSuchMethodException {
        getClass().getMethod("m").getAnnotation(TestAnnotation.class).equals(null);
    }
}
