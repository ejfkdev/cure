package spoon.test.comment.testclasses;

public class CommentsOnStatements {
    String value = "";
    public String m1() {
        if (value == null) {
            value = "toto";
        } else if (value.equals("x")) {
            this.getClass();
        }
        return value.substring(1);
    }
}
