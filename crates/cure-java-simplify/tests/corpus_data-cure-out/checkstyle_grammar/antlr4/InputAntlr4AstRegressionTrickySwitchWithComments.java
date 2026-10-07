package com.puppycrawl.tools.checkstyle.grammar.antlr4;

public class InputAntlr4AstRegressionTrickySwitchWithComments {
    private static void fooSwitch() {
        switch ("") {
            case "0":
            case "1":
                foo1();
                break;
            case "2":
                foo1();
                break;
            case "3":
                foo1();
                break;
            case "5":
                foo1();
            case "6":
                int k = 7;
            case "7":
                {}
            case "8":
                break;
            case "9":
                foo1();
            case "10":
                {}
            case "11":
                {}
            case "28":
                {}
            case "12":
                {
                    int i;
                }
            case "13":
                {}
            case "14":
                {}
            case "15":
                {
                    foo1();
                }
            case "16":
                {
                    int a;
                }
            case "17":
                {
                    int a;
                }
            case "18":
                {
                    System.lineSeparator();
                }
            case "19":
            case "20":
            case "21":
            default:
                break;
        }
    }
    private static void foo1() {
        switch (1) {
            case 0:
            case 1:
                int b = 10;
            default:
        }
    }
    public void fooDotInCaseBlock() {
        int i = 0;
        String s = "";
        switch (i) {
            case -2:
                i++;
            case 0:
                s.indexOf("ignore");
            case -1:
                s.indexOf("no way");
            case 1:
            case 2:
                i--;
            case 3:
                {}
        }
        String breaks = "</table>";
    }
    public void foo2() {
        switch (1) {
            case 1:
            default:
        }
    }
    public void foo3() {
        switch (1) {
            case 1:
            default:
        }
    }
    public void foo4() {
        switch (1) {
            case 1:
                int b;
            default:
        }
    }
    public void foo5() {
        switch (1) {
            case 1:
                int b;
            default:
        }
    }
    public void foo6() {
        switch (1) {
            case 1:
                int b;
            default:
        }
    }
    public void foo7() {
        switch (2) {
            case 1:
            case 2:
                foo1();
            case 3:
            case 4:
            case 5:
                "".toString().toString().toString();
            default:
        }
    }
    public void foo8() {
        switch (2) {
            case 1:
            case 2:
                foo1();
            case 3:
                "".toString().toString().toString();
            case 4:
            default:
        }
    }
    public void foo9() {
        switch (5) {
            case 1:
            case 2:
        }
    }
    public void foo10() {
        switch (5) {
            case 1:
            default:
        }
    }
    public void foo11() {
        switch (5) {
            case 1:
            case 2:
        }
    }
    public void foo12() {
        switch (5) {
            case 1:
        }
    }
}
