package com.google.googlejavaformat.intellij;

import com.google.googlejavaformat.java.JavaFormatterOptions;
import com.google.googlejavaformat.java.JavaFormatterOptions.Style;
import java.util.Arrays;
import java.util.Objects;

enum UiFormatterStyle {
    GOOGLE("Default Google Java style", Style.GOOGLE), AOSP("Android Open Source Project (AOSP) style", Style.AOSP);
    private final String description;
    private final JavaFormatterOptions.Style style;
    UiFormatterStyle(String description, JavaFormatterOptions.Style style) {
        this.description = description;
        this.style = style;
    }
    @Override
  public String toString() {
        return description;
    }
    public JavaFormatterOptions.Style convert() {
        return style;
    }
    static UiFormatterStyle convert(JavaFormatterOptions.Style style) {
        return Arrays.stream(UiFormatterStyle.values()).filter((value) -> Objects.equals(value.style, style)).findFirst().get();
    }
}
