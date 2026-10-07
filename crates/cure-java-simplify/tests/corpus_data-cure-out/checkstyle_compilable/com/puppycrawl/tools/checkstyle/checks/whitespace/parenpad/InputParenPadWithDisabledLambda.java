package com.puppycrawl.tools.checkstyle.checks.whitespace.parenpad;

public class InputParenPadWithDisabledLambda {
    {
        java.util.function.Consumer a = (o) -> {
            o.toString();
        };
        java.util.function.Consumer b = (o) -> {
            o.toString();
        };
        java.util.function.Consumer c = (o) -> {
            o.toString();
        };
        java.util.function.Consumer d = (o) -> {
            o.toString();
        };
        java.util.function.Consumer e = (o) -> {
            o.toString();
        };
        java.util.stream.Stream.of().forEach((o) -> o.toString());
        java.util.stream.Stream.of().forEach((Object o) -> o.toString());
        java.util.stream.Stream.of().forEach((o) -> o.toString());
    }
    void someMethod(String param) {}
}
