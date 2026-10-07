import java.lang.annotation.*;

class TypeCasts {
    void methodA() {
        String s = (String) null;
        Object o = (Class<@A String>) null;
    }
    void methodB() {
        String s = (String) null;
        Object o = (Class<@B("m") String>) null;
    }
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface A {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface B {
    String value();
}
