import java.lang.annotation.*;

class C {
    static int f;
    static {
        @A C.f = 1;
    }
    int a = C.f;
    static int f() {
        return C.f;
    }
    public static void meth() {
        int a = C.f;
    }
}

@Target(ElementType.TYPE_USE) @interface A {
}
