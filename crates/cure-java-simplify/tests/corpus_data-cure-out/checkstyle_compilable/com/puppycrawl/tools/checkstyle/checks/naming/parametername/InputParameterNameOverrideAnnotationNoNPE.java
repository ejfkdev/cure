package com.puppycrawl.tools.checkstyle.checks.naming.parametername;

class InputParameterNameOverrideAnnotationNoNPE {
    void InputParameterNameOverrideAnnotationNoNPEMethod(int a, int b) {}
    void InputParameterNameOverrideAnnotationNoNPEMethod2(int a, int b) {}
}

class Test extends InputParameterNameOverrideAnnotationNoNPE {
    @Override void InputParameterNameOverrideAnnotationNoNPEMethod(int a, int b) {}
    @java.lang.Override void InputParameterNameOverrideAnnotationNoNPEMethod2(int a, int b) {}
}
