package com.puppycrawl.tools.checkstyle.checks.coding.magicnumber;

public class InputMagicNumberIgnoreNone2 {
    long l = 0xffffffffL;
    public static final int CONST_PLUS_THREE = 3;
    public static final int CONST_MINUS_TWO = -2;
    private int mPlusThree = 3;
    private int mMinusTwo = -2;
    private double mPlusDecimal = 3.5;
    private double mMinusDecimal = -2.5;
    private int hexIntMinusOne = 0xffffffff;
    private long hexLongMinusOne = 0xffffffffffffffffL;
    private long hexIntMinValue = 0x80000000;
    private long hexLongMinValue = 0x8000000000000000L;
    private int octalIntMinusOne = 037777777777;
    private long octalLongMinusOne = 01777777777777777777777L;
    private long octalIntMinValue = 020000000000;
    private long octalLongMinValue = 01000000000000000000000L;
    public static final int TESTINTVAL = (byte) 0x80;
    public static final java.util.List MYLIST = new java.util.ArrayList() {
        public int size() {

            return 378; // violation ''378' is a magic number'
        }
    };
    public final double SpecialSum = 2 + 1e10, SpecialDifference = 4 - java.lang.Math.PI;
    public final Integer DefaultInit = new Integer(27);
    public final int SpecsPerDay = 1440 * 60, SpecialRatio = 4 / 3;
    public final javax.swing.border.Border StdBorder = javax.swing.BorderFactory.createEmptyBorder(3, 3, 3, 3);
    enum MyEnum2IgnoreNone2 {
        A_3(3), B_3(54);
        private MyEnum2IgnoreNone2(int value) {}
    }
}
