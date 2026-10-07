package com.google.googlejavaformat.java.javadoc;

import static com.google.common.base.Verify.verify;
import com.google.common.collect.ArrayListMultimap;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableListMultimap;
import com.google.common.collect.ListMultimap;
import com.google.googlejavaformat.java.javadoc.Token.HeaderCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.HeaderOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.ListCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.ListItemCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.ListItemOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.ListOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.MarkdownBlockQuoteClose;
import com.google.googlejavaformat.java.javadoc.Token.MarkdownBlockQuoteOpen;
import com.google.googlejavaformat.java.javadoc.Token.MarkdownCodeSpanEnd;
import com.google.googlejavaformat.java.javadoc.Token.MarkdownCodeSpanStart;
import com.google.googlejavaformat.java.javadoc.Token.MarkdownFencedCodeBlock;
import com.google.googlejavaformat.java.javadoc.Token.MarkdownTable;
import com.google.googlejavaformat.java.javadoc.Token.ParagraphCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.ParagraphOpenTag;
import java.util.Objects;
import java.util.function.Function;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import org.commonmark.ext.gfm.tables.TableBlock;
import org.commonmark.ext.gfm.tables.TablesExtension;
import org.commonmark.node.BlockQuote;
import org.commonmark.node.BulletList;
import org.commonmark.node.Code;
import org.commonmark.node.FencedCodeBlock;
import org.commonmark.node.Heading;
import org.commonmark.node.IndentedCodeBlock;
import org.commonmark.node.LinkReferenceDefinition;
import org.commonmark.node.ListItem;
import org.commonmark.node.Node;
import org.commonmark.node.OrderedList;
import org.commonmark.node.Paragraph;
import org.commonmark.node.ThematicBreak;
import org.commonmark.parser.IncludeSourceSpans;
import org.commonmark.parser.Parser;

