public class FoldConditional {
    static void f(double x) {}
    static void f(int x) {
        throw new Error();
    }
    public static void main(String[] args) {
        String value0 = ("value is " + 9.0).intern();
        String value1 = "value is 9".intern();
        String value2 = "value is 9".intern();
        f(9);
        f(9);
        if (value0 != value1) 
            throw new Error();
        if (value0 != value2) 
            throw new Error();
    }
}
