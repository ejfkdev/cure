public class T8312229 {
    void test(Object o) {
        Runnable r = () -> {
            var l = switch (o) {
                default -> {
                    Integer i = 42;
                    yield new Runnable() {
                                            public void run() {
                                                i.toString(); // should not crash here
                                            }
                                        };
                }
            };
        };
    }
}