final class MarkdownPositions {
    final ImmutableListMultimap<Integer, Token> positionToToken;
    private MarkdownPositions(ImmutableListMultimap<Integer, Token> positionToToken) {
        this.positionToToken = positionToToken;
    }
    static final MarkdownPositions EMPTY = new MarkdownPositions(ImmutableListMultimap.of());
    static MarkdownPositions parse(String input) {
        Node document = PARSER.parse(input);
        ListMultimap<Integer, Token> positionToToken = ArrayListMultimap.create();
        new TokenVisitor(input, positionToToken).visit(document);
        return new MarkdownPositions(ImmutableListMultimap.copyOf(positionToToken));
    }
    ImmutableList<Token> tokensAt(int position) {
        return positionToToken.get(position);
    }
    private static class TokenVisitor {
        private final String input;
        private final ListMultimap<Integer, Token> positionToToken;
        TokenVisitor(String input, ListMultimap<Integer, Token> positionToToken) {
            this.input = input;
            this.positionToToken = positionToToken;
        }
        void visit(Node node) {
            boolean alreadyVisitedChildren = false;
            switch (node) {
                case Heading heading -> visitHeading(heading);
                case Paragraph paragraph -> addSpan(paragraph, PARAGRAPH_OPEN_TOKEN, PARAGRAPH_CLOSE_TOKEN);
                case BulletList bulletList -> addSpan(bulletList, LIST_OPEN_TOKEN, LIST_CLOSE_TOKEN);
                case OrderedList orderedList -> addSpan(orderedList, LIST_OPEN_TOKEN, LIST_CLOSE_TOKEN);
                case ListItem listItem -> alreadyVisitedChildren = visitListItem(listItem);
                case FencedCodeBlock fencedCodeBlock -> visitFencedCodeBlock(fencedCodeBlock);
                case TableBlock tableBlock -> {
                    visitTableBlock(tableBlock);
                    alreadyVisitedChildren = true;
                }
                case Code code -> visitCodeSpan(code);
                case BlockQuote blockQuote -> alreadyVisitedChildren = visitBlockQuote(blockQuote);
                case IndentedCodeBlock indentedCodeBlock -> throw new UnsupportedOperationException("Indented code blocks not supported");
                case ThematicBreak thematicBreak -> throw new UnsupportedOperationException("Thematic breaks not supported");
                case LinkReferenceDefinition linkReferenceDefinition -> throw new UnsupportedOperationException("Link reference definitions not supported");
                default -> {}
            }
            if (!alreadyVisitedChildren) {
                visitNodeList(node.getFirstChild());
            }
        }
        private boolean visitListItem(ListItem listItem) {
            return visitListItemOrBlockQuote(listItem, startPosition(listItem) + listItem.getMarkerIndent(), LIST_ITEM_START_PATTERN, ListItemOpenTag::new, LIST_ITEM_CLOSE_TOKEN);
        }
        private boolean visitBlockQuote(BlockQuote blockQuote) {
            return visitListItemOrBlockQuote(blockQuote, input.indexOf('>', startPosition(blockQuote)), BLOCKQUOTE_START_PATTERN, MarkdownBlockQuoteOpen::new, BLOCKQUOTE_CLOSE_TOKEN);
        }
        private boolean visitListItemOrBlockQuote(Node node, int start, Pattern startPattern, Function<String, Token> openTokenFactory, Token closeToken) {
            Matcher matcher = startPattern.matcher(input).region(start, input.length());
            verify(matcher.lookingAt());
            addSpan(node, openTokenFactory.apply(matcher.group(1)), closeToken, matcher.start(1));
            if (node.getFirstChild() instanceof Paragraph paragraph) {
                visitNodeList(paragraph.getFirstChild());
                visitNodeList(paragraph.getNext());
                return true;
            }
            return false;
        }
        private void visitHeading(Heading heading) {
            String s = nodeString(heading);
            if (s.contains("\n")) {
                throw new UnsupportedOperationException("Unsupported heading: " + s);
            }
            addSpan(heading, HEADER_OPEN_TOKEN, HEADER_CLOSE_TOKEN);
        }
        private void visitFencedCodeBlock(FencedCodeBlock fencedCodeBlock) {
            int start = startPosition(fencedCodeBlock) + fencedCodeBlock.getFenceIndent();
            int closingLength = Objects.requireNonNullElse(fencedCodeBlock.getClosingFenceLength(), fencedCodeBlock.getOpeningFenceLength());
            MarkdownFencedCodeBlock token = new MarkdownFencedCodeBlock(input.substring(start, endPosition(fencedCodeBlock)), fencedCodeBlock.getFenceCharacter().repeat(fencedCodeBlock.getOpeningFenceLength()) + fencedCodeBlock.getInfo(), fencedCodeBlock.getFenceCharacter().repeat(closingLength), fencedCodeBlock.getLiteral());
            positionToToken.get(start).addLast(token);
        }
        private void visitTableBlock(TableBlock tableBlock) {
            int start = startPosition(tableBlock);
            int end = endPosition(tableBlock);
            positionToToken.get(start).addLast(new MarkdownTable(input.substring(start, end)));
        }
        private void visitCodeSpan(Code code) {
            int start = startPosition(code);
            int end = endPosition(code);
            int count;
            for (count = 0; input.charAt(start + count) == '`'; count++) {
                verify(input.charAt(end - 1 - count) == '`', "Mismatched backticks: %s", input.substring(start, end));
            }
            verify(count > 0, "Code span does not start with backticks: %s", input.substring(start, end));
            String backticks = "`".repeat(count);
            positionToToken.get(start).addLast(new MarkdownCodeSpanStart(backticks));
            positionToToken.get(end - count).addFirst(new MarkdownCodeSpanEnd(backticks));
        }
        private void visitNodeList(Node node) {
            for (; node != null; node = node.getNext()) {
                visit(node);
            }
        }
        private void addSpan(Node node, Token startToken, Token endToken) {
            addSpan(node, startToken, endToken, startPosition(node));
        }
        private void addSpan(Node node, Token startToken, Token endToken, int startPosition) {
            positionToToken.get(startPosition).addLast(startToken);
            positionToToken.get(endPosition(node)).addFirst(endToken);
        }
        private int startPosition(Node node) {
            return node.getSourceSpans().getFirst().getInputIndex();
        }
        private int endPosition(Node node) {
            var last = node.getSourceSpans().getLast();
            return last.getInputIndex() + last.getLength();
        }
        private String nodeString(Node node) {
            return input.substring(startPosition(node), endPosition(node));
        }
    }
    @Override
  public String toString() {
        return positionToToken.toString();
    }
    private static final Parser PARSER = Parser.builder().includeSourceSpans(IncludeSourceSpans.BLOCKS_AND_INLINES).extensions(ImmutableList.of(TablesExtension.create())).build();
    private static final HeaderOpenTag HEADER_OPEN_TOKEN = new HeaderOpenTag("");
    private static final HeaderCloseTag HEADER_CLOSE_TOKEN = new HeaderCloseTag("");
    private static final ParagraphOpenTag PARAGRAPH_OPEN_TOKEN = new ParagraphOpenTag("");
    private static final ParagraphCloseTag PARAGRAPH_CLOSE_TOKEN = new ParagraphCloseTag("");
    private static final ListOpenTag LIST_OPEN_TOKEN = new ListOpenTag("");
    private static final ListCloseTag LIST_CLOSE_TOKEN = new ListCloseTag("");
    private static final ListItemCloseTag LIST_ITEM_CLOSE_TOKEN = new ListItemCloseTag("");
    private static final MarkdownBlockQuoteClose BLOCKQUOTE_CLOSE_TOKEN = new MarkdownBlockQuoteClose("");
    private static final Pattern BLOCKQUOTE_START_PATTERN = Pattern.compile("(> ?)");
    private static final Pattern LIST_ITEM_START_PATTERN = Pattern.compile("(([-+*]|[0-9]+[.)])(?:\\s|$))");
}
