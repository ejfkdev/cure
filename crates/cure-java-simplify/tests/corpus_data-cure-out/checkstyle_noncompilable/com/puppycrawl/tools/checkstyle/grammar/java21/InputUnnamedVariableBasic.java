package com.puppycrawl.tools.checkstyle.grammar.java21;

public class InputUnnamedVariableBasic {
    private void unnamedTest() {
        int _ = 0;
        int _ = 1;
        try (Lock _ = null) {
            try (Lock _ = null) {} catch (Exception _) {
                try {} catch (Exception _) {}
            }
        }
        try (Lock _ = null) {}
        try (Lock _ = null) {}
        String[] strs = {"str1", "str2"};
        for (var _ : strs) {
            for (var _ : strs) {}
        }
        TwoParams p1 = (_, _) -> {};
        TwoParams p2 = (var _, var _) -> {};
        TwoIntParams p3 = (int _, int b) -> {};
        TwoIntParams p4 = (int _, int _) -> {};
        TwoIntParamsIntRet p5 = (int _, int _) -> {
            return 1;
        };
        p1.run(1, 2);
        p2.run(1, 2);
        p3.run(1, 2);
        p4.run(1, 2);
        p5.run(1, 2);
        R r = new R(null);
        for (int _ = 0, _ = 1, x = 1; x <= 1; x++) {}
    }
    record Point(int x, int y) {
    }
    enum Color {
        RED, GREEN, BLUE
    }
    record ColoredPoint(Point p, Color c) {
    }
    void jep443examples(ColoredPoint r) {}
    class Lock implements AutoCloseable {
        @Override
        public void close() {}
    }
    interface TwoParams {
        public void run(Object o1, Object o2);
    }
    interface TwoIntParams {
        public void run(int o1, int o2);
    }
    interface TwoIntParamsIntRet {
        public int run(int a, int b);
    }
    record R(Object o) {
    }
    public @interface Foo {
    }
}
