package spoon.test.comment.testclasses;

public class WithIfBlock {
    String value = "";
    public String myMethod() {
        if (value == null) {
            value = "toto";
        }
        return value.substring(1);
    }
}
