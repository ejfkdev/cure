public class PatternVariablesAreNonFinal2 {
    public static void main(String[] args) {
        Object o = "42";
        if (o instanceof String s) {
            new Object() {
                void run() { System.err.println(s); }
            }.run();
        }
        if (o instanceof String s) {
            s = "hello again";
            System.out.println(s);
        }
    }
}
