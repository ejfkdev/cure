package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

public class InputIndentationSwitchCasesAndEnums {
    {
        System.out.println("Hello Checks");
    }
    public static void test() {
        Test myTest = Test.ONE;
        switch (myTest) {
            case ONE:
                {
                    System.out.println("One");
                    break;
                }
            case TWO:
                {
                    System.out.println("Two");
                    break;
                }
            case THREE:
                {
                    System.out.println("Three");
                    break;
                }
            case FOUR:
                {
                    System.out.println("FOur with different brace style");
                }
            default:
                throw new RuntimeException("Unexpected value");
        }
    }
    public enum Test {
        ONE, TWO, THREE, FOUR
    }
    {
        System.out.println("Hello Checks");
    }
}
