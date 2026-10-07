public class LambdaTest2_neg1 {
    public static void meth() {
        new LambdaTest2_neg1().methodQooRoo((Integer i) -> {});
    }
    void methodQooRoo(QooRoo<Integer, Integer, Void> qooroo) {}
}
