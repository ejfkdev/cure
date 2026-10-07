package com.github.javaparser;

import com.github.javaparser.ast.CompilationUnit;
import com.github.javaparser.ast.ImportDeclaration;
import com.github.javaparser.ast.body.BodyDeclaration;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.stmt.BlockStmt;
import com.github.javaparser.ast.stmt.Statement;

@FunctionalInterface
public interface ParseStart<R> {
    ParseStart<CompilationUnit> COMPILATION_UNIT = ASTParser::CompilationUnit;
    ParseStart<BlockStmt> BLOCK = ASTParser::Block;
    ParseStart<Statement> STATEMENT = ASTParser::BlockStatement;
    ParseStart<ImportDeclaration> IMPORT_DECLARATION = ASTParser::ImportDeclaration;
    ParseStart<Expression> EXPRESSION = ASTParser::Expression;
    ParseStart<AnnotationExpr> ANNOTATION = ASTParser::Annotation;
    ParseStart<BodyDeclaration<?>> ANNOTATION_BODY = ASTParser::AnnotationBodyDeclaration;
    ParseStart<BodyDeclaration<?>> CLASS_BODY = (p) -> p.ClassOrInterfaceBodyDeclaration(false);
    ParseStart<BodyDeclaration<?>> INTERFACE_BODY = (p) -> p.ClassOrInterfaceBodyDeclaration(true);
    R parse(ASTParser parser) throws ParseException;
}
