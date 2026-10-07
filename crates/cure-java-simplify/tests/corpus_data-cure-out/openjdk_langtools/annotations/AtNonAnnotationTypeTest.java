import java.lang.annotation.Annotation;

class AtNonAnnotationTypeTest<Override extends Annotation> {
    AtNonAnnotationTypeTest(@Override String foo) {}
}
