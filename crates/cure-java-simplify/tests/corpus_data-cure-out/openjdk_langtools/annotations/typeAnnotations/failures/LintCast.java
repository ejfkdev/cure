import java.lang.annotation.*;
import java.util.List;

class LintCast {
    void unparameterized() {}
    void parameterized() {
        List<String> l2 = (List<@A String>) null;
    }
    void array() {}
    void sameAnnotations() {
        @A String annotated = null;
        String unannotated = null;
        @A String anno1 = (@A String)annotated;

                // warn if redundant without an annotation
        String anno2 = (String) annotated;
    }
    void more() {
        Object[] a = null;
        Object[] a1 = a;
        Object[] a2 = a;
        @A List<String> l3 = null;
        List<String> l4 = (List<String>) l3;
        List<String> l5 = (List<String>) l3;
        List<@A String> l6 = null;
        List<String> l7 = (List<String>) l6;
        List<String> l8 = l6;
        @A Object o = null;
        Object o1 = (Object) o;
        Object o2 = (Object) o;
    }
    class Outer {
        class Inner {
        }
    }
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface A {
}
