public class T8332463b {
    public int test1() {
        Byte i = (byte) 42;
        return switch (i) {
            case Byte ib -> 1;
            case (short) 0 -> 2;
        };
    }
}
