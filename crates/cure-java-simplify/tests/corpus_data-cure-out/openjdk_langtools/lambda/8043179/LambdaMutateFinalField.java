class LambdaMutateFinalField {
    final String x;
    LambdaMutateFinalField() {
        Runnable r1 = () -> x = "not ok";
        this.x = "ok";
    }
}
