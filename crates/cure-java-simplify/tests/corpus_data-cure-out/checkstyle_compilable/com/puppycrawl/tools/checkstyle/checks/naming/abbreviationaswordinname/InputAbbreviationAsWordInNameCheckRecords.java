package com.puppycrawl.tools.checkstyle.checks.naming.abbreviationaswordinname;

import org.w3c.dom.Node;

public class InputAbbreviationAsWordInNameCheckRecords {
    class myCLASS {
        int INTEGER = 2;
        void METHOD() {}
        public myCLASS(String STRING) {}
    }
    record myRECORD1(String STRING) {
        void METHOD() {}
        public myRECORD1() {
            this("string");
        }
    }
    record myRECORD2() {
        static int INTEGER = 6;
        public myRECORD2 {}
    }
    record myRECORD3(String STRING, int INTEGER, Node[] NODES) {
    }
}
