package com.puppycrawl.tools.checkstyle.checks.blocks.needbraces;

public class InputNeedBracesTestSwitchExpressionNoSingleLine {
    void howMany1(NumsOne k) {
        switch (k) {
            case ONE:
                System.out.println("case two");
                MathOperationOne case5 = (a, b) -> a + b;
            case TWO:
            case THREE:
                System.out.println("case two");
            case FOUR:
                System.out.println("case three");
            default:
                throw new IllegalStateException("Not a nums");
        }
    }
    void howMany2(NumsOne k) {
        switch (k) {
            case ONE -> System.out.println("case one");
            case TWO, THREE -> System.out.println("case two");
            case FOUR -> System.out.println("case three");
            default -> throw new IllegalStateException("Not a nums");
        }
    }
    int howMany3(NumsOne k) {
        return switch (k) {
            case ONE:
                MathOperationOne case5 = (a, b) -> a + b;
                yield 3;
            case TWO:
            case THREE:
                yield 5;
            case FOUR:
                yield 9;
            default:
                throw new IllegalStateException("Not a Nums");
        };
    }
    int howMany4(NumsOne k) {
        return switch (k) {
            case ONE -> {
                yield 4;
            }
            case TWO, THREE -> {
                MathOperationOne case5 = (a, b) -> a + b;
                yield 42;
            }
            case FOUR -> {
                yield 99;
            }
            default -> throw new IllegalStateException("Not a Nums");
        };
    }
    int howMany5(NumsOne k) {
        return switch (k) {
            case ONE -> 1;
            case TWO, THREE -> 3;
            case FOUR -> 4;
            default -> {
                throw new IllegalStateException("Not a Nums");
            }
        };
    }
}

enum NumsOne {
    ONE, TWO, THREE, FOUR
}

interface MathOperationOne {
    int operation(int a, int b);
}
