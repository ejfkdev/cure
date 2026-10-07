class LambdaMutateFinalVar {
    LambdaMutateFinalVar() {
        String x;
        Runnable r1 = () -> x = "not ok";
        x = "ok";
    }
}
