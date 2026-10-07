package com.puppycrawl.tools.checkstyle.checks.coding.magicnumber;

public class InputMagicNumberMagicNumberWithUnnamedVariables {
    void test() {
        int _ = 9;
        int _ = 1;
        Integer _ = 17;
        double _ = 3.1415;
    }
}
