package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

public class InputIndentationDoWhile {
    void test() {
        int a = 9;
        do 
            expression(); while (a < 11);
        do 
            expression(); while (a < 11);
        do 
            expression(); while (a < 11);
        do 
            expression(); while (a < 11);
        do 
            expression(); while (a < 11);
    }
    void expression() {}
}
