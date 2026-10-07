package com.puppycrawl.tools.checkstyle.checks.naming.catchparametername;

import java.io.*;

final class InputCatchParameterNameSimpleOne2 {
    void ALL_UPPERCASE_METHOD() {}
    private static final int BAD__NAME = 3;
    void errorColumnAfterTabs() {}
    void veryLong() {}
    void toManyArgs(int aArg1, int aArg2, int aArg3, int aArg4, int aArg5, int aArg6, int aArg7, int aArg8, int aArg9) {}
}
