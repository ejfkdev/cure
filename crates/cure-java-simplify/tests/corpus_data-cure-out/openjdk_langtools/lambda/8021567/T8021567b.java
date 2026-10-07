public class T8021567b {
    interface SAM {
        int m();
    }
    public static void main(String[] argv) {
        test();
    }
    static boolean test() {
        SAM s = () -> 0;
        return s.m() == 0;
    }
}
