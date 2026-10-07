package com.puppycrawl.tools.checkstyle.checks.blocks.leftcurly;

import org.w3c.dom.Node;

public class InputLeftCurlyTestRecordsAndCompactCtors {
    record MyTestRecord(String string, Record rec) {
        private boolean inRecord(Object obj) {
            int value = 0;
            if (obj instanceof Integer i) {
                value = i;
            }
            return value > 10;
        }
    }
    record MyTestRecord2() {
        MyTestRecord2(String one, String two, String three) {
            this();
        }
    }
    record MyTestRecord3(Integer i, Node node) {
        public MyTestRecord3 {
            int x = 5;
        }
        public static void main(String... args) {
            System.out.println("works!");
        }
    }
    record MyTestRecord4() {
    }
    record MyTestRecord5() {
        static MyTestRecord mtr = new MyTestRecord("my string", new MyTestRecord4());
    }
    class MyTestClass {
        private MyTestRecord mtr = new MyTestRecord("my string", new MyTestRecord4());
    }
}
