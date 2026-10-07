package com.puppycrawl.tools.checkstyle.checks.naming.localvariablename;

import java.io.*;

class InputLocalVariableName2one {
    public void doSomething() {
        for (Object O : new java.util.ArrayList()) {}
        for (int k_ : new int[] {}) {}
    }
}

enum InputLocalVariableNameEnum1 {
    ABC, XYZ;
    private int someMember;
}
