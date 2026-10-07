package com.google.googlejavaformat.java.javadoc;

import static com.google.common.collect.ImmutableListMultimap.flatteningToImmutableListMultimap;
import static com.google.common.truth.Truth.assertThat;
import com.google.common.collect.ImmutableListMultimap;
import com.google.googlejavaformat.java.javadoc.Token.HeaderCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.HeaderOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.ListCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.ListItemCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.ListItemOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.ListOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.MarkdownFencedCodeBlock;
import com.google.googlejavaformat.java.javadoc.Token.ParagraphCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.ParagraphOpenTag;
import java.util.Map;
import java.util.stream.IntStream;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public final class MarkdownPositionsTest {
    @Test
  public void empty() {
        String text = "";
        assertThat(positionToToken(MarkdownPositions.parse(text), text)).isEmpty();
    }
    @Test
  public void list() {
        String text = """
- foo
- bar
""";
        ImmutableListMultimap<Integer, Token> map = positionToToken(MarkdownPositions.parse(text), text);
        int firstBullet = text.indexOf('-');
        int secondBullet = text.lastIndexOf('-');
        int end = text.length() - 1;
        ImmutableListMultimap<Integer, Token> expected = ImmutableListMultimap.builder().put(firstBullet, new ListOpenTag("")).put(firstBullet, new ListItemOpenTag("- ")).put(secondBullet - 1, new ListItemCloseTag("")).put(secondBullet, new ListItemOpenTag("- ")).put(end, new ListItemCloseTag("")).put(end, new ListCloseTag("")).build();
        assertThat(map).isEqualTo(expected);
    }
    @Test
  public void listWithParenthesis() {
        String text = """
1) foo
2) bar
""";
        ImmutableListMultimap<Integer, Token> map = positionToToken(MarkdownPositions.parse(text), text);
        int firstItem = text.indexOf('1');
        int secondItem = text.indexOf('2');
        int end = text.length() - 1;
        ImmutableListMultimap<Integer, Token> expected = ImmutableListMultimap.builder().put(firstItem, new ListOpenTag("")).put(firstItem, new ListItemOpenTag("1) ")).put(secondItem - 1, new ListItemCloseTag("")).put(secondItem, new ListItemOpenTag("2) ")).put(end, new ListItemCloseTag("")).put(end, new ListCloseTag("")).build();
        assertThat(map).isEqualTo(expected);
    }
    @Test
  public void listWithPlus() {
        String text = """
+ foo
+ bar
""";
        ImmutableListMultimap<Integer, Token> map = positionToToken(MarkdownPositions.parse(text), text);
        int firstItem = text.indexOf('+');
        int secondItem = text.lastIndexOf('+');
        int end = text.length() - 1;
        ImmutableListMultimap<Integer, Token> expected = ImmutableListMultimap.builder().put(firstItem, new ListOpenTag("")).put(firstItem, new ListItemOpenTag("+ ")).put(secondItem - 1, new ListItemCloseTag("")).put(secondItem, new ListItemOpenTag("+ ")).put(end, new ListItemCloseTag("")).put(end, new ListCloseTag("")).build();
        assertThat(map).isEqualTo(expected);
    }
    @Test
  public void heading() {
        String text = """
# Foo

blah blah blah

## Bar

tiddly pom
""";
        ImmutableListMultimap<Integer, Token> map = positionToToken(MarkdownPositions.parse(text), text);
        int firstHeading = text.indexOf('#');
        int firstHeadingEnd = text.indexOf('\n', firstHeading);
        int firstParagraph = text.indexOf("blah");
        int firstParagraphEnd = text.indexOf('\n', firstParagraph);
        int secondHeading = text.indexOf('#', firstHeading + 1);
        int secondHeadingEnd = text.indexOf('\n', secondHeading);
        int secondParagraph = text.indexOf("tiddly");
        int secondParagraphEnd = text.indexOf('\n', secondParagraph);
        ImmutableListMultimap<Integer, Token> expected = ImmutableListMultimap.builder().put(firstHeading, new HeaderOpenTag("")).put(firstHeadingEnd, new HeaderCloseTag("")).put(firstParagraph, new ParagraphOpenTag("")).put(firstParagraphEnd, new ParagraphCloseTag("")).put(secondHeading, new HeaderOpenTag("")).put(secondHeadingEnd, new HeaderCloseTag("")).put(secondParagraph, new ParagraphOpenTag("")).put(secondParagraphEnd, new ParagraphCloseTag("")).build();
        assertThat(map).isEqualTo(expected);
    }
    @Test
  public void codeBlock() {
        String text = """
- ```
  foo
  bar
  ```

~~~java
code
with tildes
~~~

  ````
  indented code
  with more than three backticks
  ````
""";
        ImmutableListMultimap<Integer, Token> map = positionToToken(MarkdownPositions.parse(text), text);
        int bullet = text.indexOf('-');
        int firstCodeStart = text.indexOf("```");
        int firstCodeEnd = text.indexOf("```", firstCodeStart + 3) + 3;
        int secondCodeStart = text.indexOf("~~~", firstCodeEnd);
        int secondCodeEnd = text.indexOf("~~~", secondCodeStart + 3) + 3;
        int thirdCodeStart = text.indexOf("````", secondCodeEnd);
        int thirdCodeEnd = text.indexOf("````", thirdCodeStart + 4) + 4;
        ImmutableListMultimap<Integer, Token> expected = ImmutableListMultimap.builder().put(bullet, new ListOpenTag("")).put(bullet, new ListItemOpenTag("- ")).put(firstCodeStart, new MarkdownFencedCodeBlock(text.substring(firstCodeStart, firstCodeEnd), "```", "```", "foo\nbar\n")).put(firstCodeEnd, new ListItemCloseTag("")).put(firstCodeEnd, new ListCloseTag("")).put(secondCodeStart, new MarkdownFencedCodeBlock(text.substring(secondCodeStart, secondCodeEnd), "~~~java", "~~~", "code\nwith tildes\n")).put(thirdCodeStart, new MarkdownFencedCodeBlock(text.substring(thirdCodeStart, thirdCodeEnd), "````", "````", "indented code\nwith more than three backticks\n")).build();
        assertThat(map).isEqualTo(expected);
    }
    private static ImmutableListMultimap<Integer, Token> positionToToken(MarkdownPositions positions, String input) {
        return IntStream.rangeClosed(0, input.length()).mapToObj((i) -> Map.entry(i, positions.tokensAt(i))).collect(flatteningToImmutableListMultimap(Map.Entry::getKey, (e) -> e.getValue().stream()));
    }
}
