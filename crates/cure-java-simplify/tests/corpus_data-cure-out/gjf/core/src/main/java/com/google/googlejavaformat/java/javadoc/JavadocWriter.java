package com.google.googlejavaformat.java.javadoc;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.collect.Comparators.max;
import static com.google.googlejavaformat.java.javadoc.JavadocWriter.AutoIndent.AUTO_INDENT;
import static com.google.googlejavaformat.java.javadoc.JavadocWriter.AutoIndent.NO_AUTO_INDENT;
import static com.google.googlejavaformat.java.javadoc.JavadocWriter.RequestedWhitespace.BLANK_LINE;
import static com.google.googlejavaformat.java.javadoc.JavadocWriter.RequestedWhitespace.NEWLINE;
import static com.google.googlejavaformat.java.javadoc.JavadocWriter.RequestedWhitespace.NONE;
import static com.google.googlejavaformat.java.javadoc.JavadocWriter.RequestedWhitespace.WHITESPACE;
import com.google.googlejavaformat.java.javadoc.Token.BlockQuoteCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.BlockQuoteOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.BrTag;
import com.google.googlejavaformat.java.javadoc.Token.CodeCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.CodeOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.FooterJavadocTagStart;
import com.google.googlejavaformat.java.javadoc.Token.HeaderCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.HeaderOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.HtmlComment;
import com.google.googlejavaformat.java.javadoc.Token.ListCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.ListItemOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.ListOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.Literal;
import com.google.googlejavaformat.java.javadoc.Token.MarkdownBlockQuoteOpen;
import com.google.googlejavaformat.java.javadoc.Token.MarkdownFencedCodeBlock;
import com.google.googlejavaformat.java.javadoc.Token.MarkdownTable;
import com.google.googlejavaformat.java.javadoc.Token.MoeBeginStripComment;
import com.google.googlejavaformat.java.javadoc.Token.MoeEndStripComment;
import com.google.googlejavaformat.java.javadoc.Token.PreCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.PreOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.SnippetBegin;
import com.google.googlejavaformat.java.javadoc.Token.SnippetEnd;
import com.google.googlejavaformat.java.javadoc.Token.StartOfLineToken;
import com.google.googlejavaformat.java.javadoc.Token.TableCloseTag;
import com.google.googlejavaformat.java.javadoc.Token.TableOpenTag;
import com.google.googlejavaformat.java.javadoc.Token.Whitespace;
import java.util.List;

