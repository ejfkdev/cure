import java.lang.annotation.*;
import java.util.List;

class LintCast {
    void unparameterized() {
        String s2 = "m";
    }
    void parameterized() {
        List<String> l = null;
        List<String> l1 = (List<String>) l;
        List<String> l2 = (List<@A String>) l;
    }
    void array() {
        int[] a = null;
        int[] a1 = (int[]) a;
        int[] a2 = (int[]) a;
    }
    void sameAnnotations() {
        @A String annotated = null;
        String unannotated = null;
        @A String anno1 = (@A String)annotated;

                // warn if redundant without an annotation
        String anno2 = (String) annotated;
        String unanno2 = (String) unannotated;
    }
    void more() {
        Object[] a = null;
        Object[] a1 = (Object[]) a;
        Object[] a2 = (Object[]) a;
        @A List<String> l3 = null;
        List<String> l4 = (List<String>) l3;
        List<String> l5 = (List<String>) l3;
        List<@A String> l6 = null;
        List<String> l7 = (List<String>) l6;
        List<String> l8 = (List<@A String>) l6;
        @A Object o = null;
        Object o1 = (Object) o;
        Object o2 = (Object) o;
        Outer.Inner oi = null;
        Outer.Inner oi1 = (Outer.Inner) oi;
        Outer.Inner oi2 = (Outer.Inner) oi;
    }
    class Outer {
        class Inner {
        }
    }
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface A {
}
