package com.google.googlejavaformat.java;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.collect.Iterables.getLast;
import static java.nio.charset.StandardCharsets.UTF_8;
import com.google.common.base.MoreObjects;
import com.google.common.base.Verify;
import com.google.common.collect.DiscreteDomain;
import com.google.common.collect.ImmutableCollection;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.common.collect.ImmutableRangeMap;
import com.google.common.collect.ImmutableSet;
import com.google.common.collect.Iterators;
import com.google.common.collect.Range;
import com.google.common.collect.RangeSet;
import com.google.common.collect.TreeRangeSet;
import com.google.googlejavaformat.Input;
import com.google.googlejavaformat.Newlines;
import com.google.googlejavaformat.java.JavacTokens.RawTok;
import com.sun.tools.javac.file.JavacFileManager;
import com.sun.tools.javac.parser.Tokens.TokenKind;
import com.sun.tools.javac.tree.JCTree.JCCompilationUnit;
import com.sun.tools.javac.util.Context;
import com.sun.tools.javac.util.JCDiagnostic;
import com.sun.tools.javac.util.Log;
import com.sun.tools.javac.util.Log.DeferredDiagnosticHandler;
import com.sun.tools.javac.util.Options;
import java.io.IOException;
import java.lang.reflect.Constructor;
import java.lang.reflect.Method;
import java.net.URI;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Iterator;
import java.util.List;
import javax.tools.Diagnostic;
import javax.tools.DiagnosticCollector;
import javax.tools.DiagnosticListener;
import javax.tools.JavaFileManager;
import javax.tools.JavaFileObject;
import javax.tools.JavaFileObject.Kind;
import javax.tools.SimpleJavaFileObject;
import org.jspecify.annotations.Nullable;

