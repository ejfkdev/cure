package com.google.googlejavaformat.java;

public class AospJavaFormatter extends JavaFormatterBase {
    public AospJavaFormatter() {
        super(JavaFormatterOptions.builder().style(JavaFormatterOptions.Style.AOSP).build());
    }
}
