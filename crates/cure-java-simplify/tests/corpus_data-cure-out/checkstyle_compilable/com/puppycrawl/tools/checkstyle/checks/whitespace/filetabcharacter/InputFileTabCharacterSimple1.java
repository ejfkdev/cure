package com.puppycrawl.tools.checkstyle.checks.whitespace.filetabcharacter;

final class InputFileTabCharacterSimple1 {
    public static final int badConstant = 2;
    public static final int MAX_ROWS = 2;
    private static int badStatic = 2;
    private static int sNumCreated = 0;
    private int badMember = 2;
    private int mNumCreated1 = 0;
    protected int mNumCreated2 = 0;
    private int[] mInts = {1, 2, 3, 4};
    public static int sTest1;
    protected static int sTest3;
    static int sTest2;
    int mTest1;
    public int mTest2;
    int test1(int badFormat1, int badFormat2, final int badFormat3) throws Exception {
        return 0;
    }
    private void longMethod() {}
    private InputFileTabCharacterSimple1() {}
    private void localVariables() {
        for (int k = 0; k < 1; k++) {
            String innerBlockVariable = "";
        }
        for (int I = 0; I < 1; I++) {
            String InnerBlockVariable = "";
        }
    }
    void ALL_UPPERCASE_METHOD() {}
    private static final int BAD__NAME = 3;
    void errorColumnAfterTabs() {
        int tab5 = 1;
    }
    void veryLong() {}
    void toManyArgs(int aArg1, int aArg2, int aArg3, int aArg4, int aArg5, int aArg6, int aArg7, int aArg8, int aArg9) {}
}

class InputSimple3 {
    public void doSomething() {
        for (Object O : new java.util.ArrayList()) {}
    }
}

enum MyEnum2 {
    ABC, XYZ;
    private int someMember;
}
