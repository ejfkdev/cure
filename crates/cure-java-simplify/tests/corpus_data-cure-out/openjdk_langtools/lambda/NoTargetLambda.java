public class NoTargetLambda {
    private void t(boolean b) {
        (b ? "" : (() -> {
            return null;
        })).toString();
    }
}
