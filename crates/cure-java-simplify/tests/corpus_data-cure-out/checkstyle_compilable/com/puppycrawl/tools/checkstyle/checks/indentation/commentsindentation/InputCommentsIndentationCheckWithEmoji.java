package com.puppycrawl.tools.checkstyle.checks.indentation.commentsindentation;

public class InputCommentsIndentationCheckWithEmoji {
    public void myMethod() {}
    public void test() {}
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
    public void test5() {}
    private void test6() {
        int b = Integer.parseInt("🎄🎄😅");
    }
}
