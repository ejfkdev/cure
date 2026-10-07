class TargetType17 {
    interface SAM<X> {
        boolean m(X x);
    }
    byte b = (byte) (() -> true);
    short s = (short) (() -> 1);
    int i = (int) (() -> 1);
    long l = (long) (() -> 1L);
    float f = (float) (() -> 1.0F);
    double d = (double) (() -> 1.0);
    char c = (char) (() -> 'c');
    boolean z = (boolean) (() -> true);
}
