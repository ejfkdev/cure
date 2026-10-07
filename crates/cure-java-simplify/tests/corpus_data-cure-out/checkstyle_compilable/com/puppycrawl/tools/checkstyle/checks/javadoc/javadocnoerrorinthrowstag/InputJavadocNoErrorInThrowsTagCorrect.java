package com.puppycrawl.tools.checkstyle.checks.javadoc.javadocnoerrorinthrowstag;

import java.io.IOException;

public class InputJavadocNoErrorInThrowsTagCorrect {
    void validExceptions() throws IOException {}
    void explicitRootError() {
        throw new Error("explicit failure");
    }
    void explicitQualifiedErrorTag() {
        throw new AssertionError("explicit failure");
    }
    void explicitQualifiedErrorInBody() {
        throw new InputJavadocNoErrorInThrowsTagCorrect.ExplicitThrownError("explicit failure");
    }
    InputJavadocNoErrorInThrowsTagCorrect() {
        throw new LinkageError("explicit failure");
    }
    record ValidRecord(String value) {
        ValidRecord {
            throw new AssertionError("explicit failure");
        }
    }
    void explicitErrorInCatch() {
        try {
            throw new RuntimeException("regular failure");
        } catch (RuntimeException ex) {
            throw new AssertionError("explicit failure");
        }
    }
    void explicitErrorInFinally() {
        try {
            return;
        } finally {
            throw new AssertionError("explicit failure");
        }
    }
    void lowercaseSuffix() throws Customerror {}
    void missingThrowsName() {}
    static class Customerror extends Exception {
    }
    static class ExplicitThrownError extends Error {
        ExplicitThrownError(String message) {
            super(message);
        }
    }
}
