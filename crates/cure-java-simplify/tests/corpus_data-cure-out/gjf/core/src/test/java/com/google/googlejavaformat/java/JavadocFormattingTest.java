package com.google.googlejavaformat.java;

import static com.google.common.truth.Truth.assertThat;
import static com.google.common.truth.Truth.assertWithMessage;
import static com.google.common.truth.TruthJUnit.assume;
import static java.nio.charset.StandardCharsets.UTF_8;
import com.google.common.io.ByteStreams;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public final class JavadocFormattingTest {
    private static final boolean MARKDOWN_JAVADOC_SUPPORTED = Runtime.version().feature() >= 23;
    private final Formatter formatter = new Formatter();
    private void doFormatTest(String input, String expected) {
        try {
            String actual = formatter.formatSource(input);
            assertThat(actual).isEqualTo(expected);
            String reformatted = formatter.formatSource(actual);
            assertWithMessage("When checking idempotency").that(reformatted).isEqualTo(actual);
        } catch (FormatterException e) {
            throw new AssertionError(e);
        }
    }
    @Test
  public void notJavadoc() {
        doFormatTest("""
        /**/
        class Test {}\
        """, """
        /**/
        class Test {}
        """);
    }
    @Test
  public void empty() {
        doFormatTest("""
        /***/
        class Test {}\
        """, """
        /***/
        class Test {}
        """);
    }
    @Test
  public void emptyMultipleLines() {
        doFormatTest("""
        /**
         */
        class Test {}\
        """, """
        /** */
        class Test {}
        """);
    }
    @Test
  public void simple() {
        doFormatTest("""
        /** */
        class Test {}\
        """, """
        /** */
        class Test {}
        """);
    }
    @Test
  public void commentMostlyUntouched() {
        doFormatTest("""
        /**
         * Foo.
         *
         *  <!--
        *abc
         *   def   \s
         * </tr>
         *-->bar
         */
        class Test {}\
        """, """
        /**
         * Foo.
         * <!--
         * abc
         *   def
         * </tr>
         * -->
         * bar
         */
        class Test {}
        """);
    }
    @Test
  public void markdownHtmlComment() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        String input = """
        /// <!--
        /// abc
        /// -->
        class Test {}
        """;
        doFormatTest(input, input);
    }
    @Test
  public void moeComments() {
        doFormatTest("        /**\n         * Deatomizes the given user.\n         * <!-- MOE:begin_intracomment_strip -->\n         * See deatomizer-v5 for the design doc.\n         * <!-- MOE:end_intracomment_strip -->\n         * To reatomize, call {@link reatomize}.\n         *\n         * <!-- MOE:begin_intracomment_strip -->\n         * <p>This method is used in the Google teleporter.\n         *\n         * <p>Yes, we have a teleporter.\n         * <!-- MOE:end_intracomment_strip -->\n         *\n         * @param user the person to teleport.\n         *     <!-- MOE:begin_intracomment_strip -->\n         *     Users must sign deatomize-waiver ahead of time.\n         *     <!-- MOE:end_intracomment_strip -->\n         * <!-- MOE:begin_intracomment_strip -->\n         * @deprecated Sometimes turns the user into a goat.\n         * <!-- MOE:end_intracomment_strip -->\n         */\n        class Test {}\\\n        ", "        /**\n         * Deatomizes the given user.\n         * <!-- MOE:begin_intracomment_strip -->\n         * See deatomizer-v5 for the design doc.\n         * <!-- MOE:end_intracomment_strip -->\n         * To reatomize, call {@link reatomize}.\n         *\n         * <!-- MOE:begin_intracomment_strip -->\n         * <p>This method is used in the Google teleporter.\n         *\n         * <p>Yes, we have a teleporter.\n         * <!-- MOE:end_intracomment_strip -->\n         *\n         * @param user the person to teleport.\n         *     <!-- MOE:begin_intracomment_strip -->\n         *     Users must sign deatomize-waiver ahead of time.\n         *     <!-- MOE:end_intracomment_strip -->\n         * <!-- MOE:begin_intracomment_strip -->\n         * @deprecated Sometimes turns the user into a goat.\n         * <!-- MOE:end_intracomment_strip -->\n         */\n        class Test {}\n        ");
    }
    @Test
  public void moeCommentBeginOnlyInMiddleOfDoc() {
        doFormatTest("        /**\n         * Foo.\n         * <!-- MOE:begin_intracomment_strip -->\n         * Bar.\n         */\n        class Test {}\\\n        ", "        /**\n         * Foo.\n         * <!-- MOE:begin_intracomment_strip -->\n         * Bar.\n         */\n        class Test {}\n        ");
    }
    @Test
  public void moeCommentBeginOnlyAtEndOfDoc() {
        doFormatTest("        /**\n         * Foo.\n         * <!-- MOE:begin_intracomment_strip -->\n         */\n        class Test {}\\\n        ", """
        /** Foo. */
        class Test {}
        """);
    }
    @Test
  public void moeCommentEndOnly() {
        doFormatTest("        /**\n         * Foo.\n         * <!-- MOE:end_intracomment_strip -->\n         */\n        class Test {}\\\n        ", "        /**\n         * Foo.\n         * <!-- MOE:end_intracomment_strip -->\n         */\n        class Test {}\n        ");
    }
    @Test
  public void moeCommentAtStartOfDoc() {
        doFormatTest("        /**\n         * <!-- MOE:begin_intracomment_strip -->\n         * Foo.\n         * <!-- MOE:end_intracomment_strip -->\n         */\n        class Test {}\\\n        ", "        /**\n         * <!-- MOE:begin_intracomment_strip -->\n         * Foo.\n         * <!-- MOE:end_intracomment_strip -->\n         */\n        class Test {}\n        ");
    }
    @Test
  public void tableMostlyUntouched() {
        doFormatTest("""
        /**
         * Foo.
         *
         *  <table>
        *<tr><td>a<td>b</tr>
         * <tr>
         * <td>A
         *     <td>B
         * </tr>
         *</table>
         */
        class Test {}\
        """, """
        /**
         * Foo.
         *
         * <table>
         * <tr><td>a<td>b</tr>
         * <tr>
         * <td>A
         *     <td>B
         * </tr>
         * </table>
         */
        class Test {}
        """);
    }
    @Test
  public void preMostlyUntouched() {
        @SuppressWarnings("MisleadingEscapedSpace") // TODO(b/496180372): remove
            String input =
                """
                /**
                 * Example:
                 *
                 *  <pre>
                *    1 2<br>    3   \s
                *4 5 6
                7 8
                 *</pre>
                 */
                class Test {}\
                """;
        doFormatTest(input, """
        /**
         * Example:
         *
         * <pre>
         *    1 2<br>    3
         * 4 5 6
         * 7 8
         * </pre>
         */
        class Test {}
        """);
    }
    @Test
  public void preCodeExample() {
        doFormatTest("""
        /**
         * Example:
         *
         * <pre>   {@code
         *
         *   Abc.def(foo, 7, true); // blah}</pre>
         */
        class Test {}\
        """, """
        /**
         * Example:
         *
         * <pre>{@code
         * Abc.def(foo, 7, true); // blah
         * }</pre>
         */
        class Test {}
        """);
    }
    @Test
  public void preNotWrapped() {
        doFormatTest("""
        /**
         * Example:
         *
         * <pre>
         * 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 45678901
         * </pre>
         */
        class Test {}\
        """, """
        /**
         * Example:
         *
         * <pre>
         * 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 45678901
         * </pre>
         */
        class Test {}
        """);
    }
    @Test
  public void javaCodeInPre() {
        doFormatTest("""
        /**
         * Example:
         *
         *<pre>
         * aaaaa    |   a  |   +
         * "bbbb    |   b  |  "
         *</pre>
         */
        class Test {}\
        """, """
        /**
         * Example:
         *
         * <pre>
         * aaaaa    |   a  |   +
         * "bbbb    |   b  |  "
         * </pre>
         */
        class Test {}
        """);
    }
    @Test
  public void joinLines() {
        doFormatTest("""
        /**
         * foo
         * bar
         * baz
         */
        class Test {}\
        """, """
        /** foo bar baz */
        class Test {}
        """);
    }
    @Test
  public void oneLinerIs100() {
        doFormatTest("""
        /**
         * 567890123 567890123 567890123 567890123 567890123 567890123 567890123 567890123 567890123 567
         */
        class Test {}\
        """, """
        /** 567890123 567890123 567890123 567890123 567890123 567890123 567890123 567890123 567890123 567 */
        class Test {}
        """);
    }
    @Test
  public void oneLinerWouldBe101() {
        doFormatTest("""
        /**
         * 567890123 567890123 567890123 567890123 567890123 567890123 567890123 567890123 567890123 5678
         */
        class Test {}\
        """, """
        /**
         * 567890123 567890123 567890123 567890123 567890123 567890123 567890123 567890123 567890123 5678
         */
        class Test {}
        """);
    }
    @Test
  public void multilineWrap() {
        doFormatTest("""
        /**
         * 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 45678901
         */
        class Test {}\
        """, """
        /**
         * 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012
         * 45678901
         */
        class Test {}
        """);
    }
    @Test
  public void tooLong() {
        doFormatTest("""
        /**
         * abc
         *
         * <p>7890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456
         */
        class Test {}\
        """, """
        /**
         * abc
         *
         * <p>7890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456
         */
        class Test {}
        """);
    }
    @Test
  public void joinedTokens() {
        doFormatTest("""
        /**
         * 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 4<b>8901
         */
        class Test {}\
        """, """
        /**
         * 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012
         * 4<b>8901
         */
        class Test {}
        """);
    }
    @Test
  public void joinedAtSign() {
        doFormatTest("""
        /**
         * 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 @5678901
         */
        class Test {}\
        """, """
        /**
         * 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012
         * 456789012 @5678901
         */
        class Test {}
        """);
    }
    @Test
  public void joinedMultipleAtSign() {
        doFormatTest("""
        /**
         * 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 @56789012 @5678901
         */
        class Test {}\
        """, """
        /**
         * 456789012 456789012 456789012 456789012 456789012 456789012 456789012
         * 456789012 @56789012 @5678901
         */
        class Test {}
        """);
    }
    @Test
  public void noAsterisk() {
        doFormatTest("""
        /**
         abc<p>def
         */
        class Test {}\
        """, """
        /**
         * abc
         *
         * <p>def
         */
        class Test {}
        """);
    }
    @Test
  public void significantAsterisks() {
        doFormatTest("""
        /** *
         * *
         */
        class Test {}\
        """, """
        /** * * */
        class Test {}
        """);
    }
    @Test
  public void links() {
        doFormatTest("""
        /**
         * 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 4567 <a
         * href=foo>foo</a>.
         *
         * <p>789012 456789012 456789012 456789012 456789012 456789012 456789012 456789 <a href=foo>
         * foo</a>.
         *
         * <p>789012 456789012 456789012 456789012 456789012 456789012 456789012 4567890 <a href=foo>
         * foo</a>.
         *
         * <p><a href=foo>
         * foo</a>.
         *
         * <p>foo <a href=bar>
         * bar</a>.
         *
         * <p>foo-<a href=bar>
         * bar</a>.
         *
         * <p>foo<a href=bar>
         * bar</a>.
         *
         * <p><a href=foo>foo</a> bar.
         */
        class Test {}\
        """, """
        /**
         * 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 456789012 4567 <a
         * href=foo>foo</a>.
         *
         * <p>789012 456789012 456789012 456789012 456789012 456789012 456789012 456789 <a href=foo>foo</a>.
         *
         * <p>789012 456789012 456789012 456789012 456789012 456789012 456789012 4567890 <a href=foo>
         * foo</a>.
         *
         * <p><a href=foo>foo</a>.
         *
         * <p>foo <a href=bar>bar</a>.
         *
         * <p>foo-<a href=bar>bar</a>.
         *
         * <p>foo<a href=bar>bar</a>.
         *
         * <p><a href=foo>foo</a> bar.
         */
        class Test {}
        """);
    }
    @Test
  public void heading() {
        doFormatTest("""
        /**
         * abc<h1>def</h1>ghi
         */
        class Test {}\
        """, """
        /**
         * abc
         *
         * <h1>def</h1>
         *
         * ghi
         */
        class Test {}
        """);
    }
    @Test
  public void blockquote() {
        doFormatTest("""
        /**
         * abc<blockquote><p>def</blockquote>ghi
         */
        class Test {}\
        """, """
        /**
         * abc
         *
         * <blockquote>
         *
         * <p>def
         * </blockquote>
         *
         * ghi
         */
        class Test {}
        """);
    }
    @Test
  public void lists() {
        doFormatTest("""
        /**
        * hi
        *
        * <ul>
        * <li>
        * <ul>
        * <li>a</li>
        * </ul>
        * </li>
        * </ul>
        */
        class Test {}\
        """, """
        /**
         * hi
         *
         * <ul>
         *   <li>
         *       <ul>
         *         <li>a
         *       </ul>
         * </ul>
         */
        class Test {}
        """);
    }
    @Test
  public void lists2() {
        doFormatTest("""
        /**
         * Foo.
         *
         * <ul><li>1<ul><li>1a<li>1b</ul>more 1<p>still more 1<li>2</ul>
         */
        class Test {}\
        """, """
        /**
         * Foo.
         *
         * <ul>
         *   <li>1
         *       <ul>
         *         <li>1a
         *         <li>1b
         *       </ul>
         *       more 1
         *       <p>still more 1
         *   <li>2
         * </ul>
         */
        class Test {}
        """);
    }
    @Test
  public void closeInnerListStillNewline() {
        doFormatTest("""
        /**
         * Foo.
         *
         * <ul><li><ul><li>a</ul>b</ul>
         */
        class Test {}\
        """, """
        /**
         * Foo.
         *
         * <ul>
         *   <li>
         *       <ul>
         *         <li>a
         *       </ul>
         *       b
         * </ul>
         */
        class Test {}
        """);
    }
    @Test
  public void listItemWrap() {
        doFormatTest("""
        /**
         * Foo.
         *
         * <ul><li>234567890 234567890 234567890 234567890 234567890 234567890 234567890 234567890 234567890 234567890</ul>
         */
        class Test {}\
        """, """
        /**
         * Foo.
         *
         * <ul>
         *   <li>234567890 234567890 234567890 234567890 234567890 234567890 234567890 234567890 234567890
         *       234567890
         * </ul>
         */
        class Test {}
        """);
    }
    @Test
  public void unclosedList() {
        doFormatTest("""
        /**
         * Foo.
         *
         * <ul><li>1
         * @return blah
         */
        class Test {}\
        """, """
        /**
         * Foo.
         *
         * <ul>
         *   <li>1
         *
         * @return blah
         */
        class Test {}
        """);
    }
    @Test
  public void br() {
        doFormatTest("""
        /**
         * abc<br>def
         */
        class Test {}\
        """, """
        /**
         * abc<br>
         * def
         */
        class Test {}
        """);
    }
    @Test
  public void brSpaceBug() {
        doFormatTest("""
        /**
         * abc <br>def
         */
        class Test {}\
        """, """
        /**
         * abc <br>
         * def
         */
        class Test {}
        """);
    }
    @Test
  public void brAtSignBug() throws FormatterException {
        assertThat(formatter.formatSource("""
        /**
         * abc<br>@foo
         */
        class Test {}\
        """)).isEqualTo("""
        /**
         * abc<br>
         * @foo
         */
        class Test {}
        """);
    }
    @Test
  public void unicodeCharacterCountArguableBug() {
        doFormatTest("""
        /**
         * 456789𝄞12 456789𝄞12 456789𝄞12 456789𝄞12 456789𝄞12 456789𝄞12 456789𝄞12 456789𝄞12 456789𝄞12 456789𝄞
         */
        class Test {}\
        """, """
        /**
         * 456789𝄞12 456789𝄞12 456789𝄞12 456789𝄞12 456789𝄞12 456789𝄞12 456789𝄞12 456789𝄞12
         * 456789𝄞12 456789𝄞
         */
        class Test {}
        """);
    }
    @Test
  public void blankLinesAroundSnippetAndNoMangling() {
        doFormatTest("""
        /**
         * hello world
         * {@snippet :
         * public class Foo {
         *   private String s;
         * }
         * }
         * hello again
         */
        class Test {}\
        """, """
        /**
         * hello world
         *
         * {@snippet :
         * public class Foo {
         *   private String s;
         * }
         * }
         *
         * hello again
         */
        class Test {}
        """);
    }
    @Test
  public void notASnippetUnlessOuterTag() {
        doFormatTest("""
        /** I would like to tell you about the {@code {@snippet ...}} tag. */
        class Test {}\
        """, """
        /** I would like to tell you about the {@code {@snippet ...}} tag. */
        class Test {}
        """);
    }
    @Test
  public void blankLineBeforeParams() {
        doFormatTest("""
        /**
         * hello world
         * @param this is a param
         */
        class Test {}\
        """, """
        /**
         * hello world
         *
         * @param this is a param
         */
        class Test {}
        """);
    }
    @Test
  public void onlyParams() {
        doFormatTest("""
        /**
         *
         *
         * @param this is a param
         */
        class Test {}\
        """, """
        /**
         * @param this is a param
         */
        class Test {}
        """);
    }
    @Test
  public void paramsContinuationIndented() {
        doFormatTest("""
        /**
         * hello world
         *
         * @param foo 567890123 567890123 567890123 567890123 567890123 567890123 567890123 567890123 567890123
         * @param bar another
         */
        class Test {}\
        """, """
        /**
         * hello world
         *
         * @param foo 567890123 567890123 567890123 567890123 567890123 567890123 567890123 567890123
         *     567890123
         * @param bar another
         */
        class Test {}
        """);
    }
    @Test
  public void paramsOtherIndents() {
        doFormatTest("""
        /**
         * hello world
         *
         * @param foo a<p>b<ul><li>a<ul><li>x</ul></ul>
         * @param bar another
         */
        class Test {}\
        """, """
        /**
         * hello world
         *
         * @param foo a
         *     <p>b
         *     <ul>
         *       <li>a
         *           <ul>
         *             <li>x
         *           </ul>
         *     </ul>
         *
         * @param bar another
         */
        class Test {}
        """);
    }
    @Test
  public void paragraphTag() {
        doFormatTest("""
        class Test {
          /**
           * hello<p>world
           */
          void f() {}

          /**
           * hello
           * <p>
           * world
           */
          void f() {}
        }\
        """, """
        class Test {
          /**
           * hello
           *
           * <p>world
           */
          void f() {}

          /**
           * hello
           *
           * <p>world
           */
          void f() {}
        }
        """);
    }
    @Test
  public void xhtmlParagraphTag() {
        doFormatTest("""
        class Test {
          /**
           * hello<p/>world
           */
          void f() {}

        }\
        """, """
        class Test {
          /**
           * hello
           *
           * <p>world
           */
          void f() {}
        }
        """);
    }
    @Test
  public void removeInitialParagraphTag() {
        doFormatTest("""
        /**
         * <p>hello<p>world
         */
        class Test {}\
        """, """
        /**
         * hello
         *
         * <p>world
         */
        class Test {}
        """);
    }
    @Test
  public void inferParagraphTags() {
        doFormatTest("""
        /**
         *
         *
         * foo
         * foo
         *
         *
         * foo
         *
         * bar
         *
         * <pre>
         *
         * baz
         *
         * </pre>
         *
         * <ul>
         * <li>foo
         *
         * bar
         * </ul>
         *
         *
         */
        class Test {}\
        """, """
        /**
         * foo foo
         *
         * <p>foo
         *
         * <p>bar
         *
         * <pre>
         *
         * baz
         *
         * </pre>
         *
         * <ul>
         *   <li>foo
         *       <p>bar
         * </ul>
         */
        class Test {}
        """);
    }
    @Test
  public void paragraphTagNewlines() throws Exception {
        String input = new String(ByteStreams.toByteArray(getClass().getResourceAsStream("testjavadoc/B28750242.input")), UTF_8);
        String expected = new String(ByteStreams.toByteArray(getClass().getResourceAsStream("testjavadoc/B28750242.output")), UTF_8);
        assertThat(formatter.formatSource(input)).isEqualTo(expected);
    }
    @Test
  public void listItemSpaces() throws Exception {
        String input = new String(ByteStreams.toByteArray(getClass().getResourceAsStream("testjavadoc/B31404367.input")), UTF_8);
        String expected = new String(ByteStreams.toByteArray(getClass().getResourceAsStream("testjavadoc/B31404367.output")), UTF_8);
        assertThat(formatter.formatSource(input)).isEqualTo(expected);
    }
    @Test
  public void htmlTagsInCode() {
        doFormatTest("""
        /** abc {@code {} <p> <li> <pre> <table>} def */
        class Test {}\
        """, """
        /** abc {@code {} <p> <li> <pre> <table>} def */
        class Test {}
        """);
    }
    @Test
  public void loneBraceDoesNotStartInlineTag() {
        doFormatTest("""
        /** {  <p> } */
        class Test {}\
        """, """
        /**
         * {
         *
         * <p>}
         */
        class Test {}
        """);
    }
    @Test
  public void unicodeEscapesNotReplaced() {
        doFormatTest("""
        /** foo \\u0000 bar \\u6c34 baz */
        class Test {}\
        """, """
        /** foo \\u0000 bar \\u6c34 baz */
        class Test {}
        """);
    }
    @Test
  public void unicodeEscapesNotInterpretedBug() {
        doFormatTest("""
        /** a\\u003Cp>b */
        class Test {}\
        """, """
        /** a\\u003Cp>b */
        class Test {}
        """);
    }
    @Test
  public void trailingLink() {
        doFormatTest("""
        /**
         * abc {@link Foo}
         * def
         */
        class Test {}\
        """, """
        /** abc {@link Foo} def */
        class Test {}
        """);
    }
    @Test
  public void codeInCode() {
        doFormatTest("""
        /** abc {@code {@code foo}} def */
        class Test {}\
        """, """
        /** abc {@code {@code foo}} def */
        class Test {}
        """);
    }
    @Test
  public void quotedTextSplitAcrossLinks() {
        doFormatTest("""
        /**
         * abc "foo
         * bar baz" def
         */
        class Test {}\
        """, """
        /** abc "foo bar baz" def */
        class Test {}
        """);
    }
    @Test
  public void standardizeTags() {
        doFormatTest("""
        /**
         * foo
         *
         * <P>bar
         *
         * <p class=clazz>baz<BR>
         * baz
         */
        class Test {}\
        """, """
        /**
         * foo
         *
         * <p>bar
         *
         * <p class=clazz>baz<br>
         * baz
         */
        class Test {}
        """);
    }
    @Test
  public void removeCloseTags() {
        doFormatTest("""
        /**
         * foo</p>
         *
         * <p>bar</p>
         */
        class Test {}\
        """, """
        /**
         * foo
         *
         * <p>bar
         */
        class Test {}
        """);
    }
    @Test
  public void javadocFullSentences() {
        doFormatTest("""
        /** In our application, bats are often found hanging from the ceiling, especially on Wednesdays.  Sometimes sick bats have issues where their claws do not close entirely.  This class provides a nice, grippable surface for them to cling to. */
        class Grippable {}\
        """, """
        /**
         * In our application, bats are often found hanging from the ceiling, especially on Wednesdays.
         * Sometimes sick bats have issues where their claws do not close entirely. This class provides a
         * nice, grippable surface for them to cling to.
         */
        class Grippable {}
        """);
    }
    @Test
  public void javadocSentenceFragment() {
        doFormatTest("""
        /** Provides a comfy, grippable surface for sick bats with claw-closing problems, which are sometimes found hanging from the ceiling on Wednesdays. */
        class Grippable {}\
        """, """
        /**
         * Provides a comfy, grippable surface for sick bats with claw-closing problems, which are sometimes
         * found hanging from the ceiling on Wednesdays.
         */
        class Grippable {}
        """);
    }
    @Test
  public void javadocCanEndAnywhere() {
        doFormatTest("""
        /** foo <pre*/
        class Test {}\
        """, """
        /** foo <pre */
        class Test {}
        """);
    }
    @Test
  public void windowsLineSeparator() throws FormatterException {
        String input = """
        /**
         * hello
         *
         * <p>world
         */
        class Test {}\
        """;
        for (String separator : new String[] {"\r", "\r\n"}) {
            assertThat(formatter.formatSource(input.replace("\n", separator))).isEqualTo(input.replace("\n", separator) + separator);
        }
    }
    @Test
  public void u2028LineSeparator() {
        doFormatTest("        public class Foo {\n          /** \n           * Set and enable something.\n           */\n          public void setSomething() {}\n        }\\\n        ", "        public class Foo {\n          /**\n           *   Set and enable something.\n           */\n          public void setSomething() {}\n        }\n        ");
    }
    @Test
  public void missingSummaryFragment() {
        doFormatTest("""
        public class Foo {
          /**
           * @return something.
           */
          public void setSomething() {}

          /**
           * @hide
           */
          public void setSomething() {}
        }\
        """, """
        public class Foo {
          /**
           * @return something.
           */
          public void setSomething() {}

          /** @hide */
          public void setSomething() {}
        }
        """);
    }
    @Test
  public void simpleMarkdown() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
