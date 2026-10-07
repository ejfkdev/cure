package com.puppycrawl.tools.checkstyle.checks.coding.finallocalvariable;

public class InputFinalLocalVariableCheckSwitchExpressionsC {
    enum T {
        A, B, C
    }
    public void foo5() {
        int x;
        T e = T.B;
        boolean t8 = switch (e) {
            case A:
                x = 1;
                yield isTrue();
            case B:
                yield (x = 1) == 1 || isTrue();
            default:
                yield false;
        } && x == 1;
        {
            T y = T.A;
            boolean t9 = switch (y) {
                case A:
                    x = 1;
                    yield true;
                case B:
                    yield (x = 1) == 1 || true;
                default:
                    yield false;
            } && x == 1;
        }
        {
            int v = switch (T.A) {
                case A -> x = 0;
                case B -> x = 0;
                case C -> x = 0;
            };
            if (x != 0 || v != 0) {
                throw new IllegalStateException("Unexpected result.");
            }
        }
        {
            T y = T.A;
            boolean tA = switch (y) {
                case A -> {
                    x = 1;
                    yield true;
                }
                case B -> {
                    x = 1;
                    yield true;
                }
                case C -> {
                    x = 1;
                    yield true;
                }
            } && x == 1;
        }
    }
    private boolean isTrue() {
        return true;
    }
}
