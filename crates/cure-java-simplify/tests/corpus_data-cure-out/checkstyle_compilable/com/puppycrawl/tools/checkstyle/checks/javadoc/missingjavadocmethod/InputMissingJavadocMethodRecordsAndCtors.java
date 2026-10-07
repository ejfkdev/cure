package com.puppycrawl.tools.checkstyle.checks.javadoc.missingjavadocmethod;

public class InputMissingJavadocMethodRecordsAndCtors {
    public record MyRecord(Integer number) {
        private static int mNumber;
        public void setNumber(final int number) {
            mNumber = number;
        }
        public int getNumber() {
            return mNumber;
        }
        public void setNumber1() {
            mNumber = mNumber;
        }
    }
    public record MySecondRecord() {
        public MySecondRecord {}
    }
    public record MyThirdRecord() {
        public MyThirdRecord() {}
    }
    public void setNumber1() {}
}
