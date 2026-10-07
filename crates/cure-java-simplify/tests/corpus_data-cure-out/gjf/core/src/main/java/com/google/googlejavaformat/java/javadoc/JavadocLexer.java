package com.google.googlejavaformat.java.javadoc;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Verify.verify;
import static com.google.common.collect.Iterators.peekingIterator;
import static java.lang.String.format;
import static java.util.regex.Pattern.CASE_INSENSITIVE;
import static java.util.regex.Pattern.DOTALL;
import static java.util.regex.Pattern.compile;
import com.google.common.base.CharMatcher;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableSet;
import com.google.common.collect.PeekingIterator;
import com.google.googlejavaformat.java.javadoc.Token.BeginJavadoc;
import com.google.googlejavaformat.java.javadoc.Token.BlockQuoteCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.BlockQuoteMarker;
import com.google.googlejavaformat.java.javadoc.Token.BlockQuoteOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.BrTag;
import com.google.googlejavaformat.java.javadoc.Token.CodeCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.CodeOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.EndJavadoc;
import com.google.googlejavaformat.java.javadoc.Token.FooterJavadocTagStart;
import com.google.googlejavaformat.java.javadoc.Token.ForcedNewline;
import com.google.googlejavaformat.java.javadoc.Token.HeaderCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.HeaderOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.HtmlComment;
import com.google.googlejavaformat.java.javadoc.Token.ListCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.ListItemCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.ListItemOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.ListOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.Literal;
import com.google.googlejavaformat.java.javadoc.Token.MarkdownBlockQuoteClose;
import com.google.googlejavaformat.java.javadoc.Token.MarkdownBlockQuoteOpen;
import com.google.googlejavaformat.java.javadoc.Token.MarkdownCodeSpanEnd;
import com.google.googlejavaformat.java.javadoc.Token.MarkdownCodeSpanStart;
import com.google.googlejavaformat.java.javadoc.Token.MarkdownHardLineBreak;
import com.google.googlejavaformat.java.javadoc.Token.MoeBeginStripComment;
import com.google.googlejavaformat.java.javadoc.Token.MoeEndStripComment;
import com.google.googlejavaformat.java.javadoc.Token.OptionalLineBreak;
import com.google.googlejavaformat.java.javadoc.Token.ParagraphCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.ParagraphOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.PreCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.PreOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.SnippetBegin;
import com.google.googlejavaformat.java.javadoc.Token.SnippetEnd;
import com.google.googlejavaformat.java.javadoc.Token.TableCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.TableOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.Whitespace;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Deque;
import java.util.List;
import java.util.function.Function;
import java.util.regex.Pattern;

