package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.util.Optional;

public class InputIndentationNewChildren {
    public Object foo() {
        return Optional.empty().orElseThrow(() -> new IllegalArgumentException("Something wrong 1, something wrong 2, something wrong 3"));
    }
    public Object foo1() {
        return Optional.empty().orElseThrow(() -> new IllegalArgumentException("Something wrong 1, something wrong 2, something wrong 3"));
    }
    void foo2() throws IOException {
        BufferedReader bf = new BufferedReader(new InputStreamReader(System.in) {                                    //indent:8 exp:12 warn
          int a = 0;                                                          //indent:10 exp:14,16,18 warn
            });
    }
    public Object foo4(int data) {
        return Optional.empty().orElseThrow(() -> new IllegalArgumentException("something wrong 1, something wrong 2, something wrong 3"));
    }
    public void createExpressionIssue(Object invocation, String expression) {
        throw new IllegalArgumentException("The expression " + expression + ", which creates" + invocation + " cannot be removed. Override method `canRemoveExpression` to customize this behavior.");
    }
    public Object foo5(int data) {
        return Optional.empty().orElseThrow(() -> new IllegalArgumentException("something wrong 1, something wrong 2, something wrong 3"));
    }
    public Object foo6(int data) {
        return Optional.empty().orElseThrow(() -> new IllegalArgumentException("something wrong 1, something wrong 2"));
    }
}
