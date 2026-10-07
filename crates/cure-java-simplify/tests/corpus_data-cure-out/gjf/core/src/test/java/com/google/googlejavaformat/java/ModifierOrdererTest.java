package com.google.googlejavaformat.java;

import static com.google.common.truth.Truth.assertThat;
import com.google.common.collect.Range;
import java.util.Arrays;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public class ModifierOrdererTest {
    @Test
  public void simple() throws FormatterException {
        assertThat(ModifierOrderer.reorderModifiers("static abstract class InnerClass {}").getText()).isEqualTo("abstract static class InnerClass {}");
    }
    @Test
  public void comment() throws FormatterException {
        assertThat(ModifierOrderer.reorderModifiers("static/*1*/abstract/*2*/public").getText()).isEqualTo("public/*1*/abstract/*2*/static");
    }
    @Test
  public void everything() throws FormatterException {
        assertThat(ModifierOrderer.reorderModifiers("strictfp native synchronized volatile transient final static abstract private protected public").getText()).isEqualTo("public protected private abstract static final transient volatile synchronized native strictfp");
    }
    @Test
  public void everythingIncludingDefault() throws FormatterException {
        assertThat(ModifierOrderer.reorderModifiers("strictfp native synchronized volatile transient final static default abstract private protected public").getText()).isEqualTo("public protected private abstract default static final transient volatile synchronized native strictfp");
    }
    @Test
  public void subRange() throws FormatterException {
        String input = """
        class Test {
          static public int a;
          static public int b;
        }\
        """;
        String substring = "static public int a";
        int start = input.indexOf(substring);
        int end = start + substring.length();
        String output = ModifierOrderer.reorderModifiers(new JavaInput(input), Arrays.asList(Range.closedOpen(start, end))).getText();
        assertThat(output).contains("public static int a;");
        assertThat(output).contains("static public int b;");
    }
    @Test
  public void whitespace() throws FormatterException {
        String input = """
        class Test {
          static
          public int a;
        }\
        """;
        String substring = "static public int a";
        int start = input.indexOf(substring);
        int end = start + substring.length();
        assertThat(ModifierOrderer.reorderModifiers(new JavaInput(input), Arrays.asList(Range.closedOpen(start, end))).getText()).contains("public\n  static int a;");
    }
    @Test
  public void sealedClass() throws FormatterException {
        assertThat(ModifierOrderer.reorderModifiers("non-sealed sealed public").getText()).isEqualTo("public sealed non-sealed");
    }
}
