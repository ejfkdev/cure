public class LambdaTest1_neg3 {
    void method() {
        int n = 2;
        ((Runnable) (() -> {})).run();
        n++;
    }
}
