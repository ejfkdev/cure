package com.puppycrawl.tools.checkstyle.checks.whitespace.whitespacearound;

class InputWhitespaceAroundBracesPart1 {
    boolean condition() {
        return false;
    }
    void testDoWhile() {
        do {
            testDoWhile();
        } while (condition());
        do 
            testDoWhile(); while (condition());
    }
    void testWhile() {
        while (condition()) {
            testWhile();
        }
        while (condition()) ;
        while (condition()) 
            testWhile();
        while (condition()) 
            if (condition()) 
                testWhile();
    }
    void testFor() {
        for (int i = 1; i < 5; i++) {
            testFor();
        }
        for (int i = 1; i < 5; i++) ;
        for (int i = 1; i < 5; i++) 
            testFor();
        for (int i = 1; i < 5; i++) 
            if (i > 2) 
                testFor();
    }
    public void testIf() {
        if (condition()) {
            testIf();
        } else if (condition()) {
            testIf();
        } else {
            testIf();
        }
        if (condition()) ;
        if (condition()) 
            testIf();
        if (condition()) 
            testIf(); else 
            testIf();
        if (condition()) 
            testIf(); else {
            testIf();
        }
        if (condition()) {
            testIf();
        } else 
            testIf();
        if (condition()) 
            if (condition()) 
                testIf();
    }
}
