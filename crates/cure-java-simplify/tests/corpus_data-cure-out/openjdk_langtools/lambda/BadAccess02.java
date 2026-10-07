public class BadAccess02 {
    interface SAM {
        int m(int h);
    }
    static void test1() {
        SAM s = (int h) -> {
            return h + 2 + 0 + 0;
        };
    }
    void test2() {
        SAM s = (int h) -> {
            return h + 0 + 2 + 0 + 0;
        };
    }
}
