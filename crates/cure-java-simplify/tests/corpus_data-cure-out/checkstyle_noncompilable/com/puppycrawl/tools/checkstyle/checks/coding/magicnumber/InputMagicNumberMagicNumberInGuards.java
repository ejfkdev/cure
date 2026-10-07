package com.puppycrawl.tools.checkstyle.checks.coding.magicnumber;

public class InputMagicNumberMagicNumberInGuards {
    void m(Object o) {
        switch (o) {
            case Point(int x, int y, double z) when (x < 3 && y != 6) -> {}
            case Point(_, _, double z) when z > (10.88) -> {}
            case String s when s.length() != 6 -> {}
            default -> {}
        }
        int w = switch (o) {
            case Point(int x, int y, double z) when (z == 0.5) -> 5;
            case String s -> {
                yield 6;
            }
            default -> 0;
        };
    }
    record Point(int x, int y, double z) {
    }
}
