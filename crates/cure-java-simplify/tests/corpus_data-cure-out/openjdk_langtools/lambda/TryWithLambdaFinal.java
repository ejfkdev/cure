class TryWithLambdaFinal {
    private final int x;
    public TryWithLambdaFinal() {
        try {
            Runnable r = () -> {
                try {
                    return;
                } catch (Exception e) {
                    throw new AssertionError(e);
                }
            };
        } catch (Exception e) {}
        x = 42;
    }
}
