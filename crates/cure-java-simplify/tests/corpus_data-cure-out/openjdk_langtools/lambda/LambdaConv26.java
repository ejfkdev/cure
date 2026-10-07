public class LambdaConv26 {
    interface I {
        Object clone();
    }
    Object m() {
        return null;
    }
    void test() {
        I i2 = this::m;
    }
}
