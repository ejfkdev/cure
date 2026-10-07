package spoon.test.comment.testclasses;

public class CatchComments {
    public static void exampleMethod() {
        try {
            Object o = new Object();
        } catch (Exception e) {
            int x = 42;
        }
    }
}
