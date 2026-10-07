package com.puppycrawl.tools.checkstyle.checks.coding.magicnumber;

public class InputMagicNumberIgnoreHashCodeMethod1 {
    public void magicMethod() {
        int int_var1;
        double double_var1;
        int[] int_array = new int[2];
        int_var1 = 3;
        int_var1 += 1;
        for (int i = 0; i < 2; i++) ;
        if (1.0 < 2.0) ;
        int int_magic1 = 3_000;
        double double_magic1 = 1.5_0;
        int int_magic2 = 7;
        int_array = new int[3];
        int_magic1 += 3;
        double_magic1 *= 1.5;
        for (int j = 3; j < 5; j += 3) {
            int_magic1++;
        }
        if (int_magic1 < 3) {
            int_magic1 = int_magic1 + 3;
        }
        long longHexVar17 = 0X11l;
    }
}

interface Blah2IgnoreHashCodeMethod1 {
    int LOW = 5;
    int HIGH = 78;
}

class ArrayMagicTestIgnoreHashCodeMethod1 {
    private static final int[] NONMAGIC = {3};
    private int[] magic = {3};
    private static final int[][] NONMAGIC2 = {{1}, {2}, {3}};
}
