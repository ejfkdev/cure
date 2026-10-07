package spoon.test.annotation.testclasses;

import java.util.ArrayList;
import java.util.List;

public class AnnotationsAppliedOnAnyTypeInAClass {
    public void m() throws @TypeAnnotation Exception {}
    public String m2() {
        Object s = new @TypeAnnotation String();
        return (String) s;
    }
    public @TypeAnnotation String m3() {
        return "";
    }
    public <@TypeAnnotation T> void m4() {
        List<@TypeAnnotation T> list = new ArrayList<>();
        List<@TypeAnnotation ?> list2 = new ArrayList<>();
        List<@TypeAnnotation BasicAnnotation> list3 = new ArrayList<@TypeAnnotation BasicAnnotation>();
    }
    public <T> void m5() {}
    public void m6(@TypeAnnotation String param) {
        @TypeAnnotation String s = "";
    }
    public enum DummyEnum implements @TypeAnnotation BasicAnnotation {
    }
    public interface DummyInterface extends @TypeAnnotation BasicAnnotation {
    }
    public class DummyClass extends @TypeAnnotation AnnotArrayInnerClass implements @TypeAnnotation BasicAnnotation {
    }
    public class DummyGenericClass<@TypeAnnotation T, @TypeAnnotation K> implements BasicAnnotation<@TypeAnnotation T> {
    }
}
