package com.puppycrawl.tools.checkstyle.checks.sizes.outertypenumber;

final class InputOuterTypeNumberSimple1 {
    public static final int MAX_ROWS = 2;
    private int mNumCreated1 = 0;
    private InputOuterTypeNumberSimple1() {}
    private void method() {
        int variable = 0;
    }
}

class InputOuterTypeNumberSimple3 {
    public void doSomething() {
        for (Object obj : new java.util.ArrayList()) {}
    }
}

enum MyEnum2 {
    ABC, XYZ;
    private int someMember;
}
