package com.puppycrawl.tools.checkstyle.grammar.java14;

public class InputJava14EscapedS {
    public static void main(String[] args) {
        String s5 = "            \\n\\s\\s            \\s\\s\\n ";
    }
    static void test2() {
        assert true;
        assert false;
    }
}