package com.example;

/// # Heading
///
/// A very long line of text, long enough that it will need to be wrapped to fit within the maximum line length.
///
/// A second paragraph.
class Test {
  /// Another very long line of text, also long enough that it will need to be wrapped to fit within the maximum line length.
  /// @param <T> a generic type
  <T> T method() {
    return null;
  }

  /// This long line of text looks like a javadoc comment, but is not, because it is separated from the actual javadoc comment by a plain comment.
  // This is the plain comment.
  /// A third very long line of text, this time a javadoc comment on a field, which again exceeds the maximum line length.
  String field;

  /// A fourth very long line of text, which however is not a javadoc comment so will be wrapped like a regular // comment.
}\
""", """
package com.example;

/// # Heading
///
/// A very long line of text, long enough that it will need to be wrapped to fit within the maximum
/// line length.
///
/// A second paragraph.
class Test {
  /// Another very long line of text, also long enough that it will need to be wrapped to fit within
  /// the maximum line length.
  ///
  /// @param <T> a generic type
  <T> T method() {
    return null;
  }

  /// This long line of text looks like a javadoc comment, but is not, because it is separated from
  // the actual javadoc comment by a plain comment.
  // This is the plain comment.
  /// A third very long line of text, this time a javadoc comment on a field, which again exceeds
  /// the maximum line length.
  String field;