final class JavadocWriter {
    private static final Literal BACKSLASH_LITERAL = new Literal("\\");
    private final int blockIndent;
    private final boolean classicJavadoc;
    private final int maxLineLength;
    private final StringBuilder output = new StringBuilder();
    private boolean continuingListItemOfInnermostList;
    private boolean continuingFooterTag;
    private final NestingStack<Indent> indentStack = new NestingStack<>();
    private final NestingStack.Int postWriteModifiedContinuingListStack = new NestingStack.Int();
    private int remainingOnLine;
    private boolean atStartOfLine;
    private RequestedWhitespace requestedWhitespace = NONE;
    private Token requestedMoeBeginStripComment;
    private String indentForMoeEndStripComment = "";
    private boolean wroteAnythingSignificant;
    JavadocWriter(int blockIndent, boolean classicJavadoc, int maxLineLength) {
        this.blockIndent = blockIndent;
        this.classicJavadoc = classicJavadoc;
        this.maxLineLength = maxLineLength;
    }
    void requestWhitespace() {
        requestWhitespace(WHITESPACE);
    }
    private void requestWhitespace(RequestedWhitespace requestedWhitespace) {
        this.requestedWhitespace = max(requestedWhitespace, this.requestedWhitespace);
    }
    void requestWhitespaceOrBlankLine(Whitespace token) {
        if (!classicJavadoc && JavadocLexer.hasMultipleNewlines(token.value())) {
            requestBlankLine();
        } else {
            requestWhitespace();
        }
    }
    void requestMoeBeginStripComment(MoeBeginStripComment token) {
        requestedMoeBeginStripComment = checkNotNull(token);
    }
    void writeBeginJavadoc() {
        if (classicJavadoc) {
            output.append("/**");
            writeNewline();
        } else {
            output.append("/// ");
            remainingOnLine = maxLineLength - blockIndent - 4;
        }
    }
    void writeEndJavadoc() {
        if (classicJavadoc) {
            output.append("\n");
            appendSpaces(blockIndent + 1);
            output.append("*/");
        }
    }
    void writeFooterJavadocTagStart(FooterJavadocTagStart token) {
        indentStack.reset();
        postWriteModifiedContinuingListStack.reset();
        if (!continuingFooterTag) {
            requestBlankLine();
        } else {
            continuingFooterTag = false;
            requestNewline();
        }
        writeToken(token);
        continuingFooterTag = true;
    }
    void writeSnippetBegin(SnippetBegin token) {
        requestBlankLine();
        writeToken(token);
    }
    void writeSnippetEnd(SnippetEnd token) {
        writeToken(token);
        requestBlankLine();
    }
    void writeListOpen(ListOpenTag token) {
        if (classicJavadoc) {
            requestBlankLine();
        }
        writeToken(token);
        int indent = token.value().isEmpty() ? 0 : 2;
        indentStack.push(new ListIndent(indent));
        postWriteModifiedContinuingListStack.push();
        requestNewline();
    }
    void writeListClose(ListCloseTag token) {
        if (classicJavadoc) {
            requestNewline();
        }
        indentStack.popUntil(ListIndent.class);
        writeToken(token);
        postWriteModifiedContinuingListStack.popIfNotEmpty();
        if (classicJavadoc) {
            requestBlankLine();
        }
    }
    void writeListItemOpen(ListItemOpenTag token) {
        requestNewline();
        if (continuingListItemOfInnermostList) {
            continuingListItemOfInnermostList = false;
            indentStack.popUntil(ListItemIndent.class);
        }
        writeToken(token);
        int indent = token.value().length();
        indentStack.push(new ListItemIndent(indent));
    }
    void writeHeaderOpen(HeaderOpenTag token) {
        requestBlankLine();
        writeToken(token);
    }
    void writeHeaderClose(HeaderCloseTag token) {
        writeToken(token);
        requestBlankLine();
    }
    void writeParagraphOpen(Token token) {
        if (!wroteAnythingSignificant) {
            return;
        }
        requestBlankLine();
        writeToken(token);
    }
    void writeBlockQuoteOpen(BlockQuoteOpenTag token) {
        requestBlankLine();
        writeToken(token);
        requestNewline();
    }
    void writeBlockQuoteClose(BlockQuoteCloseTag token) {
        requestNewline();
        writeToken(token);
        requestBlankLine();
    }
    void writeMarkdownBlockQuoteOpen(MarkdownBlockQuoteOpen token) {
        if (!atStartOfLine) {
            requestNewline();
        }
        writeToken(new MarkdownBlockQuoteOpen("> "));
        indentStack.push(new BlockQuoteIndent());
    }
    void writeMarkdownBlockQuoteClose() {
        indentStack.popUntil(BlockQuoteIndent.class);
    }
    void writePreOpen(PreOpenTag token) {
        requestBlankLine();
        writeToken(token);
    }
    void writePreClose(PreCloseTag token) {
        writeToken(token);
        requestBlankLine();
    }
    void writeCodeOpen(CodeOpenTag token) {
        writeToken(token);
    }
    void writeCodeClose(CodeCloseTag token) {
        writeToken(token);
    }
    void writeTableOpen(TableOpenTag token) {
        requestBlankLine();
        writeToken(token);
    }
    void writeTableClose(TableCloseTag token) {
        writeToken(token);
        requestBlankLine();
    }
    void writeMoeEndStripComment(MoeEndStripComment token) {
        writeLineBreakNoAutoIndent();
        output.append(indentForMoeEndStripComment);
        writeToken(token);
        requestNewline();
    }
    void writeHtmlComment(HtmlComment token) {
        requestNewline();
        List<String> lines = token.value().lines().toList();
        writeToken(new HtmlComment(lines.get(0)));
        for (String line : lines.subList(1, lines.size())) {
            writeNewline(AutoIndent.NO_AUTO_INDENT);
            output.append(line);
            remainingOnLine -= line.length();
        }
        requestNewline();
    }
    void writeBr(BrTag token) {
        writeToken(token);
        requestNewline();
    }
    void writeLineBreakNoAutoIndent() {
        writeNewline(NO_AUTO_INDENT);
    }
    void writeMarkdownHardLineBreak() {
        writeLiteral(BACKSLASH_LITERAL);
        writeNewline();
    }
    void writeLiteral(Literal token) {
        writeToken(token);
    }
    void writeMarkdownFencedCodeBlock(MarkdownFencedCodeBlock token) {
        if (!atStartOfLine) {
            requestBlankLine();
        }
        flushWhitespace();
        output.append(token.start());
        token.literal().lines().forEach((line) -> {
            writeNewline();
            output.append(line);
        });
        writeNewline();
        output.append(token.end());
        requestBlankLine();
    }
    void writeMarkdownTable(MarkdownTable token) {
        if (!atStartOfLine) {
            requestBlankLine();
        }
        flushWhitespace();
        List<String> lines = token.value().lines().toList();
        output.append(lines.get(0));
        for (String line : lines.subList(1, lines.size())) {
            writeNewline(AutoIndent.NO_AUTO_INDENT);
            output.append(line);
        }
        requestBlankLine();
    }
    @Override
  public String toString() {
        return output.toString();
    }
    private void requestBlankLine() {
        requestWhitespace(BLANK_LINE);
    }
    private void requestNewline() {
        requestWhitespace(NEWLINE);
    }
    enum RequestedWhitespace {
        NONE, WHITESPACE, NEWLINE, BLANK_LINE
    }
    private void flushWhitespace() {
        if (requestedMoeBeginStripComment != null) {
            requestNewline();
        }
        if (!wroteAnythingSignificant) {
            requestedWhitespace = NONE;
        }
        if (classicJavadoc && requestedWhitespace == BLANK_LINE && (!postWriteModifiedContinuingListStack.isEmpty() || continuingFooterTag)) {
            requestedWhitespace = NEWLINE;
        }
        if (requestedWhitespace == BLANK_LINE) {
            writeBlankLine();
            requestedWhitespace = NONE;
        } else if (requestedWhitespace == NEWLINE) {
            writeNewline();
            requestedWhitespace = NONE;
        }
    }
    private void writeToken(Token token) {
        if (token.value().isEmpty()) {
            return;
        }
        flushWhitespace();
        boolean needWhitespace = requestedWhitespace == WHITESPACE;
        if (!atStartOfLine && token.length() + (needWhitespace ? 1 : 0) > remainingOnLine) {
            writeNewline();
        }
        if (!atStartOfLine && needWhitespace) {
            output.append(" ");
            remainingOnLine--;
        }
        if (requestedMoeBeginStripComment != null) {
            output.append(requestedMoeBeginStripComment.value());
            requestedMoeBeginStripComment = null;
            indentForMoeEndStripComment = innerIndentString();
            wroteAnythingSignificant = true;
            requestNewline();
            writeToken(token);
            return;
        }
        output.append(token.value());
        if (!(token instanceof StartOfLineToken)) {
            atStartOfLine = false;
        }
        remainingOnLine -= token.length();
        requestedWhitespace = NONE;
        wroteAnythingSignificant = true;
    }
    private void writeNewlineStart() {
        output.append("\n");
        appendSpaces(blockIndent + (classicJavadoc ? 1 : 0));
        output.append(classicJavadoc ? "*" : "///");
    }
    private void writeBlankLine() {
        writeNewlineStart();
        String indent = innerIndentString();
        if (!indent.isBlank()) {
            output.append(" ");
            output.append(indent.stripTrailing());
        }
        writeNewline();
    }
    private void writeNewline() {
        writeNewline(AUTO_INDENT);
    }
    private void writeNewline(AutoIndent autoIndent) {
        writeNewlineStart();
        appendSpaces(1);
        remainingOnLine = maxLineLength - blockIndent - (classicJavadoc ? 3 : 4);
        if (autoIndent == AUTO_INDENT) {
            String indent = innerIndentString();
            output.append(indent);
            remainingOnLine -= indent.length();
        }
        atStartOfLine = true;
    }
    enum AutoIndent {
        AUTO_INDENT, NO_AUTO_INDENT
    }
    private String innerIndentString() {
        StringBuilder sb = new StringBuilder();
        if (continuingFooterTag) {
            sb.repeat(' ', classicJavadoc ? 4 : 2);
        }
        for (Indent indent : indentStack.bottomToTop()) {
            indent.render(sb);
        }
        return sb.toString();
    }
    private sealed interface Indent {
        void render(StringBuilder sb);
    }
    private record ListIndent(int width) implements Indent {
        @Override
    public void render(StringBuilder sb) {
            sb.repeat(' ', width);
        }
    }
    private record ListItemIndent(int width) implements Indent {
        @Override
    public void render(StringBuilder sb) {
            sb.repeat(' ', width);
        }
    }
    private record BlockQuoteIndent() implements Indent {
        @Override
    public void render(StringBuilder sb) {
            sb.append("> ");
        }
    }
    private void appendSpaces(int count) {
        output.repeat(' ', count);
    }
}
