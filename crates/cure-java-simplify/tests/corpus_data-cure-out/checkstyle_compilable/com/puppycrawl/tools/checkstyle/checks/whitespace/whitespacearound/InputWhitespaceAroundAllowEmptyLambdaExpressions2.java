package com.puppycrawl.tools.checkstyle.checks.whitespace.whitespacearound;

import java.util.function.BinaryOperator;

public class InputWhitespaceAroundAllowEmptyLambdaExpressions2 {
    Runnable noop = () -> {};
    Runnable noop2 = () -> {};
    BinaryOperator<Integer> sum = (x, y) -> x + y;
    Runnable noop3 = () -> {};
    Runnable noop4 = () -> {};
}
