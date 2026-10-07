public class PatternErrorRecovery {
    void errorRecoveryNoPattern1(Object o) {
        switch (o) {
            case String:
                break;
            case Object obj:
                break;
        }
    }
    int errorRecoveryNoPattern2(Object o) {
        return switch (o) {
            case R(var v, ) -> 1;
            default -> -1;
        };
    }
    record R(String x) {
    }
}
