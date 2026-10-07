package com.puppycrawl.tools.checkstyle.checks.indentation.commentsindentation;

public class InputCommentsIndentationCheckWithEmoji {
    public void myMethod() {
        String breaks = "J🥳🥳VASd🥳A🥳";
    }
    public void test() {
        String a = "🥳";
    }
    String s = String.format(java.util.Locale.ENGLISH, " 🥳 🥳 🥳asdda   🥳🎄🎄  🎄🎄       ", " ");
    public void test2() {
        String a = "🥳";
        switch (a) {
            case "1":
                break;
            case "2":
            default:
                a = "🎄".toString();
        }
    }
    private void test3() {}
    private void test4() {
        String a = "🎄";
        a.toString().toLowerCase().charAt(0);
        try {
            assert a.equals("🎄") == true;
        } catch (Exception ex) {}
    }
    public void test5() {
        String someStr = "🎄🎄😅";
    }
    private void test6() {
        int b = Integer.parseInt("🎄🎄😅");
        double d;
        String x = "😁mkuhyg";
    }
}
