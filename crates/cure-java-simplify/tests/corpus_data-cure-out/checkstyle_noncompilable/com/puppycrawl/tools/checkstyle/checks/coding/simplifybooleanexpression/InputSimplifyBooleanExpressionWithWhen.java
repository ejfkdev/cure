package com.puppycrawl.tools.checkstyle.checks.coding.simplifybooleanexpression;

public class InputSimplifyBooleanExpressionWithWhen {
    void test(Object o) {
        switch (o) {
            case R(boolean x, _) when x == true -> {}
            case R(_, boolean y) when y != false -> {}
            default -> {}
        }
        switch (o) {
            case R(boolean x, _) when x == false -> {}
            case R(_, boolean y) when (!(y != true)) -> {}
            default -> {}
        }
    }
    void test2(Object o) {
        switch (o) {
            case R(boolean x, _) when x -> {}
            case R(_, boolean y) when y -> {}
            default -> {}
        }
        switch (o) {
            case R(boolean x, _) when !x -> {}
            case R(_, boolean y) when y -> {}
            default -> {}
        }
    }
    record R(boolean x, boolean y) {
    }
}
