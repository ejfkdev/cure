package com.google.googlejavaformat.java;

import static com.google.common.truth.Truth.assertThat;
import static org.junit.Assume.assumeTrue;
import com.google.googlejavaformat.java.JavaFormatterOptions.Style;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public class StringWrapperTest {
    @Test
  public void testAwkwardLineEndWrapping() throws Exception {
        assertThat(StringWrapper.wrap(100, """
class T {
  String s = someMethodWithQuiteALongNameThatWillGetUsUpCloseToTheColumnLimit() + "foo bar foo bar foo bar";

  String someMethodWithQuiteALongNameThatWillGetUsUpCloseToTheColumnLimit() {
    return null;
  }
}
""", new Formatter())).isEqualTo("""
        class T {
          String s =
              someMethodWithQuiteALongNameThatWillGetUsUpCloseToTheColumnLimit()
                  + "foo bar foo bar foo bar";

          String someMethodWithQuiteALongNameThatWillGetUsUpCloseToTheColumnLimit() {
            return null;
          }
        }
        """);
    }
    @Test
  public void textBlock() throws Exception {
        assumeTrue(Runtime.version().feature() >= 15);
        String input = """
package com.mypackage;
public class ReproBug {
  private String myString;
  private ReproBug() {
    String str =
\"""
{"sourceEndpoint":"ri.something.1-1.object-internal.1","targetEndpoint":"ri.something.1-1.object-internal.2","typeId":"typeId"}\\
\""";
    myString = str;
  }
}
""";
        assertThat(StringWrapper.wrap(100, input, new Formatter())).isEqualTo(input);
    }
    @Test
  public void textBlockControlCharacter() throws Exception {
        assumeTrue(Runtime.version().feature() >= 15);
        String input = """
        package p;
        public class T {
          String s =
              \"""
              lorem
              
              ipsum
              \""";
        }
        """;
        assertThat(StringWrapper.wrap(100, input, new Formatter())).isEqualTo(input);
    }
    @Test
  public void textBlockTrailingWhitespace() throws Exception {
        assumeTrue(Runtime.version().feature() >= 15);
        @SuppressWarnings("MisleadingEscapedSpace") // TODO(b/496180372): remove
            String input =
                """
                public class T {
                  String s =
                      \"""
                      lorem   \s
                      ipsum
                      \""";
                }
                """;
        assertThat(StringWrapper.wrap(100, input, new Formatter())).isEqualTo("""
        public class T {
          String s =
              \"""
              lorem
              ipsum
              \""";
        }
        """);
    }
    @Test
  public void textBlockTrailingWhitespaceUnicodeEscape() throws Exception {
        assumeTrue(Runtime.version().feature() >= 15);
        assertThat(StringWrapper.wrap(100, """
        public class T {
          String s =
              \"""
              lorem\\u0020
              ipsum
              \""";
        }
        """, new Formatter())).isEqualTo("""
        public class T {
          String s =
              \"""
              lorem\\u0020
              ipsum
              \""";
        }
        """);
    }
    @Test
  public void textBlockSpaceTabMix() throws Exception {
        assumeTrue(Runtime.version().feature() >= 15);
        assertThat(StringWrapper.wrap(100, """
        public class T {
          String s =
              \"""
              lorem
             \tipsum
              \""";
        }
        """, new Formatter())).isEqualTo("""
        public class T {
          String s =
              \"""
              lorem
              ipsum
              \""";
        }
        """);
    }
    @Test
  public void leadingBlankLine() throws Exception {
        assumeTrue(Runtime.version().feature() >= 15);
        assertThat(StringWrapper.wrap(100, """
        public class T {
          String s =
              \"""

              lorem
              ipsum
              \""";
        }
        """, new Formatter())).isEqualTo("""
        public class T {
          String s =
              \"""

              lorem
              ipsum
              \""";
        }
        """);
    }
    @Test
  public void wrapWithMaxLineLengthAndTabs() throws Exception {
        assertThat(new Formatter(JavaFormatterOptions.builder().style(Style.GOOGLE.toBuilder().maxLineLength(35).useTabs(true).build()).build()).formatSourceAndFixImports("""
        class T {
          String s = "one two three four five six seven eight";
          String tb =
              \"""
              hello
              world
              \""";
        }
        """)).isEqualTo("""
            class T {
            \tString s =
            \t\t\t"one two three four five six"
            \t\t\t\t\t+ " seven eight";
            \tString tb =
            \t\t\t\"""
            \t\t\thello
            \t\t\tworld
            \t\t\t\""";
            }
            """);
    }
}
