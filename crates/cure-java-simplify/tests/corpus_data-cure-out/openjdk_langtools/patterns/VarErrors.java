public class VarErrors {
    void testIf(CharSequence cs) {}
    void testSwitchStatement(CharSequence cs) {
        switch (cs) {
            case var v -> {}
        }
    }
    void testSwitchExpression(CharSequence cs) {
        int i = switch (cs) {
            case var v -> 0;
        };
    }
}
