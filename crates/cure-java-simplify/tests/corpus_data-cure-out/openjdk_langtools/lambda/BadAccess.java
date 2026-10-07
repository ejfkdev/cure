public class BadAccess {
    int i;
    static int I;
    interface SAM {
        int m();
    }
    static void test1() {
        SAM s = () -> i + I + 0 + 0;
    }
    void test2() {
        SAM s = () -> i + I + 0 + 0;
    }
}
