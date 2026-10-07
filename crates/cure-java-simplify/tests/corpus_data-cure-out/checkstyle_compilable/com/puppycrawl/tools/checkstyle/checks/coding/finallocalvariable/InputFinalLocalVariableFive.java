package com.puppycrawl.tools.checkstyle.checks.coding.finallocalvariable;

public class InputFinalLocalVariableFive {
    class class5 {
        public void test1() {
            boolean b = false;
            int shouldBeFinal;
            if (b) {
                shouldBeFinal = b ? 1 : 2;
            }
        }
        public void test2() {
            int b = 10;
            int shouldBeFinal;
            switch (b) {
                case 0:
                    switch (b) {
                        case 0:
                            shouldBeFinal = 1;
                            break;
                        default:
                            shouldBeFinal = 2;
                            break;
                    }
                    break;
                default:
                    shouldBeFinal = 3;
                    break;
            }
        }
        public void test3() {
            int x;
            try {
                x = 0;
                try {
                    x = 0;
                } catch (Exception e) {
                    x = 1;
                }
            } catch (Exception e) {
                x = 1;
            }
        }
        public void test4() {
            int shouldBeFinal;
            class Bar {
                            void bar () {
                                // violation below "Variable 'shouldBeFinal' should be declared final"
                                int shouldBeFinal;
                                final boolean b = false;
                                if (b) {
                                    if (b) {
                                        shouldBeFinal = 1;
                                    } else {
                                        shouldBeFinal = 2;
                                    }
                                } else {
                                    shouldBeFinal = 2;
                                }
                            }
                        }

                        abstract class Bar2 {
                            abstract void method(String param);
                        }
        }
        public void test5() {
            InputFinalLocalVariableFive table = new InputFinalLocalVariableFive();
            new Runnable() {
                @Override
                public void run() {
                    InputFinalLocalVariableFive table = null;
                    table = new InputFinalLocalVariableFive();
                }
            };
        }
        public void test6() {
            byte[] tmpByte = new byte[0];
        }
    }
}
