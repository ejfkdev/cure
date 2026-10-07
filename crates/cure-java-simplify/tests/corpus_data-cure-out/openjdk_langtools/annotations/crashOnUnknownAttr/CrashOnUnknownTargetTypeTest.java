public class CrashOnUnknownTargetTypeTest {
    @A Object o;
    @A void m() {}
    void foo(@A int a) {
        @A int i = 0;
    }
    @A CrashOnUnknownTargetTypeTest() {}
    @A @interface B {
    }
    class Inner<@A T> {
    }
    @A class Inner2 {
    }
    record R(@A int i) {
    }
}
