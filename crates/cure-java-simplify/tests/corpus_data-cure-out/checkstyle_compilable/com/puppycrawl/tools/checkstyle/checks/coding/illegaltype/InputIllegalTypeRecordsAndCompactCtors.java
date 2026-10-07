package com.puppycrawl.tools.checkstyle.checks.coding.illegaltype;

import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.TreeSet;

public class InputIllegalTypeRecordsAndCompactCtors {
    record MyTestRecord(LinkedHashMap<Integer, Integer> linkedHashMap) {
    }
    record MyTestRecord2(String string) implements Cloneable {
        static LinkedHashMap<Integer, Integer> lhm = new LinkedHashMap<>();
        public MyTestRecord2 {
            TreeSet<String> treeSet = new TreeSet<>();
        }
    }
    record MyTestRecord3(String str, TreeSet treeSet) {
        void foo(HashMap<Integer, Integer> hashMap) {}
    }
    record MyTestRecord4(int x, int y) {
        public MyTestRecord4(TreeSet treeSet) {
            this(4, 5);
            LinkedHashMap<Integer, Integer> linkedHashMap = new LinkedHashMap<>();
        }
    }
}
