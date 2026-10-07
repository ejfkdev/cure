class LambdaConv10 {
    interface Method1<R, A1> {
        public R call(A1 a1);
    }
    public static void meth() {
        Method1<Integer,Integer> m1 = (int i) -> 2 * i;
    }
}
