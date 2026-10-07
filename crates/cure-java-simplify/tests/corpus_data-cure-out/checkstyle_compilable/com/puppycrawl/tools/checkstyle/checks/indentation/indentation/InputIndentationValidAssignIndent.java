package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

public class InputIndentationValidAssignIndent {
    void foo(String[] args) {
        int i = 3;
        String line = mIndentCheck[getLineNo()];
        String line1 = getLine();
        line1 = getLine();
        int i1 = 1;
        Integer brace = candidate == SLIST ? candidate : null;
        AnInterfaceFooWithALongName f = new AnInterfaceFooWithALongName() {                                //indent:12 exp:>=12
                public void bar() {                                            //indent:16 exp:16
                }                                                              //indent:16 exp:16
            };
        AnInterfaceFooWithALongName f1 = new AnInterfaceFooWithALongName() {                              //indent:12 exp:>=12
                public void bar() {                                            //indent:16 exp:16
                }                                                              //indent:16 exp:16
            };
    }
    private interface AnInterfaceFooWithALongName {
        void bar();
    }
    private static final int SLIST = 1;
    private static final int parameters = 1;
    int candidate = 0;
    private String[] mIndentCheck = null;
    private InputIndentationValidAssignIndent function = null;
    int getLineNo() {
        return 1;
    }
    String getLine() {
        return "";
    }
    InputIndentationValidAssignIndent lastArgument() {
        return this;
    }
}