final class JavadocLexer {
    static ImmutableList<Token> lex(String input, boolean classicJavadoc) throws LexException {
        input = normalizeLineEndings(input);
        MarkdownPositions markdownPositions;
        if (classicJavadoc) {
            markdownPositions = MarkdownPositions.EMPTY;
        } else {
            try {
                markdownPositions = MarkdownPositions.parse(input);
            } catch (UnsupportedOperationException e) {
                throw new LexException(e);
            }
        }
        return new JavadocLexer(new CharStream(input), markdownPositions, classicJavadoc).generateTokens();
    }
    private static String normalizeLineEndings(String input) {
        return NON_UNIX_LINE_ENDING.matcher(input).replaceAll("\n");
    }
    private static final Pattern NON_UNIX_LINE_ENDING = Pattern.compile("\r\n?");
    enum NestingContext {
        HTML_PRE_CONTEXT, HTML_CODE_CONTEXT, MARKDOWN_CODE_CONTEXT, TABLE, SNIPPET_CONTEXT, BRACE_CONTEXT, INLINE_TAG_CONTEXT, BLOCKQUOTE
    }
    private final CharStream input;
    private final boolean classicJavadoc;
    private final MarkdownPositions markdownPositions;
    private final NestingStack<NestingContext> contextStack = new NestingStack<>();
    private boolean somethingSinceNewline;
    private JavadocLexer(CharStream input, MarkdownPositions markdownPositions, boolean classicJavadoc) {
        this.input = checkNotNull(input);
        this.markdownPositions = markdownPositions;
        this.classicJavadoc = classicJavadoc;
    }
    private ImmutableList<Token> generateTokens() throws LexException {
        ImmutableList.Builder<Token> tokens = ImmutableList.builder();
        Token token = new BeginJavadoc(classicJavadoc ? "/**" : "///");
        tokens.add(token);
        while (!input.isExhausted()) {
            boolean moreMarkdown;
            do {
                moreMarkdown = false;
                for (Token markdownToken : markdownPositions.tokensAt(input.position())) {
                    markdownToken = processMarkdownToken(markdownToken);
                    tokens.add(markdownToken);
                    if (!markdownToken.value().isEmpty()) {
                        verify(input.tryConsume(markdownToken.value()), "Did not consume markdown token: %s", markdownToken);
                        var unused = input.readAndResetRecorded();
                        moreMarkdown = true;
                    }
                }
            } while (moreMarkdown);
            if (input.isExhausted()) {
                break;
            }
            token = readToken();
            tokens.add(token);
        }
        for (Token markdownToken : markdownPositions.tokensAt(input.position())) {
            markdownToken = processMarkdownToken(markdownToken);
            tokens.add(markdownToken);
        }
        checkMatchingTags();
        token = new EndJavadoc(classicJavadoc ? "*/" : "");
        tokens.add(token);
        ImmutableList<Token> result = tokens.build();
        result = joinAdjacentLiteralsAndAdjacentWhitespace(result);
        if (classicJavadoc) {
            result = inferParagraphTags(result);
        }
        result = optionalizeSpacesAfterLinks(result);
        result = deindentPreCodeBlocks(result);
        return result;
    }
    private Token processMarkdownToken(Token markdownToken) {
        switch (markdownToken) {
            case MarkdownCodeSpanStart unused -> {
                contextStack.push(NestingContext.MARKDOWN_CODE_CONTEXT);
                return new Literal(markdownToken.value());
            }
            case MarkdownCodeSpanEnd unused -> {
                contextStack.popUntil(NestingContext.MARKDOWN_CODE_CONTEXT);
                return new Literal(markdownToken.value());
            }
            case MarkdownBlockQuoteOpen unused -> {
                contextStack.push(NestingContext.BLOCKQUOTE);
            }
            case MarkdownBlockQuoteClose unused -> {
                contextStack.popUntil(NestingContext.BLOCKQUOTE);
            }
            default -> {}
        }
        return markdownToken;
    }
    private Token readToken() throws LexException {
        Function<String, Token> tokenFactory = consumeToken();
        String value = input.readAndResetRecorded();
        return tokenFactory.apply(value);
    }
    private Function<String, Token> consumeToken() throws LexException {
        boolean preserveExistingFormatting = preserveExistingFormatting();
        if (input.tryConsumeRegex(NEWLINE_PATTERN)) {
            somethingSinceNewline = false;
            return preserveExistingFormatting ? ForcedNewline::new : Whitespace::new;
        }
        if (!classicJavadoc && !somethingSinceNewline && contextStack.contains(NestingContext.BLOCKQUOTE) && input.tryConsumeRegex(BLOCKQUOTE_MARKER_PATTERN)) {
            return BlockQuoteMarker::new;
        }
        if (input.tryConsume(" ") || input.tryConsume("\t")) {
            return preserveExistingFormatting ? Literal::new : Whitespace::new;
        }
        if (contextStack.contains(NestingContext.MARKDOWN_CODE_CONTEXT)) {
            verify(input.tryConsumeRegex(WORD_IN_CODE_SPAN_PATTERN));
            return Literal::new;
        }
        if (!classicJavadoc) {
            if (input.tryConsumeRegex(MARKDOWN_HARD_LINE_BREAK_PATTERN)) {
                somethingSinceNewline = false;
                return MarkdownHardLineBreak::new;
            } else if (input.tryConsumeRegex(BACKSLASH_PLUS_CHARACTER_PATTERN)) {
                somethingSinceNewline = true;
                return Literal::new;
            }
        }
        if (!somethingSinceNewline && input.tryConsumeRegex(FOOTER_TAG_PATTERN)) {
            checkMatchingTags();
            somethingSinceNewline = true;
            return FooterJavadocTagStart::new;
        }
        somethingSinceNewline = true;
        if (input.tryConsumeRegex(SNIPPET_TAG_OPEN_PATTERN)) {
            if (contextStack.containsAny(BRACE_CONTEXTS)) {
                contextStack.push(NestingContext.BRACE_CONTEXT);
                return Literal::new;
            } else {
                contextStack.push(NestingContext.SNIPPET_CONTEXT);
                return SnippetBegin::new;
            }
        } else if (input.tryConsumeRegex(INLINE_TAG_OPEN_PATTERN)) {
            contextStack.push(NestingContext.INLINE_TAG_CONTEXT);
            return Literal::new;
        } else if (input.tryConsume("{")) {
            if (contextStack.containsAny(BRACE_CONTEXTS)) {
                contextStack.push(NestingContext.BRACE_CONTEXT);
            }
            return Literal::new;
        } else if (input.tryConsume("}")) {
            var popped = contextStack.popIfIn(BRACE_CONTEXTS);
            return popped == NestingContext.SNIPPET_CONTEXT ? SnippetEnd::new : Literal::new;
        }
        if (contextStack.containsAny(TAG_CONTEXTS)) {
            verify(input.tryConsumeRegex(literalPattern()));
            return Literal::new;
        }
        if (input.tryConsumeRegex(PRE_OPEN_PATTERN)) {
            contextStack.push(NestingContext.HTML_PRE_CONTEXT);
            return preserveExistingFormatting ? Literal::new : PreOpenTag::new;
        } else if (input.tryConsumeRegex(PRE_CLOSE_PATTERN)) {
            contextStack.popUntil(NestingContext.HTML_PRE_CONTEXT);
            return preserveExistingFormatting() ? Literal::new : PreCloseTag::new;
        }
        if (input.tryConsumeRegex(CODE_OPEN_PATTERN)) {
            contextStack.push(NestingContext.HTML_CODE_CONTEXT);
            return preserveExistingFormatting ? Literal::new : CodeOpenTag::new;
        } else if (input.tryConsumeRegex(CODE_CLOSE_PATTERN)) {
            contextStack.popUntil(NestingContext.HTML_CODE_CONTEXT);
            return preserveExistingFormatting() ? Literal::new : CodeCloseTag::new;
        }
        if (input.tryConsumeRegex(TABLE_OPEN_PATTERN)) {
            contextStack.push(NestingContext.TABLE);
            return preserveExistingFormatting ? Literal::new : TableOpenTag::new;
        } else if (input.tryConsumeRegex(TABLE_CLOSE_PATTERN)) {
            contextStack.popUntil(NestingContext.TABLE);
            return preserveExistingFormatting() ? Literal::new : TableCloseTag::new;
        }
        if (preserveExistingFormatting) {
            verify(input.tryConsumeRegex(literalPattern()));
            return Literal::new;
        }
        if (input.tryConsumeRegex(PARAGRAPH_OPEN_PATTERN)) {
            return ParagraphOpenTag::new;
        } else if (input.tryConsumeRegex(PARAGRAPH_CLOSE_PATTERN)) {
            return ParagraphCloseTag::new;
        } else if (input.tryConsumeRegex(LIST_OPEN_PATTERN)) {
            return ListOpenTag::new;
        } else if (input.tryConsumeRegex(LIST_CLOSE_PATTERN)) {
            return ListCloseTag::new;
        } else if (input.tryConsumeRegex(LIST_ITEM_OPEN_PATTERN)) {
            return ListItemOpenTag::new;
        } else if (input.tryConsumeRegex(LIST_ITEM_CLOSE_PATTERN)) {
            return ListItemCloseTag::new;
        } else if (input.tryConsumeRegex(BLOCKQUOTE_OPEN_PATTERN)) {
            return BlockQuoteOpenTag::new;
        } else if (input.tryConsumeRegex(BLOCKQUOTE_CLOSE_PATTERN)) {
            return BlockQuoteCloseTag::new;
        } else if (input.tryConsumeRegex(HEADER_OPEN_PATTERN)) {
            return HeaderOpenTag::new;
        } else if (input.tryConsumeRegex(HEADER_CLOSE_PATTERN)) {
            return HeaderCloseTag::new;
        } else if (input.tryConsumeRegex(BR_PATTERN)) {
            return BrTag::new;
        } else if (input.tryConsumeRegex(MOE_BEGIN_STRIP_COMMENT_PATTERN)) {
            return MoeBeginStripComment::new;
        } else if (input.tryConsumeRegex(MOE_END_STRIP_COMMENT_PATTERN)) {
            return MoeEndStripComment::new;
        } else if (input.tryConsumeRegex(HTML_COMMENT_PATTERN)) {
            return HtmlComment::new;
        } else if (input.tryConsumeRegex(literalPattern())) {
            return Literal::new;
        }
        throw new AssertionError();
    }
    private boolean preserveExistingFormatting() {
        return contextStack.containsAny(PRESERVE_FORMATTING_CONTEXTS);
    }
    private void checkMatchingTags() throws LexException {
        if (!contextStack.isEmpty()) {
            throw new LexException();
        }
    }
    private static ImmutableList<Token> joinAdjacentLiteralsAndAdjacentWhitespace(List<Token> input) {
        ImmutableList.Builder<Token> output = ImmutableList.builder();
        StringBuilder accumulated = new StringBuilder();
        for (PeekingIterator<Token> tokens = peekingIterator(input.iterator()); tokens.hasNext(); ) {
            if (tokens.peek() instanceof Literal) {
                accumulated.append(tokens.next().value());
                continue;
            }
            if (accumulated.isEmpty()) {
                if (tokens.peek() instanceof Whitespace) {
                    output.add(new Whitespace(consumeAdjacentWhitespace(tokens)));
                } else {
                    output.add(tokens.next());
                }
                continue;
            }
            String seenWhitespace = consumeAdjacentWhitespace(tokens);
            if (tokens.peek() instanceof Literal literal && literal.value().startsWith("@")) {
                accumulated.append(" ");
                accumulated.append(tokens.next().value());
                continue;
            }
            output.add(new Literal(accumulated.toString()));
            accumulated.setLength(0);
            if (!seenWhitespace.isEmpty()) {
                output.add(new Whitespace(seenWhitespace));
            }
        }
        return output.build();
    }
    private static String consumeAdjacentWhitespace(PeekingIterator<Token> tokens) {
        StringBuilder seenWhitespace = new StringBuilder();
        while (tokens.peek() instanceof Whitespace) {
            seenWhitespace.append(tokens.next().value());
        }
        return seenWhitespace.toString();
    }
    private static ImmutableList<Token> inferParagraphTags(List<Token> input) {
        ImmutableList.Builder<Token> output = ImmutableList.builder();
        for (PeekingIterator<Token> tokens = peekingIterator(input.iterator()); tokens.hasNext(); ) {
            if (tokens.peek() instanceof Literal) {
                output.add(tokens.next());
                if (tokens.peek() instanceof Whitespace && hasMultipleNewlines(tokens.peek().value())) {
                    output.add(tokens.next());
                    if (tokens.peek() instanceof Literal) {
                        output.add(new ParagraphOpenTag("<p>"));
                    }
                }
            } else {
                output.add(tokens.next());
            }
        }
        return output.build();
    }
    private static ImmutableList<Token> optionalizeSpacesAfterLinks(List<Token> input) {
        ImmutableList.Builder<Token> output = ImmutableList.builder();
        for (PeekingIterator<Token> tokens = peekingIterator(input.iterator()); tokens.hasNext(); ) {
            if (tokens.peek() instanceof Literal && tokens.peek().value().matches("href=[^>]*>")) {
                output.add(tokens.next());
                if (tokens.peek() instanceof Whitespace) {
                    output.add(new OptionalLineBreak(tokens.next().value()));
                }
            } else {
                output.add(tokens.next());
            }
        }
        return output.build();
    }
    private static ImmutableList<Token> deindentPreCodeBlocks(List<Token> input) {
        ImmutableList.Builder<Token> output = ImmutableList.builder();
        for (PeekingIterator<Token> tokens = peekingIterator(input.iterator()); tokens.hasNext(); ) {
            if (!(tokens.peek() instanceof PreOpenTag)) {
                output.add(tokens.next());
                continue;
            }
            output.add(tokens.next());
            List<Token> initialNewlines = new ArrayList<>();
            while (tokens.hasNext() && tokens.peek() instanceof ForcedNewline) {
                initialNewlines.add(tokens.next());
            }
            if (!(tokens.peek() instanceof Literal) || !tokens.peek().value().matches("[ \t]*[{]@code")) {
                output.addAll(initialNewlines);
                output.add(tokens.next());
                continue;
            }
            deindentPreCodeBlock(output, tokens);
        }
        return output.build();
    }
    private static void deindentPreCodeBlock(ImmutableList.Builder<Token> output, PeekingIterator<Token> tokens) {
        Deque<Token> saved = new ArrayDeque<>();
        output.add(new Literal(tokens.next().value().trim()));
        while (tokens.hasNext() && !(tokens.peek() instanceof PreCloseTag)) {
            Token token = tokens.next();
            saved.addLast(token);
        }
        while (!saved.isEmpty() && saved.peekFirst() instanceof ForcedNewline) {
            saved.removeFirst();
        }
        while (!saved.isEmpty() && saved.peekLast() instanceof ForcedNewline) {
            saved.removeLast();
        }
        if (saved.isEmpty()) {
            return;
        }
        Token last = saved.peekLast();
        boolean trailingBrace = false;
        if (last instanceof Literal && last.value().endsWith("}")) {
            saved.removeLast();
            if (last.length() > 1) {
                saved.addLast(new Literal(last.value().substring(0, last.value().length() - 1)));
                saved.addLast(new ForcedNewline(null));
            }
            trailingBrace = true;
        }
        int trim = -1;
        for (Token token : saved) {
            if (token instanceof Literal) {
                int idx = CharMatcher.isNot(' ').indexIn(token.value());
                if (idx != -1 && (trim == -1 || idx < trim)) {
                    trim = idx;
                }
            }
        }
        output.add(new ForcedNewline("\n"));
        for (Token token : saved) {
            if (token instanceof Literal) {
                output.add(new Literal(trim > 0 && token.length() > trim ? token.value().substring(trim) : token.value()));
            } else {
                output.add(token);
            }
        }
        if (trailingBrace) {
            output.add(new Literal("}"));
        } else {
            output.add(new ForcedNewline("\n"));
        }
    }
    private static final ImmutableSet<NestingContext> TAG_CONTEXTS = ImmutableSet.of(NestingContext.SNIPPET_CONTEXT, NestingContext.INLINE_TAG_CONTEXT);
    private static final ImmutableSet<NestingContext> BRACE_CONTEXTS = ImmutableSet.of(NestingContext.SNIPPET_CONTEXT, NestingContext.INLINE_TAG_CONTEXT, NestingContext.BRACE_CONTEXT);
    private static final ImmutableSet<NestingContext> PRESERVE_FORMATTING_CONTEXTS = ImmutableSet.of(NestingContext.HTML_PRE_CONTEXT, NestingContext.TABLE, NestingContext.HTML_CODE_CONTEXT, NestingContext.SNIPPET_CONTEXT);
    private static final CharMatcher NEWLINE = CharMatcher.is('\n');
    static boolean hasMultipleNewlines(String s) {
        return NEWLINE.countIn(s) > 1;
    }
    private static final Pattern NEWLINE_PATTERN = compile("[ \t]*\n");
    private static final Pattern BLOCKQUOTE_MARKER_PATTERN = compile("> ?");
    private static final Pattern FOOTER_TAG_PATTERN = compile("@(param\\s+<\\w+>|[a-z]\\w*)");
    private static final Pattern MOE_BEGIN_STRIP_COMMENT_PATTERN = compile("<!--\\s*MOE:begin_intracomment_strip\\s*-->");
    private static final Pattern MOE_END_STRIP_COMMENT_PATTERN = compile("<!--\\s*MOE:end_intracomment_strip\\s*-->");
    private static final Pattern HTML_COMMENT_PATTERN = compile("<!--.*?-->", DOTALL);
    private static final Pattern PRE_OPEN_PATTERN = openTagPattern("pre");
    private static final Pattern PRE_CLOSE_PATTERN = closeTagPattern("pre");
    private static final Pattern CODE_OPEN_PATTERN = openTagPattern("code");
    private static final Pattern CODE_CLOSE_PATTERN = closeTagPattern("code");
    private static final Pattern TABLE_OPEN_PATTERN = openTagPattern("table");
    private static final Pattern TABLE_CLOSE_PATTERN = closeTagPattern("table");
    private static final Pattern LIST_OPEN_PATTERN = openTagPattern("ul|ol|dl");
    private static final Pattern LIST_CLOSE_PATTERN = closeTagPattern("ul|ol|dl");
    private static final Pattern LIST_ITEM_OPEN_PATTERN = openTagPattern("li|dt|dd");
    private static final Pattern LIST_ITEM_CLOSE_PATTERN = closeTagPattern("li|dt|dd");
    private static final Pattern HEADER_OPEN_PATTERN = openTagPattern("h[1-6]");
    private static final Pattern HEADER_CLOSE_PATTERN = closeTagPattern("h[1-6]");
    private static final Pattern PARAGRAPH_OPEN_PATTERN = openTagPattern("p");
    private static final Pattern PARAGRAPH_CLOSE_PATTERN = closeTagPattern("p");
    private static final Pattern BLOCKQUOTE_OPEN_PATTERN = openTagPattern("blockquote");
    private static final Pattern BLOCKQUOTE_CLOSE_PATTERN = closeTagPattern("blockquote");
    private static final Pattern BR_PATTERN = openTagPattern("br");
    private static final Pattern SNIPPET_TAG_OPEN_PATTERN = compile("[{]@snippet\\b");
    private static final Pattern INLINE_TAG_OPEN_PATTERN = compile("[{]@\\w*");
    private static final Pattern WORD_IN_CODE_SPAN_PATTERN = compile(".[^ \t\n`]*");
    private static final Pattern MARKDOWN_HARD_LINE_BREAK_PATTERN = compile("\\\\[ \t]*\n");
    private static final Pattern BACKSLASH_PLUS_CHARACTER_PATTERN = compile("\\\\.");
    private static final Pattern CLASSIC_LITERAL_PATTERN = compile(".[^ \t\n@<{}*]*", DOTALL);
    private static final Pattern MARKDOWN_LITERAL_PATTERN = compile(".\\p{IsAlphabetic}*", DOTALL);
    private Pattern literalPattern() {
        return classicJavadoc ? CLASSIC_LITERAL_PATTERN : MARKDOWN_LITERAL_PATTERN;
    }
    private static Pattern openTagPattern(String namePattern) {
        return compile(format("<(?:%s)\\b[^>]*>", namePattern), CASE_INSENSITIVE);
    }
    private static Pattern closeTagPattern(String namePattern) {
        return compile(format("</(?:%s)\\b[^>]*>", namePattern), CASE_INSENSITIVE);
    }
    static class LexException extends Exception {
        LexException() {}
        LexException(Throwable cause) {
            super(cause);
        }
    }
}
