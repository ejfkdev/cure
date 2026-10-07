public class VoidLambdaParameter {
    Runnable r = (void v) -> {};
    I i = (void v) -> {};
    interface I {
        public void v(void v);
    }
}
