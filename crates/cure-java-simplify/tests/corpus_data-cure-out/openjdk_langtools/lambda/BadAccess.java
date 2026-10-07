public class BadAccess {
    int i;
    static int I;
    interface SAM {
        int m();
    }
    static void test1() {}
    void test2() {}
}
