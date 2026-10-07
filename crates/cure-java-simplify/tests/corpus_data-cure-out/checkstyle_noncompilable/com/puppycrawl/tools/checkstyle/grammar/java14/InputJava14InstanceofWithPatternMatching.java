package com.puppycrawl.tools.checkstyle.grammar.java14;

import java.util.*;
import java.util.Arrays;
import java.util.Locale;

public class InputJava14InstanceofWithPatternMatching {
    public class Keyboard {
        private String model = null;
        private final int price;
        public Keyboard() {
            price = 0;
        }
        @Override
        public boolean equals(Object obj) {
            return obj instanceof Keyboard other && model.equals(other.model) && price == other.price;
        }
        public int getPrice() {
            return price;
        }
        public String getModel() {
            return model;
        }
        public void setModel(String model) {
            this.model = model;
        }
    }
    static {
        Object o = "";
        if (o instanceof String s) {
            System.out.println(s.toLowerCase(Locale.forLanguageTag(s)));
            boolean stringCheck = "test".equals(s);
        }
        if (o instanceof Integer count) {
            int value = count.byteValue();
            if (count.equals(value)) {
                value = 25;
            }
        }
        if (o instanceof String) {
            System.out.println("Yes");
        }
        String[] someString1 = {"some string"};
        if (someString1 instanceof Object[]) {
            System.out.println(Arrays.toString(someString1));
        }
        String[][] someString2 = new String[2][3];
        if (someString2 instanceof Object[][]) {
            System.out.println(Arrays.toString(someString2));
        }
        String[][][] someString3 = new String[3][4][5];
        if (someString3 instanceof Object[][]) {
            System.out.println(Arrays.toString(someString3));
        }
    }
    interface VoidPredicate {
        public boolean get();
    }
    public void t(Object o1, Object o2) {
        Object b;
        Object c;
        b = ((VoidPredicate) (() -> o1 instanceof String s)).get();
        List<Integer> arrayList = new ArrayList<Integer>();
        if (arrayList instanceof ArrayList<Integer> ai) {
            System.out.println("Blah");
        }
        if (!(o1 instanceof String k)) {
            return;
        }
        if (!(o1 instanceof String s4)) {
            return;
        }
        boolean result = o1 instanceof String a1 ? o1 instanceof String a2 : !(o1 instanceof String a3);
        if (!(o1 instanceof String s) ? false : s.length() > 0) {
            System.out.println("done");
        }
        if (o1 instanceof String s && s.length() > 0) {
            System.out.println("done");
        }
        if (!(!(o1 instanceof String s) || !(o2 instanceof Integer i))) {
            s.length();
            i.intValue();
        }
        if (!(!(o1 instanceof String s) || !(o2 instanceof Integer i))) {
            s.length();
            i.intValue();
        }
        if (o1 instanceof String s && o2 instanceof Integer in) {
            s.length();
            in.intValue();
        }
        L1:
            {
                if (o1 instanceof String s) {
                    s.length();
                } else {
                    break L1;
                }
                s.length();
            }
        L2:
            for (; !(o1 instanceof String s); ) {}
        s.length();
        while (!(o1 instanceof String s)) {
            L3:
                break L3;
        }
        while (o1 instanceof String str) {
            str.length();
        }
        if (!new VoidPredicate() { public boolean get() { return o1 instanceof String str
                && !str.isEmpty();} }.get()) {
            throw new AssertionError();
        }
        if (!((VoidPredicate) (() -> o1 instanceof String str && !str.isEmpty())).get()) {
            throw new AssertionError();
        }
        if (o1 instanceof String j && j.length() == 5 && o2 instanceof Integer z && z == 42) {
            System.out.println(j);
            System.out.println(z);
        }
        int x = o1 instanceof String j ? j.length() : 2;
        x = !(o1 instanceof String j) ? 2 : j.length();
        for (; o1 instanceof String j; j.length()) {
            System.out.println(j);
        }
        String formatted = o1 instanceof Integer i ? String.format("int %d", i) : o1 instanceof Byte by ? String.format("byte %d", by) : o1 instanceof Long l ? String.format("long %d", l) : o1 instanceof Double d ? String.format("double %f", d) : o1 instanceof String s ? String.format("String %s", s) : String.format("Something else " + o1.toString());
        do {
            L4:
                break L4;
        } while (!(o1 instanceof String s));
        do {
            L4:
                break L4;
        } while (!(o1 instanceof java.lang.Double log));
    }
    static class Pattern_Simple {
        public static void test(Object o) {}
    }
    static class Pattern_Lambda {
        public static void test(Object o) {
            if (o instanceof String s) {
                Runnable r = () -> {
                    s.length();
                };
            }
        }
    }
}
