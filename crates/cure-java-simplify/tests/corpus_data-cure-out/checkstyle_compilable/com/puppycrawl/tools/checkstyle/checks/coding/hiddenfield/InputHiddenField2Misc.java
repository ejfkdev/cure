package com.puppycrawl.tools.checkstyle.checks.coding.hiddenfield;

class InputHiddenField2Misc {
    abstract class InputHiddenFieldBug10845122 {
        String x;
        public abstract void methodA(String x);
    }
    class Bug33709462 {
        private int xAxis;
        public void setxAxis(int xAxis) {
            this.xAxis = xAxis;
        }
    }
    class OneLetterField2 {
        int i;
        void setI(int i) {
            this.i = i;
        }
        enum Inner {
        }
    }
    class DuplicateFieldFromPreviousClass2 {
        public void method() {
            int i = 0;
        }
    }
    class NestedEnum2 {
        enum Test {
            A, B, C;
            int i;
        }
        void method(int i) {}
    }
}
