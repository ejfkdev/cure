package pkg;

public class T8145051 {
    public String s;
    public static class Sup {
        Sup(Runnable r) {
            r.run();
        }
    }
    public class Sub extends Sup {
        public Sub() {
            super(() -> {
                s = "Executed lambda";
            });
        }
    }
}
