package com.puppycrawl.tools.checkstyle.checks.metrics.booleanexpressioncomplexity;

public class InputBooleanExpressionComplexityWhenExpression {
    void test(Object obj) {
        switch (obj) {
            case ColoredPoint(boolean a, boolean b, boolean _)
                    when (a ^ (a || b) ^ (b || a) & (a | b)) -> {
                boolean c = a ^ (a || b) ^ (b || a) & (a | b);
            }
            case ColoredPoint(boolean a, _, _) when a && a -> {}
            default -> System.out.println("none");
        }
    }
    record ColoredPoint(boolean p, boolean x, boolean c) {
    }
}
