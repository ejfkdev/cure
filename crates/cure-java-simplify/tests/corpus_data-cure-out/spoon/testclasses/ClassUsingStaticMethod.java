package spoon.test.prettyprinter.testclasses;

import static spoon.test.prettyprinter.testclasses.ClassWithStaticMethod.findFirst;

public class ClassUsingStaticMethod {
    public void callFindFirst() {
        findFirst();
        new ClassWithStaticMethod().notStaticFindFirst();
    }
}
