package spoon.test.comment.testclasses;

public class ArrayAccessComments {
    public void bar(int[] foo) {
        foo[1] = 0;
        int bar = foo[0];
    }
}
