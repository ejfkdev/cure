package spoon.test.template.testclasses.match;

import static java.lang.System.out;

public class MatchIfElse {
    public void matcher1(boolean option, boolean option2) {
        if (option) {
            System.out.println("string");
        } else {
            System.out.println(4.5);
        }
    }
    public void testMatch1() {
        System.out.println(0);
        System.out.println("a");
        out.println("Xxxx");
        System.out.println((String) null);
        System.out.println((Integer) null);
        System.out.println(2018);
        System.out.println(Long.class.toString());
        System.out.println(3.14);
    }
}