  /// A fourth very long line of text, which however is not a javadoc comment so will be wrapped
  // like a regular // comment.
}
""");
    }
    @Test
  public void moduleMarkdown() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
/// A very long line of text, long enough that it will need to be wrapped to fit within the maximum line length.
module com.example {}
""", """
/// A very long line of text, long enough that it will need to be wrapped to fit within the maximum
/// line length.
module com.example {}
""");
    }
    @Test
  public void markdownLists() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
/// A list that contains:
/// - things
/// - very very long lines that are going to need to be wrapped with appropriate indentation on the next line
/// - item that unnecessarily
///   continues onto the next line
/// - a nested list
///   * nested thing 1
///   * nested thing 2
/// -    a nested numbered list with unnecessary leading whitespace
///      1. nested thing 1
///         on more than one line
///      2. nested thing 2 on only one line but which is long enough that it is going to need to be wrapped
///
///      3. nested thing 3 after a blank line
///
/// A following paragraph.
class Test {}
""", """
/// A list that contains:
/// - things
/// - very very long lines that are going to need to be wrapped with appropriate indentation on the
///   next line
/// - item that unnecessarily continues onto the next line
/// - a nested list
///   * nested thing 1
///   * nested thing 2
/// - a nested numbered list with unnecessary leading whitespace
///   1. nested thing 1 on more than one line
///   2. nested thing 2 on only one line but which is long enough that it is going to need to be
///      wrapped
///
///   3. nested thing 3 after a blank line
///
/// A following paragraph.
class Test {}
""");
    }
    @Test
  public void markdownIndentedListItem() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
/// A list:
///   - `foo`: enabled by default
///   - `bar`: disabled by default
class Test {}
""", """
/// A list:
/// - `foo`: enabled by default
/// - `bar`: disabled by default
class Test {}
""");
    }
    @Test
  public void markdownEmptyListItem() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        String input = """
/// A list with an empty item:
/// - `foo`: enabled by default
/// - `bar`: disabled by default
/// -
class Test {}
""";
        doFormatTest(input, input);
    }
    @Test
  public void markdownFencedCodeBlocks() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
