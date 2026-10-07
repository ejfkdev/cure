package spoon.test.prettyprinter.testclasses;

public class MissingVariableDeclaration {
    int testedField;
    void failingMethod() {
        testedField = 1;
    }
}
