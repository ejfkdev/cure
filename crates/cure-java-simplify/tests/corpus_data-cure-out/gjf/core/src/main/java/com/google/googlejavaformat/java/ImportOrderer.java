package com.google.googlejavaformat.java;

import static com.google.common.collect.Iterables.getLast;
import static com.google.common.primitives.Booleans.trueFirst;
import com.google.common.base.CharMatcher;
import com.google.common.base.Preconditions;
import com.google.common.base.Splitter;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableSet;
import com.google.common.collect.ImmutableSortedSet;
import com.google.googlejavaformat.Newlines;
import com.google.googlejavaformat.java.JavaFormatterOptions.Style;
import com.google.googlejavaformat.java.JavaInput.Tok;
import com.sun.tools.javac.parser.Tokens.TokenKind;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Optional;
import java.util.function.BiFunction;
import java.util.stream.Stream;

public class ImportOrderer {
    private static final Splitter DOT_SPLITTER = Splitter.on('.');
    public static String reorderImports(String text, Style style) throws FormatterException {
        return new ImportOrderer(text, JavaInput.buildToks(text, CLASS_START), style).reorderImports();
    }
    @Deprecated
  public static String reorderImports(String text) throws FormatterException {
        return reorderImports(text, Style.GOOGLE);
    }
    private String reorderImports() throws FormatterException {
        Optional<Integer> maybeFirstImport = findIdentifier(0, IMPORT_OR_CLASS_START);
        if (!maybeFirstImport.isPresent() || !tokenAt(maybeFirstImport.get()).equals("import")) {
            return text;
        }
        int firstImportStart = maybeFirstImport.get();
        int unindentedFirstImportStart = unindent(firstImportStart);
        ImportsAndIndex imports = scanImports(firstImportStart);
        int afterLastImport = imports.index;
        Optional<Integer> maybeLaterImport = findIdentifier(afterLastImport, IMPORT_OR_CLASS_START);
        if (maybeLaterImport.isPresent() && tokenAt(maybeLaterImport.get()).equals("import")) {
            throw new FormatterException("Imports not contiguous (perhaps a comment separates them?)");
        }
        StringBuilder result = new StringBuilder();
        String prefix = tokString(0, unindentedFirstImportStart);
        result.append(prefix);
        if (!prefix.isEmpty() && Newlines.getLineEnding(prefix) == null) {
            result.append(lineSeparator).append(lineSeparator);
        }
        result.append(reorderedImportsString(imports.imports));
        List<String> tail = new ArrayList<>();
        tail.add(CharMatcher.whitespace().trimLeadingFrom(tokString(afterLastImport, toks.size())));
        if (!toks.isEmpty()) {
            Tok lastTok = getLast(toks);
            int tailStart = lastTok.getPosition() + lastTok.length();
            tail.add(text.substring(tailStart));
        }
        if (tail.stream().anyMatch((s) -> !s.isEmpty())) {
            result.append(lineSeparator);
            tail.forEach(result::append);
        }
        return result.toString();
    }
    private static final ImmutableSet<TokenKind> CLASS_START = ImmutableSet.of(TokenKind.CLASS, TokenKind.INTERFACE, TokenKind.ENUM);
    private static final ImmutableSet<String> IMPORT_OR_CLASS_START = ImmutableSet.of("import", "class", "interface", "enum");
    private static final Comparator<Import> GOOGLE_IMPORT_COMPARATOR = Comparator.comparing(Import::importType).thenComparing(Import::imported);
    private static final Comparator<Import> AOSP_IMPORT_COMPARATOR = Comparator.comparing(Import::importType).thenComparing(Import::isAndroid, trueFirst()).thenComparing(Import::isThirdParty, trueFirst()).thenComparing(Import::isJava, trueFirst()).thenComparing(Import::imported);
    private static boolean shouldInsertBlankLineGoogle(Import prev, Import curr) {
        return !prev.importType().equals(curr.importType());
    }
    private static boolean shouldInsertBlankLineAosp(Import prev, Import curr) {
        return !prev.importType().equals(curr.importType()) || (prev.isAndroid() && !curr.isAndroid() || !prev.topLevel().equals(curr.topLevel()));
    }
    private final String text;
    private final ImmutableList<Tok> toks;
    private final String lineSeparator;
    private final Comparator<Import> importComparator;
    private final BiFunction<Import, Import, Boolean> shouldInsertBlankLineFn;
    private ImportOrderer(String text, ImmutableList<Tok> toks, Style style) {
        this.text = text;
        this.toks = toks;
        this.lineSeparator = Newlines.guessLineSeparator(text);
        switch (style.importOrder()) {
            case GOOGLE -> {
                this.importComparator = GOOGLE_IMPORT_COMPARATOR;
                this.shouldInsertBlankLineFn = ImportOrderer::shouldInsertBlankLineGoogle;
            }
            case AOSP -> {
                this.importComparator = AOSP_IMPORT_COMPARATOR;
                this.shouldInsertBlankLineFn = ImportOrderer::shouldInsertBlankLineAosp;
            }
            default -> throw new IllegalArgumentException("Unsupported code style: " + style);
        }
    }
    private enum ImportType {
        STATIC, MODULE, NORMAL
    }
    private record Import(
      String imported, String trailing, ImportType importType, String lineSeparator) {
        String topLevel() {
            return DOT_SPLITTER.split(imported).iterator().next();
        }
        boolean isAndroid() {
            return Stream.of("android.", "androidx.", "dalvik.", "libcore.", "com.android.").anyMatch(imported::startsWith);
        }
        boolean isJava() {
            return switch (topLevel()) {
                case "java", "javax" -> true;
                default -> false;
            };
        }
        boolean isThirdParty() {
            return !(isAndroid() || isJava());
        }
        @Override
    public String toString() {
            StringBuilder sb = new StringBuilder();
            sb.append("import ");
            switch (importType) {
                case STATIC -> sb.append("static ");
                case MODULE -> sb.append("module ");
                case NORMAL -> {}
            }
            sb.append(imported()).append(';');
            if (trailing().trim().isEmpty()) {
                sb.append(lineSeparator);
            } else {
                sb.append(trailing());
            }
            return sb.toString();
        }
    }
    private String tokString(int start, int end) {
        StringBuilder sb = new StringBuilder();
        for (int i = start; i < end; i++) {
            sb.append(toks.get(i).getOriginalText());
        }
        return sb.toString();
    }
    private record ImportsAndIndex(ImmutableSortedSet<Import> imports, int index) {
    }
    private ImportsAndIndex scanImports(int i) throws FormatterException {
        int afterLastImport = i;
        ImmutableSortedSet.Builder<Import> imports = ImmutableSortedSet.orderedBy(importComparator);
        while (i < toks.size() && tokenAt(i).equals("import")) {
            i++;
            if (isSpaceToken(i)) {
                i++;
            }
            ImportType importType = switch (tokenAt(i)) {
                case "static" -> ImportType.STATIC;
                case "module" -> ImportType.MODULE;
                default -> ImportType.NORMAL;
            };
            if (!importType.equals(ImportType.NORMAL)) {
                i++;
                if (isSpaceToken(i)) {
                    i++;
                }
            }
            if (!isIdentifierToken(i)) {
                throw new FormatterException("Unexpected token after import: " + tokenAt(i));
            }
            StringAndIndex imported = scanImported(i);
            String importedName = imported.string;
            i = imported.index;
            if (isSpaceToken(i)) {
                i++;
            }
            if (!tokenAt(i).equals(";")) {
                throw new FormatterException("Expected ; after import");
            }
            while (tokenAt(i).equals(";")) {
                i++;
            }
            StringBuilder trailing = new StringBuilder();
            if (isSpaceToken(i)) {
                trailing.append(tokenAt(i));
                i++;
            }
            if (isNewlineToken(i)) {
                trailing.append(tokenAt(i));
                i++;
            }
            while (isSlashSlashCommentToken(i)) {
                trailing.append(tokenAt(i));
                i++;
                if (isNewlineToken(i)) {
                    trailing.append(tokenAt(i));
                    i++;
                }
            }
            while (tokenAt(i).equals(";")) {
                i++;
            }
            imports.add(new Import(importedName, trailing.toString(), importType, lineSeparator));
            afterLastImport = i;
            while (isNewlineToken(i) || isSpaceToken(i)) {
                i++;
            }
        }
        return new ImportsAndIndex(imports.build(), afterLastImport);
    }
    private String reorderedImportsString(ImmutableSortedSet<Import> imports) {
        Preconditions.checkArgument(!imports.isEmpty(), "imports");
        Import prevImport = imports.iterator().next();
        StringBuilder sb = new StringBuilder();
        for (Import currImport : imports) {
            if (shouldInsertBlankLineFn.apply(prevImport, currImport)) {
                sb.append(lineSeparator);
            }
            sb.append(currImport);
            prevImport = currImport;
        }
        return sb.toString();
    }
    private static class StringAndIndex {
        private final String string;
        private final int index;
        StringAndIndex(String string, int index) {
            this.string = string;
            this.index = index;
        }
    }
    private StringAndIndex scanImported(int start) throws FormatterException {
        int i = start;
        StringBuilder imported = new StringBuilder();
        while (true) {
            Preconditions.checkState(isIdentifierToken(i));
            imported.append(tokenAt(i));
            i++;
            if (!tokenAt(i).equals(".")) {
                return new StringAndIndex(imported.toString(), i);
            }
            imported.append('.');
            i++;
            if (tokenAt(i).equals("*")) {
                imported.append('*');
                return new StringAndIndex(imported.toString(), i + 1);
            } else if (!isIdentifierToken(i)) {
                throw new FormatterException("Could not parse imported name, at: " + tokenAt(i));
            }
        }
    }
    private Optional<Integer> findIdentifier(int start, ImmutableSet<String> identifiers) {
        for (int i = start; i < toks.size(); i++) {
            if (isIdentifierToken(i)) {
                String id = tokenAt(i);
                if (identifiers.contains(id)) {
                    return Optional.of(i);
                }
            }
        }
        return Optional.empty();
    }
    private int unindent(int i) {
        return i > 0 && isSpaceToken(i - 1) ? i - 1 : i;
    }
    private String tokenAt(int i) {
        return toks.get(i).getOriginalText();
    }
    private boolean isIdentifierToken(int i) {
        String s = tokenAt(i);
        return !s.isEmpty() && Character.isJavaIdentifierStart(s.codePointAt(0));
    }
    private boolean isSpaceToken(int i) {
        String s = tokenAt(i);
        return s.isEmpty() ? false : " \t\u000C".indexOf(s.codePointAt(0)) >= 0;
    }
    private boolean isSlashSlashCommentToken(int i) {
        return toks.get(i).isSlashSlashComment();
    }
    private boolean isNewlineToken(int i) {
        return toks.get(i).isNewline();
    }
}
