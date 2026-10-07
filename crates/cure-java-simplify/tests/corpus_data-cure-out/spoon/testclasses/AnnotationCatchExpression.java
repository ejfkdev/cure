package spoon.test.annotation.testclasses;

public class AnnotationCatchExpression {
    void m1(String s) {
        try {
            s.length();
        } catch (@CustomAnnotation(something =  "annotation string") NullPointerException | NumberFormatException e) {
            e.printStackTrace();
        }
    }
    void m2(String s) {
        try {
            s.length();
        } catch (NullPointerException e0) {
            e0.printStackTrace();
        } catch (@CustomAnnotation(something =  "annotation string") NumberFormatException e1) {
            e1.printStackTrace();
        }
    }
}
