package spoon.test.template.testclasses.logger;

public class Logger {
    public static void enter(String className, String methodName) {
        System.out.println("enter: " + className + " - " + methodName);
    }
    public static void exit(String methodName) {
        System.err.println("exit: " + methodName);
    }
}