/// ```
/// foo
/// bar
/// ```
///
/// -  ```
///    code block
///    in a list
///        with an indented line
///    ```
///
/// - flibbertigibbet
///   ```
///   code block in a list after text with no blank line intervening (one will be inserted)
///   ```
///
/// - flibbertigibbet
///
///   ```
///   code block in a list after text with a blank line intervening
///   ```
///
/// ~~~java
/// code block
/// with tildes and an info string ("java")
/// ~~~
///
///  ````
///  code block
///  with more than three backticks and an extra leading space
///  ````
class Test {}
""", """
/// ```
/// foo
/// bar
/// ```
///
/// - ```
///   code block
///   in a list
///       with an indented line
///   ```
///
/// - flibbertigibbet
///
///   ```
///   code block in a list after text with no blank line intervening (one will be inserted)
///   ```
///
/// - flibbertigibbet
///
///   ```
///   code block in a list after text with a blank line intervening
///   ```
///
/// ~~~java
/// code block
/// with tildes and an info string ("java")
/// ~~~
///
/// ````
/// code block
/// with more than three backticks and an extra leading space
/// ````
class Test {}
""");
    }
    @Test
  public void markdownMoeComments() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("        /// This is the first line.\n        ///\n        /// <!-- MOE:begin_intracomment_strip -->\n        /// This is a comment that should be stripped\n        ///\n        /// ```\n        /// this is a code block\n        /// ```\n        /// <!-- MOE:end_intracomment_strip -->\n        class Test {}\n        ", "        /// This is the first line.\n        ///\n        /// <!-- MOE:begin_intracomment_strip -->\n        /// This is a comment that should be stripped\n        ///\n        /// ```\n        /// this is a code block\n        /// ```\n        /// <!-- MOE:end_intracomment_strip -->\n        class Test {}\n        ");
    }
    @Test
  public void markdownBackslashes() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        @SuppressWarnings("MisleadingEscapedSpace")
            String input =
        """
        /// ╲<br> is not a break.
        /// ╲&#42; is not an HTML entity.
        /// Backslash does not escape the end of a `code span╲` so <br> is a real break,
        /// but backslash does escape the *start* of a ╲`code span so <br> is also a real break.
        /// hard╲
        /// line╲\t\s
        /// breaks
        /// - foo ╲
        ///     bar
        /// ╲@param not a param tag
        /// ╲╲@param not a param tag either
        class Test {}
        """
                    .replace('╲', '\\');
            // I don't think anything changes if we do or do not respect the \& backslash so nothing here
            // proves whether we do.
        doFormatTest(input, "/// \\<br> is not a break. \\&#42; is not an HTML entity. Backslash does not escape the end of a `code\n/// span\\` so <br>\n/// is a real break, but backslash does escape the *start* of a \\`code span so <br>\n/// is also a real break. hard\\\n/// line\\\n/// breaks\n/// - foo \\\n///   bar \\@param not a param tag \\\\@param not a param tag either\nclass Test {}\n");
    }
    @Test
  public void markdownLinkWrapping() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
/// enough text to cause the following link to need to be wrapped [foo](http://very.long/url/that/will/need/to/be/wrapped)
class Test {}
""", """
/// enough text to cause the following link to need to be wrapped
/// [foo](http://very.long/url/that/will/need/to/be/wrapped)
class Test {}
""");
    }
    @Test
  public void markdownThematicBreaks() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        String input = """
        /// foo
        /// ***
        /// bar
        /// baz
        ///
        ///
        class Test {}
        """;
        doFormatTest(input, input);
    }
    @Test
  public void markdownSetextHeadings() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        String input = """
        /// Heading
        /// =======
        /// Phoebe B. Peabody-Beebe
        ///
        ///
        /// Subheading
        /// ----------
        class Test {}
        """;
        doFormatTest(input, input);
    }
    @Test
  public void markdownIndentedCodeBlocks() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        String input = """
        ///     code block
        ///     is indented
        /// text after code block
        ///
        ///
        class Test {}
        """;
        doFormatTest(input, input);
    }
    @Test
  public void markdownBlockTagContinuationLines() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
