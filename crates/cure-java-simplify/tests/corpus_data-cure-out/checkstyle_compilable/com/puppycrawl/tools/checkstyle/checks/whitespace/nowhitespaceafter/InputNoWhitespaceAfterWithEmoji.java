package com.puppycrawl.tools.checkstyle.checks.whitespace.nowhitespaceafter;

public class InputNoWhitespaceAfterWithEmoji {
    String txt = new String("sd🤩🎄😂 ");
    public String foo() {
        String[] s = {"🎄😂", "🎄😂12wq"};
        for (int i = 0; i < s.length; i++) {
            char[] c = "🤩🎄".toCharArray();
            char[] c2 = "🤩🎄".toCharArray();
        }
        return "😅🧐 dsad ";
    }
    public String foo2() {
        String str3 = (String) "🤩dsa😂adsad" + "😂sadsa😅🧐 ";
        return "  🎄😂  ";
    }
    public String foo3() {
        return "dsa😂a";
    }
    public String[] foo4() {
        return new String[] {"sd😂😅🧐", "👉🏻"};
    }
}
