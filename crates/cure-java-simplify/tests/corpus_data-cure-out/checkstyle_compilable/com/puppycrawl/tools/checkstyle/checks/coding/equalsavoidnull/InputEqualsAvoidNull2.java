package com.puppycrawl.tools.checkstyle.checks.coding.equalsavoidnull;

public class InputEqualsAvoidNull2 {
    void foo() {
        String s = "";
        s.equals(s + s);
        s.equals("ab");
        s.equals(getInt() + s);
        s.equals(getInt() + getInt());
        s.endsWith("a");
        if (!s.equals("Hi[EOL]" + System.getProperty(""))) 
            foo();
    }
    int getInt() {
        return 0;
    }
    public void flagForEquals() {
        new Object().equals("hot pizza");
    }
}
