package com.puppycrawl.tools.checkstyle.checks.coding.illegaltokentext;

public class InputIllegalTokenTextCheckCommentToken {
    public void methodWithPreviouslyIllegalTokens() {
        int i = 0;
        switch (i) {
            default:
                i--;
                i++;
                break;
        }
    }
    public native void nativeMethod();
    public void methodWithLiterals() {}
    public void methodWithLabels() {
        label:
            {
                anotherLabel:
                    do {
                        continue anotherLabel;
                    } while (false);
                break label;
            }
    }
}
