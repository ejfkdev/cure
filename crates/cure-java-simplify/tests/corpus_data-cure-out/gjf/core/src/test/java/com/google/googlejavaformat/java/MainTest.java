package com.google.googlejavaformat.java;

import static com.google.common.base.StandardSystemProperty.JAVA_CLASS_PATH;
import static com.google.common.base.StandardSystemProperty.JAVA_HOME;
import static com.google.common.truth.Truth.assertThat;
import static com.google.common.truth.Truth.assertWithMessage;
import static java.nio.charset.StandardCharsets.UTF_8;
import com.google.common.collect.ImmutableList;
import com.google.common.io.ByteStreams;
import java.io.BufferedWriter;
import java.io.ByteArrayInputStream;
import java.io.File;
import java.io.InputStream;
import java.io.OutputStreamWriter;
import java.io.PrintWriter;
import java.io.StringWriter;
import java.lang.ProcessBuilder.Redirect;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.attribute.PosixFilePermission;
import java.util.EnumSet;
import java.util.Locale;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public class MainTest {
    @Rule public TemporaryFolder testFolder = new TemporaryFolder();
    private static final ImmutableList<String> ADD_EXPORTS = ImmutableList.of("--add-exports=jdk.compiler/com.sun.tools.javac.api=ALL-UNNAMED", "--add-exports=jdk.compiler/com.sun.tools.javac.code=ALL-UNNAMED", "--add-exports=jdk.compiler/com.sun.tools.javac.file=ALL-UNNAMED", "--add-exports=jdk.compiler/com.sun.tools.javac.parser=ALL-UNNAMED", "--add-exports=jdk.compiler/com.sun.tools.javac.tree=ALL-UNNAMED", "--add-exports=jdk.compiler/com.sun.tools.javac.util=ALL-UNNAMED");
    @Test
  public void testUsageOutput() {
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        Main main = new Main(new PrintWriter(out, true), new PrintWriter(err, true), System.in);
        try {
            main.format("--help");
            throw new AssertionError("Expected UsageException to be thrown");
        } catch (UsageException e) {
            String usage = e.getMessage();
            assertThat(usage).contains("https://github.com/google/google-java-format");
            assertThat(usage).contains("Usage: google-java-format");
            assertThat(usage).contains("--length");
            assertThat(usage).contains("Character length to format.");
            assertThat(usage).contains("the result is sent to stdout");
        }
    }
    @Test
  public void version() throws UsageException {
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(err, true), System.in).format("-version")).isEqualTo(0);
        assertThat(err.toString()).contains("google-java-format: Version ");
    }
    @Test
  public void preserveOriginalFile() throws Exception {
        Path path = testFolder.newFile("Test.java").toPath();
        Files.write(path, "class Test {}\n".getBytes(UTF_8));
        try {
            Files.setPosixFilePermissions(path, EnumSet.of(PosixFilePermission.OWNER_READ));
        } catch (UnsupportedOperationException e) {
            return;
        }
        int errorCode = new Main(new PrintWriter(new BufferedWriter(new OutputStreamWriter(System.out, UTF_8)), true), new PrintWriter(new BufferedWriter(new OutputStreamWriter(System.err, UTF_8)), true), System.in).format("-replace", path.toAbsolutePath().toString());
        assertWithMessage("Error Code").that(errorCode).isEqualTo(0);
    }
    @Test
  public void testMain() throws Exception {
        Process process = new ProcessBuilder(ImmutableList.builder().add(Paths.get(JAVA_HOME.value()).resolve("bin/java").toString()).addAll(ADD_EXPORTS).add("-cp").add(JAVA_CLASS_PATH.value()).add(Main.class.getName()).build()).redirectError(Redirect.PIPE).redirectOutput(Redirect.PIPE).start();
        process.waitFor();
        assertThat(new String(ByteStreams.toByteArray(process.getErrorStream()), UTF_8)).contains("Usage: google-java-format");
        assertThat(process.exitValue()).isEqualTo(2);
    }
    @Test
  public void javadoc() throws Exception {
        InputStream in = new ByteArrayInputStream("""
/**
 * graph
 *
 * graph
 *
 * @param foo lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua
 */
class Test {
  /**
   * creates entropy
   */
  public static void main(String... args) {}
}\
""".getBytes(UTF_8));
        StringWriter out = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(new BufferedWriter(new OutputStreamWriter(System.err, UTF_8)), true), in).format("-")).isEqualTo(0);
        assertThat(out.toString()).isEqualTo("""
/**
 * graph
 *
 * <p>graph
 *
 * @param foo lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor
 *     incididunt ut labore et dolore magna aliqua
 */
class Test {
  /** creates entropy */
  public static void main(String... args) {}
}
""");
    }
    @Test
  public void imports() throws Exception {
        InputStream in = new ByteArrayInputStream("""
        import java.util.LinkedList;
        import java.util.List;
        import java.util.ArrayList;
        class Test {
          /**
           * May be an {@link ArrayList}.
           */
          public static List<String> names;
        }\
        """.getBytes(UTF_8));
        StringWriter out = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(new BufferedWriter(new OutputStreamWriter(System.err, UTF_8)), true), in).format("-", "--fix-imports-only")).isEqualTo(0);
        assertThat(out.toString()).isEqualTo("""
        import java.util.ArrayList;
        import java.util.List;

        class Test {
          /**
           * May be an {@link ArrayList}.
           */
          public static List<String> names;
        }\
        """);
    }
    @Test
  public void optimizeImportsDoesNotLeaveEmptyLines() throws Exception {
        @SuppressWarnings("MisleadingEscapedSpace") // TODO(b/496180372): remove
            String input =
                """
                package abc;

                import java.util.LinkedList;
                import java.util.List;
                import java.util.ArrayList;

                import static java.nio.charset.StandardCharsets.UTF_8;

                import java.util.EnumSet;

                class Test\s
                extends ArrayList {
                }\
                """;
        String expected = """
        package abc;

        import java.util.ArrayList;

        class Test extends ArrayList {}
        """;
        assertThat(new Formatter().formatSourceAndFixImports(input)).isEqualTo(expected);
        InputStream in = new ByteArrayInputStream(input.getBytes(UTF_8));
        StringWriter out = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(new BufferedWriter(new OutputStreamWriter(System.err, UTF_8)), true), in).format("-")).isEqualTo(0);
        assertThat(out.toString()).isEqualTo(expected);
    }
    @Test
  public void importRemovalLines() throws Exception {
        StringWriter out = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(new BufferedWriter(new OutputStreamWriter(System.err, UTF_8)), true), new ByteArrayInputStream("""
        import java.util.ArrayList;
        import java.util.List;
        class Test {
        ArrayList<String> a = new ArrayList<>();
        ArrayList<String> b = new ArrayList<>();
        }\
        """.getBytes(UTF_8))).format("-", "-lines", "4")).isEqualTo(0);
        assertThat(out.toString()).isEqualTo("""
        import java.util.ArrayList;

        class Test {
          ArrayList<String> a = new ArrayList<>();
        ArrayList<String> b = new ArrayList<>();
        }\
        """);
    }
    @Test
  public void importRemoveErrorParseError() throws Exception {
        Locale backupLocale = Locale.getDefault();
        try {
            Locale.setDefault(Locale.ROOT);
            StringWriter out = new StringWriter();
            StringWriter err = new StringWriter();
            assertThat(new Main(new PrintWriter(out, true), new PrintWriter(err, true), new ByteArrayInputStream("""
          import java.util.ArrayList;
          import java.util.List;
          class Test {
          }}\
          """.getBytes(UTF_8))).format("-")).isEqualTo(1);
            assertThat(err.toString()).contains("<stdin>:4:2: error: class, interface");
        } finally {
            Locale.setDefault(backupLocale);
        }
    }
    @Test
  public void packageInfo() throws Exception {
        String input = """
        @CheckReturnValue
        @ParametersAreNonnullByDefault
        package com.google.common.labs.base;

        import com.google.errorprone.annotations.CheckReturnValue;
        import javax.annotation.ParametersAreNonnullByDefault;
        """;
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(err, true), new ByteArrayInputStream(input.getBytes(UTF_8))).format("-")).isEqualTo(0);
        assertThat(out.toString()).isEqualTo(input);
    }
    @Test
  public void newline() throws Exception {
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(err, true), new ByteArrayInputStream("class T {}\n\t".getBytes(UTF_8))).format("-")).isEqualTo(0);
        assertThat(out.toString()).isEqualTo("class T {}\n");
    }
    @Test
  public void dryRunStdinUnchanged() throws Exception {
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(err, true), new ByteArrayInputStream("class Test {}\n".getBytes(UTF_8))).format("-n", "-")).isEqualTo(0);
        assertThat(out.toString()).isEmpty();
        assertThat(err.toString()).isEmpty();
    }
    @Test
  public void dryRunStdinChanged() throws Exception {
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(err, true), new ByteArrayInputStream("class Test {\n}\n".getBytes(UTF_8))).format("-n", "-")).isEqualTo(0);
        assertThat(out.toString()).isEqualTo("<stdin>" + System.lineSeparator());
        assertThat(err.toString()).isEmpty();
    }
    @Test
  public void dryRunFiles() throws Exception {
        Path a = testFolder.newFile("A.java").toPath();
        Path b = testFolder.newFile("B.java").toPath();
        Path c = testFolder.newFile("C.java").toPath();
        Files.write(a, "class A {}\n".getBytes(UTF_8));
        Files.write(b, "class B {\n}\n".getBytes(UTF_8));
        Files.write(c, "class C {\n}\n".getBytes(UTF_8));
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(err, true), System.in).format("-n", a.toAbsolutePath().toAbsolutePath().toString(), b.toAbsolutePath().toString(), c.toAbsolutePath().toString())).isEqualTo(0);
        assertThat(out.toString()).isEqualTo(b.toAbsolutePath() + System.lineSeparator() + c.toAbsolutePath() + System.lineSeparator());
        assertThat(err.toString()).isEmpty();
    }
    @Test
  public void keepGoingWhenFilesDontExist() throws Exception {
        Path a = testFolder.newFile("A.java").toPath();
        Path b = testFolder.newFile("B.java").toPath();
        File cFile = testFolder.newFile("C.java");
        Path c = cFile.toPath();
        cFile.delete();
        Files.write(a, "class A{}\n".getBytes(UTF_8));
        Files.write(b, "class B{}\n".getBytes(UTF_8));
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(err, true), System.in).format("", a.toAbsolutePath().toString(), c.toAbsolutePath().toString(), b.toAbsolutePath().toString())).isEqualTo(1);
        assertThat(out.toString()).isEqualTo("class A {}\nclass B {}\n");
        assertThat(err.toString()).isNotEmpty();
    }
    @Test
  public void exitIfChangedStdin() throws Exception {
        Path path = testFolder.newFile("Test.java").toPath();
        Files.write(path, "class Test {\n}\n".getBytes(UTF_8));
        Process process = new ProcessBuilder(ImmutableList.builder().add(Paths.get(JAVA_HOME.value()).resolve("bin/java").toString()).addAll(ADD_EXPORTS).add("-cp").add(JAVA_CLASS_PATH.value()).add(Main.class.getName()).add("-n").add("--set-exit-if-changed").add("-").build()).redirectInput(path.toFile()).redirectError(Redirect.PIPE).redirectOutput(Redirect.PIPE).start();
        process.waitFor();
        String err = new String(ByteStreams.toByteArray(process.getErrorStream()), UTF_8);
        String out = new String(ByteStreams.toByteArray(process.getInputStream()), UTF_8);
        assertThat(err).isEmpty();
        assertThat(out).isEqualTo("<stdin>" + System.lineSeparator());
        assertThat(process.exitValue()).isEqualTo(1);
    }
    @Test
  public void exitIfChangedFiles() throws Exception {
        Path path = testFolder.newFile("Test.java").toPath();
        Files.write(path, "class Test {\n}\n".getBytes(UTF_8));
        Process process = new ProcessBuilder(ImmutableList.builder().add(Paths.get(JAVA_HOME.value()).resolve("bin/java").toString()).addAll(ADD_EXPORTS).add("-cp").add(JAVA_CLASS_PATH.value()).add(Main.class.getName()).add("-n").add("--set-exit-if-changed").add(path.toAbsolutePath().toString()).build()).redirectError(Redirect.PIPE).redirectOutput(Redirect.PIPE).start();
        process.waitFor();
        String err = new String(ByteStreams.toByteArray(process.getErrorStream()), UTF_8);
        String out = new String(ByteStreams.toByteArray(process.getInputStream()), UTF_8);
        assertThat(err).isEmpty();
        assertThat(out).isEqualTo(path.toAbsolutePath() + System.lineSeparator());
        assertThat(process.exitValue()).isEqualTo(1);
    }
    @Test
  public void assumeFilename_error() throws Exception {
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(err, true), new ByteArrayInputStream("""
        class Test {}}\
        """.getBytes(UTF_8))).format("--assume-filename=Foo.java", "-")).isEqualTo(1);
        assertThat(err.toString()).contains("Foo.java:1:14: error: class, interface");
    }
    @Test
  public void assumeFilename_dryRun() throws Exception {
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(err, true), new ByteArrayInputStream("""
        class Test {
        }\
        """.getBytes(UTF_8))).format("--dry-run", "--assume-filename=Foo.java", "-")).isEqualTo(0);
        assertThat(out.toString()).isEqualTo("Foo.java" + System.lineSeparator());
    }
    @Test
  public void reflowLongStrings() throws Exception {
        InputStream in = new ByteArrayInputStream("""
        class T {
          String s = "one long incredibly unbroken sentence moving from topic to topic so that no one had a chance to interrupt";
        }\
        """.getBytes(UTF_8));
        StringWriter out = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(new BufferedWriter(new OutputStreamWriter(System.err, UTF_8)), true), in).format("-")).isEqualTo(0);
        assertThat(out.toString()).isEqualTo("""
        class T {
          String s =
              "one long incredibly unbroken sentence moving from topic to topic so that no one had a chance"
                  + " to interrupt";
        }
        """);
    }
    @Test
  public void noReflowLongStrings() throws Exception {
        InputStream in = new ByteArrayInputStream("""
class T {
  String s = "one long incredibly unbroken sentence moving from topic to topic so that no one had a chance to interrupt";
}\
""".getBytes(UTF_8));
        StringWriter out = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(new BufferedWriter(new OutputStreamWriter(System.err, UTF_8)), true), in).format("--skip-reflowing-long-strings", "-")).isEqualTo(0);
        assertThat(out.toString()).isEqualTo("""
class T {
  String s =
      "one long incredibly unbroken sentence moving from topic to topic so that no one had a chance to interrupt";
}
""");
    }
    @Test
  public void noFormatJavadoc() throws Exception {
        String input = """
        /**
         * graph
         *
         * graph
         *
         * @param foo lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor
         *     incididunt ut labore et dolore magna aliqua
         */
        class Test {
          /**
           * creates entropy
           */
          public static void main(String... args) {}
        }
        """;
        InputStream in = new ByteArrayInputStream(input.getBytes(UTF_8));
        StringWriter out = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(new BufferedWriter(new OutputStreamWriter(System.err, UTF_8)), true), in).format("--skip-javadoc-formatting", "-")).isEqualTo(0);
        assertThat(out.toString()).isEqualTo(input);
    }
    @Test
  public void reorderModifiersOptionTest() throws Exception {
        String input = """
        class Test {
          static public void main(String... args) {}
        }
        """;
        assertThat(new Formatter(JavaFormatterOptions.builder().build()).formatSource(input)).isEqualTo("""
        class Test {
          public static void main(String... args) {}
        }
        """);
        assertThat(new Formatter(JavaFormatterOptions.builder().reorderModifiers(false).build()).formatSource(input)).isEqualTo(input);
    }
    @Test
  public void noReorderModifiers() throws Exception {
        String input = """
        class Test {
          static public void main(String... args) {}
        }
        """;
        InputStream in = new ByteArrayInputStream(input.getBytes(UTF_8));
        StringWriter out = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(new BufferedWriter(new OutputStreamWriter(System.err, UTF_8)), true), in).format("--skip-reordering-modifiers", "-")).isEqualTo(0);
        assertThat(out.toString()).isEqualTo(input);
    }
    @Test
  public void syntaxError() throws Exception {
        Path path = testFolder.newFile("Test.java").toPath();
        Files.writeString(path, """
        class Test {
          void f(int package) {
            int
          }
        }\
        """, UTF_8);
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        int errorCode = new Main(new PrintWriter(out, true), new PrintWriter(err, true), System.in).format(path.toAbsolutePath().toString());
        assertWithMessage("Error Code").that(errorCode).isEqualTo(1);
        String expected = """
        «path»:2:13: error: <identifier> expected
          void f(int package) {
                    ^
        «path»:3:5: error: not a statement
            int
            ^
        «path»:3:8: error: ';' expected
            int
               ^
        """.replace("«path»", path.toString()).replace("\n", System.lineSeparator());
        assertThat(err.toString()).isEqualTo(expected);
    }
    @Test
  public void syntaxErrorBeginning() throws Exception {
        Path path = testFolder.newFile("Test.java").toPath();
        Files.writeString(path, "error", UTF_8);
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        int errorCode = new Main(new PrintWriter(out, true), new PrintWriter(err, true), System.in).format(path.toAbsolutePath().toString());
        assertWithMessage("Error Code").that(errorCode).isEqualTo(1);
        String expected = """
        «path»:1:1: error: reached end of file while parsing
        error
        ^
        """.replace("«path»", path.toString()).replace("\n", System.lineSeparator());
        assertThat(err.toString()).isEqualTo(expected);
    }
    @Test
  public void maxLineLength() throws Exception {
        InputStream in = new ByteArrayInputStream("""
        class Test {
          void f() {
            int x = aaaaaaaaaa + bbbbbbbbbb + cccccccccc;
          }
        }
        """.getBytes(UTF_8));
        StringWriter out = new StringWriter();
        assertThat(new Main(new PrintWriter(out, true), new PrintWriter(new BufferedWriter(new OutputStreamWriter(System.err, UTF_8)), true), in).format("--max-line-length=30", "-")).isEqualTo(0);
        assertThat(out.toString()).isEqualTo("""
        class Test {
          void f() {
            int x =
                aaaaaaaaaa
                    + bbbbbbbbbb
                    + cccccccccc;
          }
        }
        """);
    }
}
