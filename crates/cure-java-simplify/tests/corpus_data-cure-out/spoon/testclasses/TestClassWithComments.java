package spoon.test.comment.testclasses;

public class TestClassWithComments {
    public interface testInterface {
        public void mytest(short a, short b);
    }
    private static class myInterface implements testInterface {
        public void mytest(short a, short b) {}
    }
}
