package com.puppycrawl.tools.checkstyle.checks.whitespace.emptylineseparator;

import java.util.*;
import java.io.*;

public class InputEmptyLineSeparatorMultipleEmptyLines {
    private int counter;
    private Object obj = null;
    private int k;
    private static void foo() {}
    private static void foo1() {}
}

class Test2 {
    void testFor() {
        for (int i = 1; i < 5; i++) {}
        for (int i = 1; i < 5; i++) ;
    }
}
