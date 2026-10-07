package com.google.googlejavaformat.java;

import static com.google.common.truth.Truth.assertThat;
import static java.nio.charset.StandardCharsets.UTF_8;
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.PrintWriter;
import java.io.StringWriter;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Locale;
import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public class DiagnosticTest {
    @Rule public TemporaryFolder testFolder = new TemporaryFolder();
    private Locale backupLocale;
    @Before
  public void setUpLocale() throws Exception {
        backupLocale = Locale.getDefault();
        Locale.setDefault(Locale.ROOT);
    }
    @After
  public void restoreLocale() throws Exception {
        Locale.setDefault(backupLocale);
    }
    @Test
  public void parseError() throws Exception {
        StringWriter stdout = new StringWriter();
        StringWriter stderr = new StringWriter();
        Main main = new Main(new PrintWriter(stdout, true), new PrintWriter(stderr, true), System.in);
        Path path = testFolder.newFolder().toPath().resolve("InvalidSyntax.java");
        Files.write(path, """
        public class InvalidSyntax {
          private static NumPrinter {
            public static void print(int n) {
              System.out.printf("%d%n", n);
            }
          }

          public static void main(String[] args) {
            NumPrinter.print(args.length);
          }
        }\
        """.getBytes(UTF_8));
        int result = main.format(path.toString());
        assertThat(stdout.toString()).isEmpty();
        assertThat(stderr.toString()).contains("InvalidSyntax.java:2:28: error: <identifier> expected");
        assertThat(result).isEqualTo(1);
    }
    @Test
  public void lexError() throws Exception {
        StringWriter stdout = new StringWriter();
        StringWriter stderr = new StringWriter();
        Main main = new Main(new PrintWriter(stdout, true), new PrintWriter(stderr, true), System.in);
        Path path = testFolder.newFolder().toPath().resolve("InvalidSyntax.java");
        Files.write(path, "\\uuuuuuuuuuuuuuuuuuuuuuuuuuuuuu00not-actually-a-unicode-escape-sequence".getBytes(UTF_8));
        int result = main.format(path.toString());
        assertThat(stdout.toString()).isEmpty();
        assertThat(stderr.toString()).contains("error: illegal unicode escape");
        assertThat(result).isEqualTo(1);
    }
    @Test
  public void oneFileParseError() throws Exception {
        String two = "class Two {}\n";
        StringWriter stdout = new StringWriter();
        StringWriter stderr = new StringWriter();
        Main main = new Main(new PrintWriter(stdout, true), new PrintWriter(stderr, true), System.in);
        Path tmpdir = testFolder.newFolder().toPath();
        Path pathOne = tmpdir.resolve("One.java");
        Files.write(pathOne, "class One {\n".getBytes(UTF_8));
        Path pathTwo = tmpdir.resolve("Two.java");
        Files.write(pathTwo, two.getBytes(UTF_8));
        int result = main.format(pathOne.toString(), pathTwo.toString());
        assertThat(stdout.toString()).isEqualTo(two);
        assertThat(stderr.toString()).contains("One.java:1:12: error: reached end of file");
        assertThat(result).isEqualTo(1);
    }
    @Test
  public void oneFileParseErrorReplace() throws Exception {
        StringWriter stdout = new StringWriter();
        StringWriter stderr = new StringWriter();
        Main main = new Main(new PrintWriter(stdout, true), new PrintWriter(stderr, true), System.in);
        Path tmpdir = testFolder.newFolder().toPath();
        Path pathOne = tmpdir.resolve("One.java");
        Files.write(pathOne, "class One {}}\n".getBytes(UTF_8));
        Path pathTwo = tmpdir.resolve("Two.java");
        Files.write(pathTwo, "class Two {\n}\n".getBytes(UTF_8));
        int result = main.format("-i", pathOne.toString(), pathTwo.toString());
        assertThat(stdout.toString()).isEmpty();
        assertThat(stderr.toString()).contains("One.java:1:13: error: class, interface");
        assertThat(result).isEqualTo(1);
        assertThat(Files.readAllLines(pathOne, UTF_8)).containsExactly("class One {}}");
        assertThat(Files.readAllLines(pathTwo, UTF_8)).containsExactly("class Two {}");
    }
    @Test
  public void parseError2() throws FormatterException, IOException, UsageException {
        Path path = testFolder.newFolder().toPath().resolve("A.java");
        Files.write(path, """
        class Foo { void f() {
         g() } }
        """.getBytes(UTF_8));
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        Main main = new Main(new PrintWriter(out, true), new PrintWriter(err, true), System.in);
        String[] args = {path.toString()};
        assertThat(main.format(args)).isEqualTo(1);
        assertThat(err.toString()).contains("A.java:2:5: error: ';' expected");
    }
    @Test
  public void parseErrorStdin() throws FormatterException, IOException, UsageException {
        InputStream inStream = new ByteArrayInputStream("""
        class Foo { void f() {
         g() } }
        """.getBytes(UTF_8));
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        Main main = new Main(new PrintWriter(out, true), new PrintWriter(err, true), inStream);
        String[] args = {"-"};
        assertThat(main.format(args)).isEqualTo(1);
        assertThat(err.toString()).contains("<stdin>:2:5: error: ';' expected");
    }
    @Test
  public void lexError2() throws FormatterException, IOException, UsageException {
        Path path = testFolder.newFolder().toPath().resolve("A.java");
        Files.write(path, """
        class Foo { void f() {
         g('foo'); } }
        """.getBytes(UTF_8));
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        Main main = new Main(new PrintWriter(out, true), new PrintWriter(err, true), System.in);
        String[] args = {path.toString()};
        assertThat(main.format(args)).isEqualTo(1);
        assertThat(err.toString()).contains("A.java:2:4: error: unclosed character literal");
    }
    @Test
  public void lexErrorStdin() throws FormatterException, IOException, UsageException {
        InputStream inStream = new ByteArrayInputStream("""
        class Foo { void f() {
         g('foo'); } }
        """.getBytes(UTF_8));
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        Main main = new Main(new PrintWriter(out, true), new PrintWriter(err, true), inStream);
        String[] args = {"-"};
        assertThat(main.format(args)).isEqualTo(1);
        assertThat(err.toString()).contains("<stdin>:2:4: error: unclosed character literal");
    }
}
