package com.puppycrawl.tools.checkstyle.grammar.java16;

public class InputPatternVariableWithModifiers {
    static void method1(Object args) {
        Object o = args;
        if (!(o instanceof String[] s)) 
            if (!(o instanceof Integer[] i)) 
                if (!(o instanceof Character[] c)) 
                    if (o instanceof Double[] d) {}
    }
    static void method2(Object args) {
        if (args instanceof String[] s) {
            s = new String[] {"modified"};
            System.out.println(s[0]);
        }
    }
}
