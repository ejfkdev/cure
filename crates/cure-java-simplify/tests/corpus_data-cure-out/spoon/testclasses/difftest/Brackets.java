package spoon.test.prettyprinter.testclasses.difftest;

public class Brackets {
    public String test() {
        Object test = "";
        boolean test2 = ((String) test).isEmpty();
        String test3 = "bla " + test.getClass() + " test";
        if (test3.isEmpty() || test3.isEmpty() || test3.isEmpty()) {}
        if (!test3.isEmpty()) {}
        return "";
    }
}
