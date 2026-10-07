public class PatternCaseErrorRecovery {
    Object expressionLikeType(Object o1, Object o2) {
        int a = 1;
        int b = 2;
        return switch (o1) {
            case true:
                (t) -> o2;
            case 2:
                (e) -> o2;
            case a < b ? a : b:
                (e) -> o2;
            default -> null;
        };
    }
}
