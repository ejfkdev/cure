package com.puppycrawl.tools.checkstyle.checks.coding.hiddenfield;

import java.util.Locale;

public class InputHiddenFieldRecords {
    public record MyRecord1() {
        private static int myHiddenInt = 2;
        public MyRecord1 {
            int myHiddenInt = 5;
        }
        MyRecord1(String string) {
            this();
            int myHiddenInt = 6;
        }
    }
    static class MyClass {
        private static int hiddenField = 5;
        MyClass(String string) {
            int hiddenField = 10;
        }
        static final Object OBJ = "";
        static String hiddenStaticField = "hiddenStaticField";
        static {
            if (OBJ instanceof String hiddenStaticField) {
                System.out.println(hiddenStaticField.toLowerCase(Locale.forLanguageTag(hiddenStaticField)));
                boolean stringCheck = "test".equals(hiddenStaticField);
            }
        }
    }
    public record Keyboard() {
        private static String model = null;
        private static int price = 2;
        public boolean doStuff(Object f) {
            return f instanceof Float price && price.floatValue() > 0 && model != null && price.intValue() == 5;
        }
    }
    record MyRecord13(String string, Integer x) {
        void foo() {
            Integer x = 8;
        }
        void foo2() {
            String string = "string";
        }
    }
    class MyClass13 {
        Integer x = 7;
        String string = "string";
        void foo() {
            Integer x = 8;
        }
        void foo2() {
            String string = "string";
        }
    }
    record MyTestRecord3(String str, Locale treeSet) {
        void foo(Locale hashMap) {}
    }
    record MyTestRecord4(int x, int y) {
        public MyTestRecord4(Locale treeSet) {
            this(4, 5);
        }
    }
}