/// Something something something.
///
/// @param foo a parameter with a long description that will need to be wrapped onto multiple
/// lines, with each line being indented by the default amount for continuation
/// lines in a block tag.
///
/// There is even a second paragraph which illustrates why the indentation should be +2
/// rather than +4.
record Test(String foo) {}
""", """
/// Something something something.
///
/// @param foo a parameter with a long description that will need to be wrapped onto multiple lines,
///   with each line being indented by the default amount for continuation lines in a block tag.
///
///   There is even a second paragraph which illustrates why the indentation should be +2 rather
///   than +4.
record Test(String foo) {}
""");
    }
    @Test
  public void markdownLinkReferenceDefinitions() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        String input = """
        /// [foo] [bar]
        ///
        /// [foo]: /url "title"
        /// [bar]: /url2 "title2"
        ///
        ///
        class Test {}
        """;
        doFormatTest(input, input);
    }
    @Test
  public void markdownLooseLists() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        String input = """
        /// - item 1
        ///
        /// - item 2
        class Test {}
        """;
        doFormatTest(input, input);
    }
    @Test
  public void markdownListPrecedingBlankLine() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        String input = """
        /// Title.
        ///
        /// Some paragraph.
        ///
        /// - Item 1.
        /// - Item 2.
        class Test {}
        """;
        doFormatTest(input, input);
    }
    @Test
  public void markdownBlockQuotes() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
        /// > foo
        /// > bar
        ///
        /// baz
        class Test {}
        """, """
        /// > foo bar
        ///
        /// baz
        class Test {}
        """);
    }
    @Test
  public void markdownBlockQuoteWithinListItem() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
        /// 1. item one
        /// 2. item two
        ///    - sublist
        ///    - sublist
        ///    - > foo
        ///      > bar
        class Test {}
        """, """
        /// 1. item one
        /// 2. item two
        ///    - sublist
        ///    - sublist
        ///    - > foo bar
        class Test {}
        """);
    }
    @Test
  public void markdownNestedBlockQuotes() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
        /// > foo
        /// > > bar
        /// > > baz
        class Test {}
        """, """
        /// > foo
        /// > > bar baz
        class Test {}
        """);
    }
    @Test
  public void markdownBlockQuoteWithCodeBlockInside() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
        /// > foo
        /// > ```
        /// > >
        /// > ```
        /// > bar
        class Test {}
        """, """
        /// > foo
        /// >
        /// > ```
        /// > >
        /// > ```
        /// >
        /// > bar
        class Test {}
        """);
    }
    @Test
  public void markdownBlockQuoteInBlockTag() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
/// A test class.
/// @param foo a Foo
/// > In the reign of James the Second
/// > It was generally reckoned
/// > As a very serious crime
/// > To marry two wives at one time.
class Test {}
""", """
/// A test class.
///
/// @param foo a Foo
///   > In the reign of James the Second It was generally reckoned As a very serious crime To marry
///   > two wives at one time.
class Test {}
""");
    }
    @Test
  public void markdownCodeSpans() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
