package spoon.test.filters.testclasses;

public class FooLine {
    void simple() {
        System.out.println(0);
    }
    void loopBlock() {
        for (int i = 0; i < 10; i++) {
            System.out.println(i);
        }
    }
    void loopNoBlock() {
        for (int i = 0; i < 10; i++) 
            System.out.println(i);
    }
    void loopNoBody() {
        for (int i = 0; i < 10; i++) ;
    }
    void ifBlock() {
        System.out.println("if");
    }
    void ifNoBlock() {
        System.out.println("if");
    }
    void switchBlock() {
        switch ("test") {
            case "test":
                break;
            default:
                System.out.println("switch");
        }
    }
}
