public class SuperClassThisCapture extends a.A {
    public static void main(String[] args) {
        new SuperClassThisCapture().f(42);
        new SuperClassThisCapture().g();
    }
    public void f(int x) {
        Runnable r = () -> {
            System.err.println(x);
            new I();
        };
        r.run();
    }
    public void g() {
        Runnable r = () -> new I();
        r.run();
    }
}
