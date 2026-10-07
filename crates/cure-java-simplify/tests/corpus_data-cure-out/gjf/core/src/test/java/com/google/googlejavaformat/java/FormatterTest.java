package com.google.googlejavaformat.java;

import static com.google.common.truth.Truth.assertThat;
import static com.google.common.truth.Truth.assertWithMessage;
import static java.nio.charset.StandardCharsets.UTF_8;
import static org.junit.Assert.assertThrows;
import com.google.common.io.CharStreams;
import com.google.googlejavaformat.java.JavaFormatterOptions.Style;
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.PrintWriter;
import java.io.StringWriter;
import java.nio.file.Files;
import java.nio.file.Path;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public final class FormatterTest {
    @Rule public TemporaryFolder testFolder = new TemporaryFolder();
    @Test
  public void testFormatAosp() throws Exception {
        Path path = testFolder.newFolder().toPath().resolve("A.java");
        Files.writeString(path, "class A{void b(){while(true){weCanBeCertainThatThisWillEndUpGettingWrapped(because, it, is, just, so, very, very, very, very, looong);}}}");
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        Main main = new Main(new PrintWriter(out, true), new PrintWriter(err, true), System.in);
        String[] args = {"--aosp", path.toString()};
        assertThat(main.format(args)).isEqualTo(0);
        assertThat(out.toString()).isEqualTo("""
        class A {
            void b() {
                while (true) {
                    weCanBeCertainThatThisWillEndUpGettingWrapped(
                            because, it, is, just, so, very, very, very, very, looong);
                }
            }
        }
        """);
    }
    @Test
  public void testFormatNonJavaFiles() throws Exception {
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        Main main = new Main(new PrintWriter(out, true), new PrintWriter(err, true), System.in);
        assertThat(main.format("foo.go")).isEqualTo(0);
        assertThat(err.toString()).contains("Skipping non-Java file: foo.go");
        assertThat(main.format("Foo.java")).isEqualTo(1);
        assertThat(err.toString()).contains("Foo.java: could not read file: ");
    }
    @Test
  public void testFormatStdinStdoutWithDashFlag() throws Exception {
        InputStream in = new ByteArrayInputStream("""
        class Foo{
        void f
        () {
        }
        }
        """.getBytes(UTF_8));
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        InputStream oldIn = System.in;
        System.setIn(in);
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(err, true), System.in).format("-")).isEqualTo(0);
        assertThat(out.toString()).isEqualTo("""
        class Foo {
          void f() {}
        }
        """);
        System.setIn(oldIn);
    }
    @Test
  public void testFormatLengthUpToEOF() throws Exception {
        String input = """
        class Foo{
        void f
        () {
        }
        }\n\n\n\n\n
        """;
        Path path = testFolder.newFolder().toPath().resolve("Foo.java");
        Files.writeString(path, input);
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        Main main = new Main(new PrintWriter(out, true), new PrintWriter(err, true), System.in);
        String[] args = {"--offset", "0", "--length", String.valueOf(input.length()), path.toString()};
        assertThat(main.format(args)).isEqualTo(0);
        assertThat(out.toString()).isEqualTo("""
        class Foo {
          void f() {}
        }
        """);
    }
    @Test
  public void testFormatLengthOutOfRange() throws Exception {
        Path path = testFolder.newFolder().toPath().resolve("Foo.java");
        Files.writeString(path, "class Foo{}\n");
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        Main main = new Main(new PrintWriter(out, true), new PrintWriter(err, true), System.in);
        String[] args = {"--offset", "0", "--length", "9999", path.toString()};
        assertThat(main.format(args)).isEqualTo(1);
        assertThat(err.toString()).contains("error: invalid offset (0) or length (9999); offset + length (9999)");
    }
    @Test
  public void testFormatOffsetOutOfRange() throws Exception {
        Path path = testFolder.newFolder().toPath().resolve("Foo.java");
        Files.writeString(path, "class Foo{}\n");
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        Main main = new Main(new PrintWriter(out, true), new PrintWriter(err, true), System.in);
        String[] args = {"--offset", "9998", "--length", "1", path.toString()};
        assertThat(main.format(args)).isEqualTo(1);
        assertThat(err.toString()).contains("error: invalid offset (9998) or length (1); offset + length (9999)");
    }
    @Test
  public void blankInClassBody() throws FormatterException {
        assertThat(new Formatter().formatSource("""
        package test;
        class T {

        }
        """)).isEqualTo("""
        package test;

        class T {}
        """);
    }
    @Test
  public void blankInClassBodyNoTrailing() throws FormatterException {
        assertThat(new Formatter().formatSource("""
        package test;
        class T {

        }\
        """)).isEqualTo("""
        package test;

        class T {}
        """);
    }
    @Test
  public void docCommentTrailingBlank() throws FormatterException {
        assertThat(new Formatter().formatSource("""
        class T {
        /** asd */

        int x;
        }\
        """)).isEqualTo("""
        class T {
          /** asd */
          int x;
        }
        """);
    }
    @Test
  public void blockCommentInteriorTrailingBlank() throws FormatterException {
        assertThat(new Formatter().formatSource("""
        class T {
        /*
        * asd
        * fgh
        */

        int x;
        }\
        """)).isEqualTo("""
        class T {
          /*
           * asd
           * fgh
           */

          int x;
        }
        """);
    }
    @Test
  public void blockCommentTrailingBlank() throws FormatterException {
        assertThat(new Formatter().formatSource("""
        class T {
        /* asd */

        int x;
        }\
        """)).isEqualTo("""
        class T {
          /* asd */

          int x;
        }
        """);
    }
    @Test
  public void lineCommentTrailingBlank() throws FormatterException {
        assertThat(new Formatter().formatSource("""
        class T {
        // asd

        int x;
        }\
        """)).isEqualTo("""
        class T {
          // asd

          int x;
        }
        """);
    }
    @Test
  public void lineCommentTrailingThinSpace() throws FormatterException {
        assertThat(new Formatter().formatSource("class T {\n  // asd \n}\n")).isEqualTo("""
        class T {
          // asd
        }
        """);
    }
    @Test
  public void noBlankAfterLineCommentWithInteriorBlankLine() throws FormatterException {
        assertThat(new Formatter().formatSource("""
        class T {
        // asd

        // dsa
        int x;
        }\
        """)).isEqualTo("""
        class T {
          // asd

          // dsa
          int x;
        }
        """);
    }
    @Test
  public void badConstructor() throws FormatterException {
        assertThat(new Formatter().formatSource("class X { Y() {} }")).isEqualTo("""
        class X {
          Y() {}
        }
        """);
    }
    @Test
  public void voidMethod() throws FormatterException {
        assertThat(new Formatter().formatSource("class X { void Y() {} }")).isEqualTo("""
        class X {
          void Y() {}
        }
        """);
    }
    private static final String UNORDERED_IMPORTS = """
      import com.google.common.base.Preconditions;

      import static org.junit.Assert.fail;
      import static com.google.truth.Truth.assertThat;

      import org.junit.runners.JUnit4;
      import org.junit.runner.RunWith;

      import java.util.List;

      import javax.annotation.Nullable;
      """;
    @Test
  public void importsNotReorderedByDefault() throws FormatterException {
        assertThat(new Formatter().formatSource("package com.google.example;\n" + UNORDERED_IMPORTS + "public class ExampleTest {}\n")).isEqualTo("package com.google.example;\n\n" + UNORDERED_IMPORTS + "\npublic class ExampleTest {}\n");
    }
    @Test
  public void importsFixedIfRequested() throws FormatterException {
        assertThat(new Formatter().formatSourceAndFixImports("package com.google.example;\n" + UNORDERED_IMPORTS + """
            public class ExampleTest {
              @Nullable List<?> xs;
            }
            """)).isEqualTo("""
        package com.google.example;

        import java.util.List;
        import javax.annotation.Nullable;

        public class ExampleTest {
          @Nullable List<?> xs;
        }
        """);
    }
    @Test
  public void importOrderingWithoutFormatting() throws IOException, UsageException {
        importOrdering("--fix-imports-only", "com/google/googlejavaformat/java/testimports/A.imports-only");
    }
    @Test
  public void importOrderingAndFormatting() throws IOException, UsageException {
        importOrdering(null, "com/google/googlejavaformat/java/testimports/A.imports-and-formatting");
    }
    @Test
  public void formattingWithoutImportOrdering() throws IOException, UsageException {
        importOrdering("--skip-sorting-imports", "com/google/googlejavaformat/java/testimports/A.formatting-and-unused-import-removal");
    }
    @Test
  public void formattingWithoutRemovingUnusedImports() throws IOException, UsageException {
        importOrdering("--skip-removing-unused-imports", "com/google/googlejavaformat/java/testimports/A.formatting-and-import-sorting");
    }
    private void importOrdering(String sortArg, String outputResourceName) throws IOException, UsageException {
        Path path = testFolder.newFolder().toPath().resolve("Foo.java");
        String input = getResource("com/google/googlejavaformat/java/testimports/A.input");
        String expectedOutput = getResource(outputResourceName);
        Files.writeString(path, input);
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        Main main = new Main(new PrintWriter(out, true), new PrintWriter(err, true), System.in);
        String[] args = sortArg != null ? new String[] {sortArg, "-i", path.toString()} : new String[] {"-i", path.toString()};
        main.format(args);
        assertThat(err.toString()).isEmpty();
        assertThat(out.toString()).isEmpty();
        assertThat(new String(Files.readAllBytes(path), UTF_8)).isEqualTo(expectedOutput);
    }
    private String getResource(String resourceName) throws IOException {
        try (InputStream stream = getClass().getClassLoader().getResourceAsStream(resourceName)) {
            assertWithMessage("Missing resource: %s", resourceName).that(stream).isNotNull();
            return CharStreams.toString(new InputStreamReader(stream, UTF_8));
        }
    }
    @Test
  public void testTrailingCommentWithoutTerminalNewline() throws Exception {
        assertThat(new Formatter().formatSource("/*\n * my comment */")).isEqualTo("/*\n * my comment */\n");
    }
    @Test
  public void testEmptyArray() throws Exception {
        assertThat(new Formatter().formatSource("class T { int x[] = {,}; }")).isEqualTo("""
            class T {
              int x[] = {,};
            }
            """);
    }
    @Test
  public void stringEscapeLength() throws Exception {
        assertThat(new Formatter().formatSource("class T {{ f(\"\\\"\"); }}")).isEqualTo("""
            class T {
              {
                f(\"\\\"\");
              }
            }
            """);
    }
    @Test
  public void wrapLineComment() throws Exception {
        assertThat(new Formatter().formatSource("""
class T {
  public static void main(String[] args) { // one long incredibly unbroken sentence moving from topic to topic so that no-one had a chance to interrupt;
  }
}
""")).isEqualTo("""
class T {
  public static void main(
      String[]
          args) { // one long incredibly unbroken sentence moving from topic to topic so that no-one
                  // had a chance to interrupt;
  }
}
""");
    }
    @Test
  public void onlyWrapLineCommentOnWhitespace() throws Exception {
        assertThat(new Formatter().formatSource("""
class T {
  public static void main(String[] args) { // one_long_incredibly_unbroken_sentence_moving_from_topic_to_topic_so_that_no-one_had_a_chance_to_interrupt;
  }
}
""")).isEqualTo("""
class T {
  public static void main(
      String[]
          args) { // one_long_incredibly_unbroken_sentence_moving_from_topic_to_topic_so_that_no-one_had_a_chance_to_interrupt;
  }
}
""");
    }
    @Test
  public void onlyWrapLineCommentOnWhitespace_noLeadingWhitespace() throws Exception {
        assertThat(new Formatter().formatSource("""
class T {
  public static void main(String[] args) { //one_long_incredibly_unbroken_sentence_moving_from_topic_to_topic_so_that_no-one_had_a_chance_to_interrupt;
  }
}
""")).isEqualTo("""
class T {
  public static void main(
      String[]
          args) { // one_long_incredibly_unbroken_sentence_moving_from_topic_to_topic_so_that_no-one_had_a_chance_to_interrupt;
  }
}
""");
    }
    @Test
  public void throwsFormatterException() throws Exception {
        assertThrows(FormatterException.class, () -> new Formatter().formatSourceAndFixImports("package foo; public class {"));
    }
    @Test
  public void blankLinesImportComment() throws FormatterException {
        String withBlank = """
        package p;

        /** test */

        import a.A;

        class T {
          A a;
        }
        """;
        String withoutBlank = """
        package p;

        /** test */
        import a.A;

        class T {
          A a;
        }
        """;
        assertThat(new Formatter().formatSource(withBlank)).isEqualTo(withoutBlank);
        assertThat(new Formatter().formatSourceAndFixImports(withBlank)).isEqualTo(withoutBlank);
        assertThat(new Formatter().formatSource(withoutBlank)).isEqualTo(withoutBlank);
        assertThat(new Formatter().formatSourceAndFixImports(withoutBlank)).isEqualTo(withoutBlank);
        assertThat(RemoveUnusedImports.removeUnusedImports(withBlank)).isEqualTo(withBlank);
        assertThat(ImportOrderer.reorderImports(withBlank, Style.GOOGLE)).isEqualTo(withBlank);
        assertThat(RemoveUnusedImports.removeUnusedImports(withoutBlank)).isEqualTo(withoutBlank);
        assertThat(ImportOrderer.reorderImports(withoutBlank, Style.GOOGLE)).isEqualTo(withoutBlank);
    }
    @Test
  public void dontWrapMoeLineComments() throws Exception {
        assertThat(new Formatter().formatSource("""
class T {
  // MOE: one long incredibly unbroken sentence moving from topic to topic so that no-one had a chance to interrupt;
}
""")).isEqualTo("""
class T {
  // MOE: one long incredibly unbroken sentence moving from topic to topic so that no-one had a chance to interrupt;
}
""");
    }
    @Test
  public void removeTrailingTabsInComments() throws Exception {
        assertThat(new Formatter().formatSource("class Foo {\n  void f() {\n    int x = 0; // comment\t\t\t\n    return;\n  }\n}\n")).isEqualTo("""
            class Foo {
              void f() {
                int x = 0; // comment
                return;
              }
            }
            """);
    }
    @Test
  public void testI1205() throws Exception {
        String input = """
        public interface Foo {

          private static String foo =
              \"\"\"
               foo\\
               bar \"\"\";
        }
        """;
        assertThat(new Formatter().formatSource(input)).isEqualTo(input);
    }
    @Test
  public void maxLineLength() throws Exception {
        assertThat(new Formatter(JavaFormatterOptions.builder().style(Style.GOOGLE.toBuilder().maxLineLength(40).build()).build()).formatSource("""
        class T {
          /** A javadoc comment that is longer than forty columns. */
          void f(int aaaaaaaaaa, int bbbbbbbbbb, int cccccccccc) {
            // A line comment that is longer than forty columns.
            int x = aaaaaaaaaa + bbbbbbbbbb + cccccccccc;
          }
        }
        """)).isEqualTo("""
            class T {
              /**
               * A javadoc comment that is longer
               * than forty columns.
               */
              void f(
                  int aaaaaaaaaa,
                  int bbbbbbbbbb,
                  int cccccccccc) {
                // A line comment that is longer
                // than forty columns.
                int x =
                    aaaaaaaaaa
                        + bbbbbbbbbb
                        + cccccccccc;
              }
            }
            """);
    }
    @Test
  public void useTabsGoogleStyle() throws Exception {
        assertThat(new Formatter(JavaFormatterOptions.builder().style(Style.GOOGLE.toBuilder().useTabs(true).maxLineLength(30).build()).build()).formatSource("""
        class T {
          /**
           * Multi-line javadoc
           * comment.
           */
          void f(int a, int b) {
            // multi-line
            // comment
            int x = aaaaaaaaaa + bbbbbbbbbb + cccccccccc;
          }
        }
        """)).isEqualTo("""
            class T {
            \t/**
            \t * Multi-line javadoc
            \t * comment.
            \t */
            \tvoid f(int a, int b) {
            \t\t// multi-line
            \t\t// comment
            \t\tint x =
            \t\t\t\taaaaaaaaaa
            \t\t\t\t\t\t+ bbbbbbbbbb
            \t\t\t\t\t\t+ cccccccccc;
            \t}
            }
            """);
    }
    @Test
  public void useTabsAospStyle() throws Exception {
        assertThat(new Formatter(JavaFormatterOptions.builder().style(Style.AOSP.toBuilder().useTabs(true).maxLineLength(30).build()).build()).formatSource("""
        class T {
          /**
           * Multi-line javadoc
           * comment.
           */
          void f(int a, int b) {
            int x = aaaaaaaaaa + bbbbbbbbbb + cccccccccc;
          }
        }
        """)).isEqualTo("""
            class T {
            \t/**
            \t * Multi-line javadoc
            \t * comment.
            \t */
            \tvoid f(int a, int b) {
            \t\tint x =
            \t\t\t\taaaaaaaaaa
            \t\t\t\t\t\t+ bbbbbbbbbb
            \t\t\t\t\t\t+ cccccccccc;
            \t}
            }
            """);
    }
    @Test
  public void styleVisualLengthAndIndentString() {
        Style google = Style.GOOGLE;
        assertThat(google.tabWidth()).isEqualTo(2);
        assertThat(google.visualLength("abcd")).isEqualTo(4);
        assertThat(google.visualLength("\tab")).isEqualTo(3);
        Style googleTabs = Style.GOOGLE.toBuilder().useTabs(true).build();
        assertThat(googleTabs.tabWidth()).isEqualTo(2);
        assertThat(googleTabs.indentString(5)).isEqualTo("\t\t ");
        assertThat(googleTabs.visualLength(googleTabs.indentString(5))).isEqualTo(5);
        assertThat(googleTabs.visualLength("\t")).isEqualTo(2);
        assertThat(googleTabs.visualLength("\t\t")).isEqualTo(4);
        assertThat(googleTabs.visualLength(" \t")).isEqualTo(2);
        assertThat(googleTabs.visualLength("\tab")).isEqualTo(4);
        Style aospTabs = Style.AOSP.toBuilder().useTabs(true).build();
        assertThat(aospTabs.tabWidth()).isEqualTo(4);
        assertThat(aospTabs.indentString(10)).isEqualTo("\t\t  ");
        assertThat(aospTabs.visualLength(aospTabs.indentString(10))).isEqualTo(10);
        assertThat(aospTabs.visualLength("\t")).isEqualTo(4);
        assertThat(aospTabs.visualLength("  \t")).isEqualTo(4);
        assertThat(google.indentString(0)).isEmpty();
        assertThat(googleTabs.indentString(0)).isEmpty();
        assertThat(aospTabs.indentString(0)).isEmpty();
        assertThat(google.indentString(4)).isSameInstanceAs(google.indentString(4));
        assertThat(google.indentString(104)).isEqualTo(" ".repeat(104));
        assertThat(googleTabs.indentString(104)).isEqualTo("\t".repeat(52));
        assertThat(JavaOutput.spaces(4)).isSameInstanceAs(JavaOutput.spaces(4));
        assertThat(JavaOutput.spaces(0)).isEmpty();
        assertThat(JavaOutput.spaces(104)).isEqualTo(" ".repeat(104));
    }
    @Test
  public void maxLineLengthExactBoundary() throws Exception {
        assertThat(new Formatter(JavaFormatterOptions.builder().style(Style.GOOGLE.toBuilder().maxLineLength(20).build()).build()).formatSource("class T extends S {}\n")).isEqualTo("class T extends S {}\n");
    }
    @Test
  public void maxLineLengthOutOfRange() {
        assertThat(assertThrows(IllegalArgumentException.class, () -> Style.builder().maxLineLength(0).build())).hasMessageThat().contains("maxLineLength must be between 1 and 999, was: 0");
        assertThat(assertThrows(IllegalArgumentException.class, () -> Style.builder().maxLineLength(-1).build())).hasMessageThat().contains("maxLineLength must be between 1 and 999, was: -1");
        assertThat(assertThrows(IllegalArgumentException.class, () -> Style.builder().maxLineLength(1000).build())).hasMessageThat().contains("maxLineLength must be between 1 and 999, was: 1000");
        assertThat(Style.builder().maxLineLength(999).build().maxLineLength()).isEqualTo(999);
    }
}
