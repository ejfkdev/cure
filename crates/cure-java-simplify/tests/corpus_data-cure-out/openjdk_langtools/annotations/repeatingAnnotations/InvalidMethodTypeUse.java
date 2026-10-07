import java.lang.annotation.*;

class InvalidMethodTypeUse {
    @Target({ElementType.TYPE_USE, ElementType.METHOD, ElementType.TYPE_PARAMETER})
    @Repeatable(TC.class) @interface T {
        int value();
    }
    @Target({ElementType.METHOD, ElementType.TYPE_PARAMETER}) @interface TC {
        T[] value();
    }
    void method() {
        this.method2();
    }
    <@T(3) S> void method2() {}
}