final class JavaInput extends Input {
    static final class Tok implements Input.Tok {
        private final int index;
        private final String originalText;
        private final String text;
        private final int position;
        private final int columnI;
        private final boolean isToken;
        private final TokenKind kind;
        Tok(int index, String originalText, String text, int position, int columnI, boolean isToken, TokenKind kind) {
            this.index = index;
            this.originalText = originalText;
            this.text = text;
            this.position = position;
            this.columnI = columnI;
            this.isToken = isToken;
            this.kind = kind;
        }
        @Override
    public int getIndex() {
            return index;
        }
        @Override
    public String getText() {
            return text;
        }
        @Override
    public String getOriginalText() {
            return originalText;
        }
        @Override
    public int length() {
            return originalText.length();
        }
        @Override
    public int getPosition() {
            return position;
        }
        @Override
    public int getColumn() {
            return columnI;
        }
        boolean isToken() {
            return isToken;
        }
        @Override
    public boolean isNewline() {
            return Newlines.isNewline(text);
        }
        @Override
    public boolean isSlashSlashComment() {
            return text.startsWith("//");
        }
        @Override
    public boolean isSlashStarComment() {
            return text.startsWith("/*");
        }
        @Override
    public boolean isJavadocComment() {
            return (text.startsWith("/**") && !text.startsWith("/***") || Runtime.version().feature() >= 23 && text.startsWith("///") && !text.startsWith("////")) && text.length() > 4;
        }
        @Override
    public boolean isComment() {
            return isSlashSlashComment() || isSlashStarComment();
        }
        @Override
    public String toString() {
            return MoreObjects.toStringHelper(this).add("index", index).add("text", text).add("position", position).add("columnI", columnI).add("isToken", isToken).toString();
        }
        public TokenKind kind() {
            return kind;
        }
    }
    static final class Token implements Input.Token {
        private final Tok tok;
        private final ImmutableList<Tok> toksBefore;
        private final ImmutableList<Tok> toksAfter;
        Token(List<Tok> toksBefore, Tok tok, List<Tok> toksAfter) {
            this.toksBefore = ImmutableList.copyOf(toksBefore);
            this.tok = tok;
            this.toksAfter = ImmutableList.copyOf(toksAfter);
        }
        @Override
    public Tok getTok() {
            return tok;
        }
        @Override
    public ImmutableList<? extends Input.Tok> getToksBefore() {
            return toksBefore;
        }
        @Override
    public ImmutableList<? extends Input.Tok> getToksAfter() {
            return toksAfter;
        }
        @Override
    public String toString() {
            return MoreObjects.toStringHelper(this).add("tok", tok).add("toksBefore", toksBefore).add("toksAfter", toksAfter).toString();
        }
    }
    private final String text;
    private int kN;
    private final ImmutableMap<Integer, Integer> positionToColumnMap;
    private final ImmutableList<Token> tokens;
    private final ImmutableRangeMap<Integer, Token> positionTokenMap;
    private final Token[] kToToken;
    JavaInput(String text) throws FormatterException {
        this.text = checkNotNull(text);
        setLines(ImmutableList.copyOf(Newlines.lineIterator(text)));
        ImmutableList<Tok> toks = buildToks(text);
        positionToColumnMap = makePositionToColumnMap(toks);
        tokens = buildTokens(toks);
        ImmutableRangeMap.Builder<Integer, Token> tokenLocations = ImmutableRangeMap.builder();
        for (Token token : tokens) {
            Input.Tok end = JavaOutput.endTok(token);
            int upper = end.getPosition();
            if (!end.getText().isEmpty()) {
                upper += end.length() - 1;
            }
            tokenLocations.put(Range.closed(JavaOutput.startTok(token).getPosition(), upper), token);
        }
        positionTokenMap = tokenLocations.build();
        kToToken = new Token[kN + 1];
        for (Token token : tokens) {
            for (Input.Tok tok : token.getToksBefore()) {
                if (tok.getIndex() < 0) {
                    continue;
                }
                kToToken[tok.getIndex()] = token;
            }
            kToToken[token.getTok().getIndex()] = token;
            for (Input.Tok tok : token.getToksAfter()) {
                if (tok.getIndex() < 0) {
                    continue;
                }
                kToToken[tok.getIndex()] = token;
            }
        }
    }
    private static ImmutableMap<Integer, Integer> makePositionToColumnMap(List<Tok> toks) {
        ImmutableMap.Builder<Integer, Integer> builder = ImmutableMap.builder();
        for (Tok tok : toks) {
            builder.put(tok.getPosition(), tok.getColumn());
        }
        return builder.buildOrThrow();
    }
    @Override
  public String getText() {
        return text;
    }
    @Override
  public ImmutableMap<Integer, Integer> getPositionToColumnMap() {
        return positionToColumnMap;
    }
    private ImmutableList<Tok> buildToks(String text) throws FormatterException {
        ImmutableList<Tok> toks = buildToks(text, ImmutableSet.of());
        kN = getLast(toks).getIndex();
        computeRanges(toks);
        return toks;
    }
    static ImmutableList<Tok> buildToks(String text, ImmutableSet<TokenKind> stopTokens) throws FormatterException {
        stopTokens = ImmutableSet.builder().addAll(stopTokens).add(TokenKind.EOF).build();
        Context context = new Context();
        Options.instance(context).put("--enable-preview", "true");
        JavaFileManager fileManager = new JavacFileManager(context, false, UTF_8);
        context.put(JavaFileManager.class, fileManager);
        DiagnosticCollector<JavaFileObject> diagnosticCollector = new DiagnosticCollector<>();
        context.put(DiagnosticListener.class, diagnosticCollector);
        Log log = Log.instance(context);
        log.useSource(new SimpleJavaFileObject(URI.create("Source.java"), Kind.SOURCE) {
          @Override
          public CharSequence getCharContent(boolean ignoreEncodingErrors) throws IOException {
            return text;
          }
        });
        DeferredDiagnosticHandler diagnostics = deferredDiagnosticHandler(log);
        ImmutableList<RawTok> rawToks = JavacTokens.getTokens(text, context, stopTokens);
        Collection<JCDiagnostic> ds;
        try {
            @SuppressWarnings("unchecked")
                  var extraLocalForSuppression = (Collection<JCDiagnostic>) GET_DIAGNOSTICS.invoke(diagnostics);
            ds = extraLocalForSuppression;
        } catch (ReflectiveOperationException e) {
            throw new LinkageError(e.getMessage(), e);
        }
        if (ds.stream().anyMatch((d) -> d.getKind() == Diagnostic.Kind.ERROR)) {
            return ImmutableList.of(new Tok(0, "", "", 0, 0, true, null));
        }
        int kN = 0;
        List<Tok> toks = new ArrayList<>();
        int charI = 0;
        int columnI = 0;
        for (RawTok t : rawToks) {
            if (stopTokens.contains(t.kind())) {
                break;
            }
            int charI0 = t.pos();
            String originalTokText = text.substring(charI0, t.endPos());
            String tokText = t.kind() == TokenKind.STRINGLITERAL ? t.stringVal() : originalTokText;
            char tokText0 = tokText.charAt(0);
            boolean isToken;
            boolean isNumbered;
            String extraNewline = null;
            List<String> strings = new ArrayList<>();
            if (Character.isWhitespace(tokText0)) {
                isToken = false;
                isNumbered = false;
                Iterator<String> it = Newlines.lineIterator(originalTokText);
                while (it.hasNext()) {
                    String line = it.next();
                    String newline = Newlines.getLineEnding(line);
                    if (newline != null) {
                        String spaces = line.substring(0, line.length() - newline.length());
                        if (!spaces.isEmpty()) {
                            strings.add(spaces);
                        }
                        strings.add(newline);
                    } else if (!line.isEmpty()) {
                        strings.add(line);
                    }
                }
            } else if (tokText.startsWith("'") || tokText.startsWith("\"")) {
                isToken = true;
                isNumbered = true;
                strings.add(originalTokText);
            } else if (tokText.startsWith("//") || tokText.startsWith("/*")) {
                if (tokText.startsWith("//") && (originalTokText.endsWith("\n") || originalTokText.endsWith("\r"))) {
                    extraNewline = Newlines.getLineEnding(originalTokText);
                    tokText = tokText.substring(0, tokText.length() - extraNewline.length());
                    originalTokText = originalTokText.substring(0, originalTokText.length() - extraNewline.length());
                }
                isToken = false;
                isNumbered = true;
                strings.add(originalTokText);
            } else if (Character.isJavaIdentifierStart(tokText0) || Character.isDigit(tokText0) || tokText0 == '.' && tokText.length() > 1 && Character.isDigit(tokText.charAt(1))) {
                isToken = true;
                isNumbered = true;
                strings.add(tokText);
            } else {
                isToken = true;
                isNumbered = true;
                for (char c : tokText.toCharArray()) {
                    strings.add(String.valueOf(c));
                }
            }
            if (strings.size() == 1) {
                toks.add(new Tok(isNumbered ? kN++ : -1, originalTokText, tokText, charI, columnI, isToken, t.kind()));
                charI += originalTokText.length();
                columnI = updateColumn(columnI, originalTokText);
            } else {
                if (strings.size() != 1 && !tokText.equals(originalTokText)) {
                    throw new FormatterException("Unicode escapes not allowed in whitespace or multi-character operators");
                }
                for (String str : strings) {
                    toks.add(new Tok(isNumbered ? kN++ : -1, str, str, charI, columnI, isToken, null));
                    charI += str.length();
                    columnI = updateColumn(columnI, originalTokText);
                }
            }
            if (extraNewline != null) {
                toks.add(new Tok(-1, extraNewline, extraNewline, charI, columnI, false, null));
                columnI = 0;
                charI += extraNewline.length();
            }
        }
        toks.add(new Tok(kN, "", "", charI, columnI, true, null));
        return ImmutableList.copyOf(toks);
    }
    private static final Constructor<DeferredDiagnosticHandler> DEFERRED_DIAGNOSTIC_HANDLER_CONSTRUCTOR = getDeferredDiagnosticHandlerConstructor();
    private static Constructor<DeferredDiagnosticHandler> getDeferredDiagnosticHandlerConstructor() {
        try {
            return DeferredDiagnosticHandler.class.getConstructor(Log.class);
        } catch (NoSuchMethodException e) {
            throw new LinkageError(e.getMessage(), e);
        }
    }
    private static DeferredDiagnosticHandler deferredDiagnosticHandler(Log log) {
        try {
            return DEFERRED_DIAGNOSTIC_HANDLER_CONSTRUCTOR.newInstance(log);
        } catch (ReflectiveOperationException e) {
            throw new LinkageError(e.getMessage(), e);
        }
    }
    private static final Method GET_DIAGNOSTICS = getGetDiagnostics();
    private static @Nullable Method getGetDiagnostics() {
        try {
            return DeferredDiagnosticHandler.class.getMethod("getDiagnostics");
        } catch (NoSuchMethodException e) {
            throw new LinkageError(e.getMessage(), e);
        }
    }
    private static int updateColumn(int columnI, String originalTokText) {
        Integer last = Iterators.getLast(Newlines.lineOffsetIterator(originalTokText));
        if (last > 0) {
            columnI = originalTokText.length() - last;
        } else {
            columnI += originalTokText.length();
        }
        return columnI;
    }
    private static ImmutableList<Token> buildTokens(List<Tok> toks) {
        ImmutableList.Builder<Token> tokens = ImmutableList.builder();
        int k = 0;
        int kN = toks.size();
        ImmutableList.Builder<Tok> toksBefore = ImmutableList.builder();
        OUTERMOST:
            while (k < kN) {
                while (!toks.get(k).isToken()) {
                    Tok tok = toks.get(k++);
                    toksBefore.add(tok);
                    if (isParamComment(tok)) {
                        while (toks.get(k).isNewline()) {
                            k++;
                        }
                    }
                }
                Tok tok = toks.get(k++);
                ImmutableList.Builder<Tok> toksAfter = ImmutableList.builder();
                OUTER:
                    while (k < kN && !toks.get(k).isToken()) {
                        if (toks.get(k).isSlashStarComment()) {
                            switch (tok.getText()) {
                                case "(", "<", "." -> {
                                    break OUTER;
                                }
                                default -> {}
                            }
                        }
                        if (toks.get(k).isJavadocComment()) {
                            switch (tok.getText()) {
                                case ";" -> {
                                    break OUTER;
                                }
                                default -> {}
                            }
                        }
                        if (isParamComment(toks.get(k))) {
                            tokens.add(new Token(toksBefore.build(), tok, toksAfter.build()));
                            toksBefore = ImmutableList.builder().add(toks.get(k++));
                            while (toks.get(k).isNewline()) {
                                k++;
                            }
                            continue OUTERMOST;
                        }
                        Tok nonTokenAfter = toks.get(k++);
                        toksAfter.add(nonTokenAfter);
                        if (Newlines.containsBreaks(nonTokenAfter.getText())) {
                            break;
                        }
                    }
                tokens.add(new Token(toksBefore.build(), tok, toksAfter.build()));
                toksBefore = ImmutableList.builder();
            }
        return tokens.build();
    }
    private static boolean isParamComment(Tok tok) {
        return tok.isSlashStarComment() && tok.getText().matches("\\/\\*[A-Za-z0-9\\s_\\-]+=\\s*\\*\\/");
    }
    private Range<Integer> characterRangeToTokenRange(Range<Integer> characterRange) throws FormatterException {
        if (characterRange.upperEndpoint() > text.length()) {
            throw new FormatterException(String.format("error: invalid offset (%d) or length (%d); offset + length (%d) > file length (%d)", characterRange.lowerEndpoint(), characterRange.upperEndpoint() - characterRange.lowerEndpoint(), characterRange.upperEndpoint(), text.length()));
        }
        Range<Integer> nonEmptyRange = characterRange.isEmpty() ? Range.closedOpen(characterRange.lowerEndpoint(), characterRange.lowerEndpoint() + 1) : characterRange;
        ImmutableCollection<Token> enclosed = getPositionTokenMap().subRangeMap(nonEmptyRange).asMapOfRanges().values();
        return enclosed.isEmpty() ? EMPTY_RANGE : Range.closedOpen(enclosed.iterator().next().getTok().getIndex(), getLast(enclosed).getTok().getIndex() + 1);
    }
    @Override
  public int getkN() {
        return kN;
    }
    @Override
  public Token getToken(int k) {
        return kToToken[k];
    }
    @Override
  public ImmutableList<? extends Input.Token> getTokens() {
        return tokens;
    }
    @Override
  public ImmutableRangeMap<Integer, Token> getPositionTokenMap() {
        return positionTokenMap;
    }
    @Override
  public String toString() {
        return MoreObjects.toStringHelper(this).add("tokens", tokens).add("super", super.toString()).toString();
    }
    private JCCompilationUnit unit;
    @Override
  public int getLineNumber(int inputPosition) {
        Verify.verifyNotNull(unit, "Expected compilation unit to be set.");
        return unit.getLineMap().getLineNumber(inputPosition);
    }
    @Override
  public int getColumnNumber(int inputPosition) {
        Verify.verifyNotNull(unit, "Expected compilation unit to be set.");
        return unit.getLineMap().getColumnNumber(inputPosition);
    }
    void setCompilationUnit(JCCompilationUnit unit) {
        this.unit = unit;
    }
    RangeSet<Integer> characterRangesToTokenRanges(Collection<Range<Integer>> characterRanges) throws FormatterException {
        RangeSet<Integer> tokenRangeSet = TreeRangeSet.create();
        for (Range<Integer> characterRange : characterRanges) {
            tokenRangeSet.add(characterRangeToTokenRange(characterRange.canonical(DiscreteDomain.integers())));
        }
        return tokenRangeSet;
    }
}
