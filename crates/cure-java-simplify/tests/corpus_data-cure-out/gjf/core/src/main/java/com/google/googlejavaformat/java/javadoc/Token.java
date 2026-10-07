package com.google.googlejavaformat.java.javadoc;

sealed interface Token {
    String value();
    default int length() {
        return value().length();
    }
    interface StartOfLineToken {
    }
    record BeginJavadoc(String value) implements Token {
    }
    record EndJavadoc(String value) implements Token {
    }
    record FooterJavadocTagStart(String value) implements Token {
    }
    record SnippetBegin(String value) implements Token {
    }
    record SnippetEnd(String value) implements Token {
    }
    record ListOpenTag(String value) implements Token {
    }
    record ListCloseTag(String value) implements Token {
    }
    record ListItemOpenTag(String value) implements Token, StartOfLineToken {
    }
    record ListItemCloseTag(String value) implements Token {
    }
    record HeaderOpenTag(String value) implements Token, StartOfLineToken {
    }
    record HeaderCloseTag(String value) implements Token {
    }
    record ParagraphOpenTag(String value) implements Token, StartOfLineToken {
    }
    record ParagraphCloseTag(String value) implements Token {
    }
    record BlockQuoteOpenTag(String value) implements Token, StartOfLineToken {
    }
    record BlockQuoteCloseTag(String value) implements Token {
    }
    record MarkdownBlockQuoteOpen(String value) implements Token, StartOfLineToken {
    }
    record MarkdownBlockQuoteClose(String value) implements Token {
    }
    record PreOpenTag(String value) implements Token {
    }
    record PreCloseTag(String value) implements Token {
    }
    record CodeOpenTag(String value) implements Token {
    }
    record CodeCloseTag(String value) implements Token {
    }
    record TableOpenTag(String value) implements Token {
    }
    record TableCloseTag(String value) implements Token {
    }
    record MoeBeginStripComment(String value) implements Token {
    }
    record MoeEndStripComment(String value) implements Token {
    }
    record HtmlComment(String value) implements Token {
    }
    record BrTag(String value) implements Token {
    }
    record MarkdownCodeSpanStart(String value) implements Token {
    }
    record MarkdownCodeSpanEnd(String value) implements Token {
    }
    record MarkdownFencedCodeBlock(String value, String start, String end, String literal) implements Token {
    }
    record MarkdownTable(String value) implements Token {
    }
    record Whitespace(String value) implements Token {
    }
    record ForcedNewline(String value) implements Token {
    }
    record MarkdownHardLineBreak(String value) implements Token {
    }
    record OptionalLineBreak(String value) implements Token {
    }
    record BlockQuoteMarker(String value) implements Token {
    }
    record Literal(String value) implements Token {
    }
}
