package com.puppycrawl.tools.checkstyle.checks.whitespace.parenpad;

import java.util.HashMap;
import org.w3c.dom.Node;

public class InputParenPadCheckRecords {
    record MyRecord1( ) {
        MyRecord1(int x) {
            this();
        }
        public MyRecord1 {
            bar(1);
        }
        static int n;
        public void fun() {
            bar(1);
        }
        public void bar(int k) {
            while (k > 0) {}
        }
        public void fun2() {
            switch (n) {
                case 2:
                    bar(n);
                default:
                    break;
            }
        }
    }
    record MyRecord2( String s) {
    }
    record MyRecord4( String s, String ...varargs ) {
    }
    record MyRecord6( String[] strArr) {
    }
    record MyRecord7(HashMap<String, Node> hashMap ) {
    }
}