/// `<ul>` should not trigger list handling.
///
/// `This very long code line should eventually trigger line wrapping because newlines are allowed in code spans.`
///
/// This other long line is carefully crafted to provoke a line break inside a double-backtick `` `<ul>` `` code span.
///
/// There should not be a line break immediately before or after a backtick in an example like this`and`that.
class Test {}
""", """
/// `<ul>` should not trigger list handling.
///
/// `This very long code line should eventually trigger line wrapping because newlines are allowed
/// in code spans.`
///
/// This other long line is carefully crafted to provoke a line break inside a double-backtick ``
/// `<ul>` `` code span.
///
/// There should not be a line break immediately before or after a backtick in an example like
/// this`and`that.
class Test {}
""");
    }
    @Test
  public void markdownAutolinks() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        String input = """
        /// <http://{@code:example.com> <br> is not inside @code}.
        class Test {}
        """;
        doFormatTest(input, input);
    }
    @Test
  public void markdownTables() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        String input = """
/// Table McTableface
///
/// | foo | bar |
/// | --- | --- |
/// | baz | qux |
///
/// - |foo|bar|
///   |--:|:--|
///   |baz|qux|
///
/// - Another list.
///
///   | which | contains |
///   | ----- | -------- |
///   | a | table |
class Test {}
""";
        doFormatTest(input, input);
    }
    @Test
  public void markdownBlankLinesAroundSnippetAndNoMangling() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
        /// hello world
        /// {@snippet :
        /// public class Foo {
        ///   private String s;
        /// }
        /// }
        /// hello again
        class Test {}\
        """, """
        /// hello world
        ///
        /// {@snippet :
        /// public class Foo {
        ///   private String s;
        /// }
        /// }
        ///
        /// hello again
        class Test {}
        """);
    }
    @Test
  public void markdownLongCommentOnEnumConstant() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
