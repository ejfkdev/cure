package com.puppycrawl.tools.checkstyle.checks.coding.finallocalvariable;

public class InputFinalLocalVariable2Three {
    class class22 {
        public void method1() {}
        public void method2() {
            for (int i = 0; i < 5; i++) {}
            int y;
            for (int i = 0; i < 5; i++) {
                y = 3;
            }
            for (int i = 0; i < 5; i++) {
                int z;
                for (int j = 0; j < 5; j++) {
                    z = 3;
                }
            }
        }
        public void method3() {
            int m;
            do {
                m = 0;
            } while (false);
            do {} while (true);
        }
        private void foo() {
            int q;
            int w;
            int e;
            q = 1;
            w = 1;
            e = 2;
            class Local {
                            void bar() {
                                int q;
                                int w;
                                int e;
                                q = 1;
                                q = 2;
                                w = 1;
                                e = 1;
                            }
                        }

                        int i;
            for (; ; i = 1) {}
        }
        public void method4() {
            int m;
            int i = 5;
            while (i > 1) {
                m = 0;
                i++;
            }
            while (true) {}
        }
        int[] array = new int[10];
        public void method5() {
            int r;
            for (int a : array) {
                r = 0;
            }
            for (int a : array) {}
        }
    }
    class classs32 {
        public void method(final int i) {
            switch (i) {
                case 1:
                    int foo = 1;
                    break;
                default:
            }
            switch (i) {
                case 1:
                    int foo = 1;
                    break;
                case 2:
                    foo = 2;
                    break;
                default:
            }
        }
    }
}
