package com.puppycrawl.tools.checkstyle.checks.naming.staticvariablename;

class InputStaticVariableName1C {
    public void doSomething() {
        for (Object O : new java.util.ArrayList()) {}
    }
    void veryLong() {}
    void toManyArgs(int aArg1, int aArg2, int aArg3, int aArg4, int aArg5, int aArg6, int aArg7, int aArg8, int aArg9) {}
}

enum MyEnum1 {
    ABC, XYZ;
    private int someMember;
}