enum Foo {
  /// Long long long, longedy long, más fada an lá tig an oíche, chaise longue, caffè lungo, ich bin so lang nicht bei dir gewest
  BAR,
}
""", """
enum Foo {
  /// Long long long, longedy long, más fada an lá tig an oíche, chaise longue, caffè lungo, ich bin
  /// so lang nicht bei dir gewest
  BAR,
}
""");
    }
    @Test
  public void markdownLongCommentOnPackageDeclaration() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
/// Long long long, longedy long, más fada an lá tig an oíche, chaise longue, caffè lungo, ich bin so lang nicht bei dir gewest
package com.example;
""", """
/// Long long long, longedy long, más fada an lá tig an oíche, chaise longue, caffè lungo, ich bin
/// so lang nicht bei dir gewest
package com.example;
""");
    }
    @Test
  public void markdownOrderedListWithParenthesis() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        String input = """
        /// Summary paragraph.
        /// 1) first item
        /// 2) second item
        class Test {}
        """;
        doFormatTest(input, input);
    }
    @Test
  public void markdownBulletListWithPlus() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        String input = """
        /// Summary paragraph.
        /// + first item
        /// + second item
        class Test {}
        """;
        doFormatTest(input, input);
    }
    @Test
  public void tableAtStartOfComment() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        String input = """
        /// <table>
        /// <tr><td>Foo</td></tr>
        /// </table>
        class Test {}
        """;
        doFormatTest(input, input);
    }
    @Test
  public void blockquoteAtStartOfComment() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
        /// <blockquote>
        /// line 1
        /// line 2
        /// </blockquote>
        class Test {}
        """, """
        /// <blockquote>
        /// line 1 line 2
        /// </blockquote>
        class Test {}
        """);
    }
    @Test
  public void preAtStartOfComment() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        String input = """
        /// <pre>
        /// line 1
        /// line 2
        /// </pre>
        class Test {}
        """;
        doFormatTest(input, input);
    }
    @Test
  public void tableInHtmlListItem() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
        /// <ul>
        /// <li>
        /// <table>
        /// <tr><td>Foo</td></tr>
        /// </table>
        /// </li>
        /// </ul>
        class Test {}
        """, """
        /// <ul>
        ///   <li>
        ///
        ///       <table>
        /// <tr><td>Foo</td></tr>
        /// </table>
        ///
        /// </ul>
        class Test {}
        """);
    }
    @Test
  public void tableInsideMarkdownListItemAfterText() {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        doFormatTest("""
        /// - item text
        ///   <table>
        ///   <tr><td>Foo</td></tr>
        ///   </table>
        class Test {}
        """, """
        /// - item text
        ///
        ///   <table>
        ///   <tr><td>Foo</td></tr>
        ///   </table>
        class Test {}
        """);
    }
    @Test
  public void markdownNoFormatJavadoc() throws Exception {
        assume().that(MARKDOWN_JAVADOC_SUPPORTED).isTrue();
        String input = """
        /// A very long line of text, long enough that it would be wrapped if javadoc formatting were enabled.
        class Test {
          /// Another very long line of text, also long enough that it would be wrapped if javadoc formatting were enabled.
          void method() {}
        }
        """;
        assertThat(new Formatter(JavaFormatterOptions.builder().formatJavadoc(false).build()).formatSource(input)).isEqualTo(input);
    }
}
