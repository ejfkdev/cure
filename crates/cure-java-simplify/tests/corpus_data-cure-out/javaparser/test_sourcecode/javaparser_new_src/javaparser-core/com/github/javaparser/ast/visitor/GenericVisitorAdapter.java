package com.github.javaparser.ast.visitor;

import com.github.javaparser.ast.*;
import com.github.javaparser.ast.body.*;
import com.github.javaparser.ast.comments.BlockComment;
import com.github.javaparser.ast.comments.Comment;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.comments.LineComment;
import com.github.javaparser.ast.expr.*;
import com.github.javaparser.ast.stmt.*;
import com.github.javaparser.ast.type.*;
import java.util.List;

public abstract class GenericVisitorAdapter<R, A> implements GenericVisitor<R, A> {
    @Override
	public R visit(final AnnotationDeclaration n, final A arg) {
        visitComment(n, arg);
        if (n.getAnnotations() != null) {
            for (AnnotationExpr a : n.getAnnotations()) {
                {
                    R result = a.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getMembers() != null) {
            for (BodyDeclaration<?> member : n.getMembers()) {
                {
                    R result = member.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final AnnotationMemberDeclaration n, final A arg) {
        visitComment(n, arg);
        if (n.getAnnotations() != null) {
            for (AnnotationExpr a : n.getAnnotations()) {
                {
                    R result = a.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        {
            R result = n.getType().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        if (n.getDefaultValue() != null) {
            {
                R result = n.getDefaultValue().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final ArrayAccessExpr n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getName().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getIndex().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final ArrayCreationExpr n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getType().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        for (ArrayCreationLevel level : n.getLevels()) {
            R result = level.accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        if (n.getInitializer() != null) {
            R result = n.getInitializer().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final ArrayInitializerExpr n, final A arg) {
        visitComment(n, arg);
        if (n.getValues() != null) {
            for (Expression expr : n.getValues()) {
                {
                    R result = expr.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final AssertStmt n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getCheck().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        if (n.getMessage() != null) {
            {
                R result = n.getMessage().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final AssignExpr n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getTarget().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getValue().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final BinaryExpr n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getLeft().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getRight().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final BlockStmt n, final A arg) {
        visitComment(n, arg);
        if (n.getStmts() != null) {
            for (Statement s : n.getStmts()) {
                {
                    R result = s.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final BooleanLiteralExpr n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final BreakStmt n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final CastExpr n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getType().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getExpr().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final CatchClause n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getParam().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getBody().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final CharLiteralExpr n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final ClassExpr n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getType().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final ClassOrInterfaceDeclaration n, final A arg) {
        visitComment(n, arg);
        if (n.getAnnotations() != null) {
            for (AnnotationExpr a : n.getAnnotations()) {
                {
                    R result = a.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getTypeParameters() != null) {
            for (TypeParameter t : n.getTypeParameters()) {
                {
                    R result = t.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getExtends() != null) {
            for (ClassOrInterfaceType c : n.getExtends()) {
                {
                    R result = c.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getImplements() != null) {
            for (ClassOrInterfaceType c : n.getImplements()) {
                {
                    R result = c.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getMembers() != null) {
            for (BodyDeclaration<?> member : n.getMembers()) {
                {
                    R result = member.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final ClassOrInterfaceType n, final A arg) {
        visitComment(n, arg);
        for (AnnotationExpr a : n.getAnnotations()) {
            R result = a.accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        if (n.getScope() != null) {
            {
                R result = n.getScope().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        if (n.getTypeArguments() != null) {
            for (Type<?> type : n.getTypeArguments()) {
                R result = type.accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final CompilationUnit n, final A arg) {
        visitComment(n, arg);
        if (n.getPackage() != null) {
            {
                R result = n.getPackage().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        if (n.getImports() != null) {
            for (ImportDeclaration i : n.getImports()) {
                {
                    R result = i.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getTypes() != null) {
            for (TypeDeclaration<?> typeDeclaration : n.getTypes()) {
                {
                    R result = typeDeclaration.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final ConditionalExpr n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getCondition().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getThenExpr().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getElseExpr().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final ConstructorDeclaration n, final A arg) {
        visitComment(n, arg);
        if (n.getAnnotations() != null) {
            for (AnnotationExpr a : n.getAnnotations()) {
                {
                    R result = a.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getTypeParameters() != null) {
            for (TypeParameter t : n.getTypeParameters()) {
                {
                    R result = t.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getParameters() != null) {
            for (Parameter p : n.getParameters()) {
                {
                    R result = p.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getThrows() != null) {
            for (ReferenceType name : n.getThrows()) {
                {
                    R result = name.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        {
            R result = n.getBody().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final ContinueStmt n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final DoStmt n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getBody().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getCondition().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final DoubleLiteralExpr n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final EmptyMemberDeclaration n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final EmptyStmt n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final EmptyTypeDeclaration n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final EnclosedExpr n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getInner().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final EnumConstantDeclaration n, final A arg) {
        visitComment(n, arg);
        if (n.getAnnotations() != null) {
            for (AnnotationExpr a : n.getAnnotations()) {
                {
                    R result = a.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getArgs() != null) {
            for (Expression e : n.getArgs()) {
                {
                    R result = e.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getClassBody() != null) {
            for (BodyDeclaration<?> member : n.getClassBody()) {
                {
                    R result = member.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final EnumDeclaration n, final A arg) {
        visitComment(n, arg);
        if (n.getAnnotations() != null) {
            for (AnnotationExpr a : n.getAnnotations()) {
                {
                    R result = a.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getImplements() != null) {
            for (ClassOrInterfaceType c : n.getImplements()) {
                {
                    R result = c.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getEntries() != null) {
            for (EnumConstantDeclaration e : n.getEntries()) {
                {
                    R result = e.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getMembers() != null) {
            for (BodyDeclaration<?> member : n.getMembers()) {
                {
                    R result = member.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final ExplicitConstructorInvocationStmt n, final A arg) {
        visitComment(n, arg);
        if (!n.isThis() && n.getExpr() != null) {
            {
                R result = n.getExpr().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        if (n.getTypeArguments() != null) {
            for (Type<?> type : n.getTypeArguments()) {
                R result = type.accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        if (n.getArgs() != null) {
            for (Expression e : n.getArgs()) {
                {
                    R result = e.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final ExpressionStmt n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getExpression().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final FieldAccessExpr n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getScope().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        if (n.getTypeArguments() != null) {
            for (Type<?> type : n.getTypeArguments()) {
                R result = type.accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final FieldDeclaration n, final A arg) {
        visitComment(n, arg);
        if (n.getAnnotations() != null) {
            for (AnnotationExpr a : n.getAnnotations()) {
                {
                    R result = a.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        {
            R result = n.getElementType().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        for (VariableDeclarator var : n.getVariables()) {
            {
                R result = var.accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final ForeachStmt n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getVariable().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getIterable().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getBody().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final ForStmt n, final A arg) {
        visitComment(n, arg);
        if (n.getInit() != null) {
            for (Expression e : n.getInit()) {
                {
                    R result = e.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getCompare() != null) {
            {
                R result = n.getCompare().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        if (n.getUpdate() != null) {
            for (Expression e : n.getUpdate()) {
                {
                    R result = e.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        {
            R result = n.getBody().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final IfStmt n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getCondition().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getThenStmt().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        if (n.getElseStmt() != null) {
            {
                R result = n.getElseStmt().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final ImportDeclaration n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getName().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final InitializerDeclaration n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getBlock().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final InstanceOfExpr n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getExpr().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getType().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final IntegerLiteralExpr n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final IntegerLiteralMinValueExpr n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final JavadocComment n, final A arg) {
        return null;
    }
    @Override
	public R visit(final LabeledStmt n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getStmt().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final LongLiteralExpr n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final LongLiteralMinValueExpr n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final MarkerAnnotationExpr n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getName().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final MemberValuePair n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getValue().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final MethodCallExpr n, final A arg) {
        visitComment(n, arg);
        if (n.getScope() != null) {
            {
                R result = n.getScope().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        if (n.getTypeArguments() != null) {
            for (Type<?> type : n.getTypeArguments()) {
                R result = type.accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        if (n.getArgs() != null) {
            for (Expression e : n.getArgs()) {
                {
                    R result = e.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final MethodDeclaration n, final A arg) {
        visitComment(n, arg);
        if (n.getAnnotations() != null) {
            for (AnnotationExpr a : n.getAnnotations()) {
                {
                    R result = a.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getTypeParameters() != null) {
            for (TypeParameter t : n.getTypeParameters()) {
                {
                    R result = t.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        {
            R result = n.getElementType().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        if (n.getParameters() != null) {
            for (Parameter p : n.getParameters()) {
                {
                    R result = p.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getThrows() != null) {
            for (ReferenceType name : n.getThrows()) {
                {
                    R result = name.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getBody() != null) {
            {
                R result = n.getBody().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final NameExpr n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final NormalAnnotationExpr n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getName().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        if (n.getPairs() != null) {
            for (MemberValuePair m : n.getPairs()) {
                {
                    R result = m.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final NullLiteralExpr n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final ObjectCreationExpr n, final A arg) {
        visitComment(n, arg);
        if (n.getScope() != null) {
            {
                R result = n.getScope().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        if (n.getTypeArguments() != null) {
            for (Type<?> type : n.getTypeArguments()) {
                R result = type.accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        {
            R result = n.getType().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        if (n.getArgs() != null) {
            for (Expression e : n.getArgs()) {
                {
                    R result = e.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getAnonymousClassBody() != null) {
            for (BodyDeclaration<?> member : n.getAnonymousClassBody()) {
                {
                    R result = member.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final PackageDeclaration n, final A arg) {
        visitComment(n, arg);
        if (n.getAnnotations() != null) {
            for (AnnotationExpr a : n.getAnnotations()) {
                {
                    R result = a.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        {
            R result = n.getName().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final Parameter n, final A arg) {
        visitComment(n, arg);
        if (n.getAnnotations() != null) {
            for (AnnotationExpr a : n.getAnnotations()) {
                {
                    R result = a.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        {
            R result = n.getElementType().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getId().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final PrimitiveType n, final A arg) {
        visitComment(n, arg);
        for (AnnotationExpr a : n.getAnnotations()) {
            R result = a.accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final QualifiedNameExpr n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getQualifier().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(ArrayType n, A arg) {
        visitComment(n, arg);
        for (AnnotationExpr a : n.getAnnotations()) {
            R result = a.accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getComponentType().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(ArrayCreationLevel n, A arg) {
        visitComment(n, arg);
        for (AnnotationExpr a : n.getAnnotations()) {
            R result = a.accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        if (n.getDimension() != null) {
            R result = n.getDimension().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
    public R visit(final IntersectionType n, final A arg) {
        visitComment(n, arg);
        for (AnnotationExpr a : n.getAnnotations()) {
            R result = a.accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            for (ReferenceType element : n.getElements()) {
                R result = element.accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
    public R visit(final UnionType n, final A arg) {
        visitComment(n, arg);
        for (AnnotationExpr a : n.getAnnotations()) {
            R result = a.accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            for (ReferenceType element : n.getElements()) {
                R result = element.accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final ReturnStmt n, final A arg) {
        visitComment(n, arg);
        if (n.getExpr() != null) {
            {
                R result = n.getExpr().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final SingleMemberAnnotationExpr n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getName().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getMemberValue().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final StringLiteralExpr n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final SuperExpr n, final A arg) {
        visitComment(n, arg);
        if (n.getClassExpr() != null) {
            {
                R result = n.getClassExpr().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final SwitchEntryStmt n, final A arg) {
        visitComment(n, arg);
        if (n.getLabel() != null) {
            {
                R result = n.getLabel().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        if (n.getStmts() != null) {
            for (Statement s : n.getStmts()) {
                {
                    R result = s.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final SwitchStmt n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getSelector().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        if (n.getEntries() != null) {
            for (SwitchEntryStmt e : n.getEntries()) {
                {
                    R result = e.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final SynchronizedStmt n, final A arg) {
        visitComment(n, arg);
        if (n.getExpr() != null) {
            R result = n.getExpr().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getBody().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final ThisExpr n, final A arg) {
        visitComment(n, arg);
        if (n.getClassExpr() != null) {
            {
                R result = n.getClassExpr().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final ThrowStmt n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getExpr().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final TryStmt n, final A arg) {
        visitComment(n, arg);
        if (n.getResources() != null) {
            for (VariableDeclarationExpr v : n.getResources()) {
                {
                    R result = v.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        {
            R result = n.getTryBlock().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        if (n.getCatchs() != null) {
            for (CatchClause c : n.getCatchs()) {
                {
                    R result = c.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        if (n.getFinallyBlock() != null) {
            {
                R result = n.getFinallyBlock().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final TypeDeclarationStmt n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getTypeDeclaration().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final TypeParameter n, final A arg) {
        visitComment(n, arg);
        if (n.getTypeBound() != null) {
            for (ClassOrInterfaceType c : n.getTypeBound()) {
                {
                    R result = c.accept(this, arg);
                    if (result != null) {
                        return result;
                    }
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final UnaryExpr n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getExpr().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final UnknownType n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final VariableDeclarationExpr n, final A arg) {
        visitComment(n, arg);
        for (AnnotationExpr a : n.getAnnotations()) {
            R result = a.accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getElementType().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        for (VariableDeclarator v : n.getVariables()) {
            {
                R result = v.accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final VariableDeclarator n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getId().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        if (n.getInit() != null) {
            {
                R result = n.getInit().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final VariableDeclaratorId n, final A arg) {
        visitComment(n, arg);
        return null;
    }
    @Override
	public R visit(final VoidType n, final A arg) {
        visitComment(n, arg);
        for (AnnotationExpr a : n.getAnnotations()) {
            R result = a.accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final WhileStmt n, final A arg) {
        visitComment(n, arg);
        {
            R result = n.getCondition().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        {
            R result = n.getBody().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(final WildcardType n, final A arg) {
        visitComment(n, arg);
        for (AnnotationExpr a : n.getAnnotations()) {
            R result = a.accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        if (n.getExtends() != null) {
            {
                R result = n.getExtends().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        if (n.getSuper() != null) {
            {
                R result = n.getSuper().accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
    public R visit(LambdaExpr n, A arg) {
        visitComment(n, arg);
        if (n.getParameters() != null) {
            for (Parameter a : n.getParameters()) {
                R result = a.accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        if (n.getBody() != null) {
            R result = n.getBody().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
    public R visit(MethodReferenceExpr n, A arg) {
        visitComment(n, arg);
        if (n.getTypeArguments() != null) {
            for (Type<?> type : n.getTypeArguments()) {
                R result = type.accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        if (n.getScope() != null) {
            R result = n.getScope().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
    public R visit(TypeExpr n, A arg) {
        visitComment(n, arg);
        if (n.getType() != null) {
            R result = n.getType().accept(this, arg);
            if (result != null) {
                return result;
            }
        }
        return null;
    }
    @Override
	public R visit(ArrayBracketPair n, A arg) {
        for (AnnotationExpr a : n.getAnnotations()) {
            {
                R result = a.accept(this, arg);
                if (result != null) {
                    return result;
                }
            }
        }
        return null;
    }
    @Override
	public R visit(final BlockComment n, final A arg) {
        return null;
    }
    @Override
	public R visit(final LineComment n, final A arg) {
        return null;
    }
    private void visitComment(Node n, A arg) {
        if (n.getComment() != null) {
            Comment result = (Comment) n.getComment().accept(this, arg);
            if (result != null) {
                n.setComment(result);
            }
        }
    }
}
