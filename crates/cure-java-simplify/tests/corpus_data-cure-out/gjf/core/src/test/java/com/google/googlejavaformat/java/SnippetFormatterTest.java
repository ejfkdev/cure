package com.google.googlejavaformat.java;

import static com.google.common.truth.Truth.assertThat;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.Range;
import com.google.googlejavaformat.java.SnippetFormatter.SnippetKind;
import java.util.List;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public class SnippetFormatterTest {
    @Test
  public void expression() throws FormatterException {
        String input = "x\n=42";
        assertThat(new SnippetFormatter().format(SnippetKind.EXPRESSION, input, ImmutableList.of(Range.closedOpen(0, input.length())), 4, false)).containsExactly(Replacement.create(1, 2, " "), Replacement.create(3, 3, " "));
    }
    @Test
  public void statement() throws FormatterException {
        String input = "int x\n=42;";
        assertThat(new SnippetFormatter().format(SnippetKind.STATEMENTS, input, ImmutableList.of(Range.closedOpen(0, input.length())), 4, false)).containsExactly(Replacement.create(5, 6, " "), Replacement.create(7, 7, " "));
    }
    @Test
  public void classMember() throws FormatterException {
        String input = "void f() {\n}";
        assertThat(new SnippetFormatter().format(SnippetKind.CLASS_BODY_DECLARATIONS, input, ImmutableList.of(Range.closedOpen(0, input.length())), 4, false)).containsExactly(Replacement.create(10, 11, ""));
    }
    @Test
  public void compilation() throws FormatterException {
        String input = "/** a\nb*/\nclass Test {\n}";
        assertThat(new SnippetFormatter().format(SnippetKind.COMPILATION_UNIT, input, ImmutableList.of(Range.closedOpen(input.indexOf("class"), input.length())), 4, false)).containsExactly(Replacement.create(22, 23, ""));
    }
    @Test
  public void compilationWithComments() throws FormatterException {
        String input = "/** a\nb*/\nclass Test {\n}";
        assertThat(new SnippetFormatter().format(SnippetKind.COMPILATION_UNIT, input, ImmutableList.of(Range.closedOpen(0, input.length())), 4, true)).containsExactly(Replacement.create(0, 24, "/** a b */\nclass Test {}\n"));
    }
}
