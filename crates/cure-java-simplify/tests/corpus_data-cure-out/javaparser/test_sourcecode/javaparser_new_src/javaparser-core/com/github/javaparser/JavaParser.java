package com.github.javaparser;

import com.github.javaparser.ast.CompilationUnit;
import com.github.javaparser.ast.ImportDeclaration;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.body.BodyDeclaration;
import com.github.javaparser.ast.comments.CommentsCollection;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.stmt.BlockStmt;
import com.github.javaparser.ast.stmt.Statement;
import java.io.*;
import java.nio.charset.Charset;
import java.nio.file.Path;
import java.util.Optional;
import static com.github.javaparser.ParseStart.*;
import static com.github.javaparser.Providers.UTF8;
import static com.github.javaparser.Providers.provider;

public final class JavaParser {
    private final CommentsInserter commentsInserter;
    private ASTParser astParser = null;
    public JavaParser() {
        this(new ParserConfiguration());
    }
    public JavaParser(ParserConfiguration configuration) {
        commentsInserter = new CommentsInserter(configuration);
    }
    private ASTParser getParserForProvider(Provider provider) {
        if (astParser == null) {
            astParser = new ASTParser(provider);
        } else {
            astParser.ReInit(provider);
        }
        return astParser;
    }
    public <N extends Node> ParseResult<N> parse(ParseStart<N> start, Provider provider) {
        try {
            ASTParser parser = getParserForProvider(provider);
            N resultNode = start.parse(parser);
            CommentsCollection comments = astParser.getCommentsCollection();
            commentsInserter.insertComments(resultNode, comments.copy().getComments());
            return new ParseResult<>(Optional.of(resultNode), parser.problems, Optional.of(astParser.getTokens()), Optional.of(astParser.getCommentsCollection()));
        } catch (ParseException e) {
            return new ParseResult<>(e);
        } catch (TokenMgrException e) {
            return new ParseResult<>(e);
        } finally {
            try {
                provider.close();
            } catch (IOException e) {}
        }
    }
    public static CompilationUnit parse(final InputStream in, Charset encoding) {
        return simplifiedParse(COMPILATION_UNIT, provider(in, encoding));
    }
    public static CompilationUnit parse(final InputStream in) {
        return parse(in, UTF8);
    }
    public static CompilationUnit parse(final File file, final Charset encoding) throws FileNotFoundException {
        return simplifiedParse(COMPILATION_UNIT, provider(file, encoding));
    }
    public static CompilationUnit parse(final File file) throws FileNotFoundException {
        return simplifiedParse(COMPILATION_UNIT, provider(file));
    }
    public static CompilationUnit parse(final Path path, final Charset encoding) throws IOException {
        return simplifiedParse(COMPILATION_UNIT, provider(path, encoding));
    }
    public static CompilationUnit parse(final Path path) throws IOException {
        return simplifiedParse(COMPILATION_UNIT, provider(path));
    }
    public static CompilationUnit parse(final Reader reader) {
        return simplifiedParse(COMPILATION_UNIT, provider(reader));
    }
    public static CompilationUnit parse(String code) {
        return simplifiedParse(COMPILATION_UNIT, provider(code));
    }
    public static BlockStmt parseBlock(final String blockStatement) {
        return simplifiedParse(BLOCK, provider(blockStatement));
    }
    public static Statement parseStatement(final String statement) {
        return simplifiedParse(STATEMENT, provider(statement));
    }
    private static <T extends Node> T simplifiedParse(ParseStart<T> context, Provider provider) {
        ParseResult<T> result = new JavaParser(new ParserConfiguration()).parse(context, provider);
        if (result.isSuccessful()) {
            return result.getResult().get();
        }
        throw new ParseProblemException(result.getProblems());
    }
    public static ImportDeclaration parseImport(final String importDeclaration) {
        return simplifiedParse(IMPORT_DECLARATION, provider(importDeclaration));
    }
    public static Expression parseExpression(final String expression) {
        return simplifiedParse(EXPRESSION, provider(expression));
    }
    public static AnnotationExpr parseAnnotation(final String annotation) {
        return simplifiedParse(ANNOTATION, provider(annotation));
    }
    public static BodyDeclaration<?> parseAnnotationBodyDeclaration(final String body) {
        return simplifiedParse(ANNOTATION_BODY, provider(body));
    }
    public static BodyDeclaration<?> parseClassBodyDeclaration(String body) {
        return simplifiedParse(CLASS_BODY, provider(body));
    }
    public static BodyDeclaration parseInterfaceBodyDeclaration(String body) {
        return simplifiedParse(INTERFACE_BODY, provider(body));
    }
}
