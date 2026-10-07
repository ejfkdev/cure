package com.puppycrawl.tools.checkstyle.checks.sizes.parameternumber;

final class InputParameterNumberSimple4 {
    int test1(int badFormat1, int badFormat2, final int badFormat3) throws java.lang.Exception {
        return 0;
    }
    void toManyArgs(int aArg1, int aArg2, int aArg3, int aArg4, int aArg5, int aArg6, int aArg7, int aArg8, int aArg9) {}
}
