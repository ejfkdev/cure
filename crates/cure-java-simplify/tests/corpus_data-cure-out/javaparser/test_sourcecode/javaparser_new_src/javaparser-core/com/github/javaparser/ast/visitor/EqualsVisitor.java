package com.github.javaparser.ast.visitor;

import java.util.List;
import com.github.javaparser.ast.*;
import com.github.javaparser.ast.type.TypeParameter;
import com.github.javaparser.ast.body.AnnotationDeclaration;
import com.github.javaparser.ast.body.AnnotationMemberDeclaration;
import com.github.javaparser.ast.body.ClassOrInterfaceDeclaration;
import com.github.javaparser.ast.body.ConstructorDeclaration;
import com.github.javaparser.ast.body.EmptyMemberDeclaration;
import com.github.javaparser.ast.body.EmptyTypeDeclaration;
import com.github.javaparser.ast.body.EnumConstantDeclaration;
import com.github.javaparser.ast.body.EnumDeclaration;
import com.github.javaparser.ast.body.FieldDeclaration;
import com.github.javaparser.ast.body.InitializerDeclaration;
import com.github.javaparser.ast.body.MethodDeclaration;
import com.github.javaparser.ast.body.Parameter;
import com.github.javaparser.ast.body.VariableDeclarator;
import com.github.javaparser.ast.body.VariableDeclaratorId;
import com.github.javaparser.ast.comments.BlockComment;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.comments.LineComment;
import com.github.javaparser.ast.expr.ArrayAccessExpr;
import com.github.javaparser.ast.expr.ArrayCreationExpr;
import com.github.javaparser.ast.expr.ArrayInitializerExpr;
import com.github.javaparser.ast.expr.AssignExpr;
import com.github.javaparser.ast.expr.BinaryExpr;
import com.github.javaparser.ast.expr.BooleanLiteralExpr;
import com.github.javaparser.ast.expr.CastExpr;
import com.github.javaparser.ast.expr.CharLiteralExpr;
import com.github.javaparser.ast.expr.ClassExpr;
import com.github.javaparser.ast.expr.ConditionalExpr;
import com.github.javaparser.ast.expr.DoubleLiteralExpr;
import com.github.javaparser.ast.expr.EnclosedExpr;
import com.github.javaparser.ast.expr.FieldAccessExpr;
import com.github.javaparser.ast.expr.InstanceOfExpr;
import com.github.javaparser.ast.expr.IntegerLiteralExpr;
import com.github.javaparser.ast.expr.IntegerLiteralMinValueExpr;
import com.github.javaparser.ast.expr.LambdaExpr;
import com.github.javaparser.ast.expr.LongLiteralExpr;
import com.github.javaparser.ast.expr.LongLiteralMinValueExpr;
import com.github.javaparser.ast.expr.MarkerAnnotationExpr;
import com.github.javaparser.ast.expr.MemberValuePair;
import com.github.javaparser.ast.expr.MethodCallExpr;
import com.github.javaparser.ast.expr.MethodReferenceExpr;
import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.ast.expr.NormalAnnotationExpr;
import com.github.javaparser.ast.expr.NullLiteralExpr;
import com.github.javaparser.ast.expr.ObjectCreationExpr;
import com.github.javaparser.ast.expr.QualifiedNameExpr;
import com.github.javaparser.ast.expr.SingleMemberAnnotationExpr;
import com.github.javaparser.ast.expr.StringLiteralExpr;
import com.github.javaparser.ast.expr.SuperExpr;
import com.github.javaparser.ast.expr.ThisExpr;
import com.github.javaparser.ast.expr.TypeExpr;
import com.github.javaparser.ast.expr.UnaryExpr;
import com.github.javaparser.ast.expr.VariableDeclarationExpr;
import com.github.javaparser.ast.stmt.AssertStmt;
import com.github.javaparser.ast.stmt.BlockStmt;
import com.github.javaparser.ast.stmt.BreakStmt;
import com.github.javaparser.ast.stmt.CatchClause;
import com.github.javaparser.ast.stmt.ContinueStmt;
import com.github.javaparser.ast.stmt.DoStmt;
import com.github.javaparser.ast.stmt.EmptyStmt;
import com.github.javaparser.ast.stmt.ExplicitConstructorInvocationStmt;
import com.github.javaparser.ast.stmt.ExpressionStmt;
import com.github.javaparser.ast.stmt.ForStmt;
import com.github.javaparser.ast.stmt.ForeachStmt;
import com.github.javaparser.ast.stmt.IfStmt;
import com.github.javaparser.ast.stmt.LabeledStmt;
import com.github.javaparser.ast.stmt.ReturnStmt;
import com.github.javaparser.ast.stmt.SwitchEntryStmt;
import com.github.javaparser.ast.stmt.SwitchStmt;
import com.github.javaparser.ast.stmt.SynchronizedStmt;
import com.github.javaparser.ast.stmt.ThrowStmt;
import com.github.javaparser.ast.stmt.TryStmt;
import com.github.javaparser.ast.stmt.TypeDeclarationStmt;
import com.github.javaparser.ast.stmt.WhileStmt;
import com.github.javaparser.ast.type.*;

public class EqualsVisitor implements GenericVisitor<Boolean, Node> {
    private static final EqualsVisitor SINGLETON = new EqualsVisitor();
    public static boolean equals(final Node n1, final Node n2) {
        return SINGLETON.nodeEquals(n1, n2);
    }
    private EqualsVisitor() {}
    private boolean commonNodeEquality(Node n1, Node n2) {
        return !nodeEquals(n1.getComment(), n2.getComment()) ? false : nodesEquals(n1.getOrphanComments(), n2.getOrphanComments());
    }
    private <T extends Node> boolean nodesEquals(final List<T> nodes1, final List<T> nodes2) {
        if (nodes1 == null) {
            return nodes2 == null;
        } else if (nodes2 == null) {
            return false;
        }
        if (nodes1.size() != nodes2.size()) {
            return false;
        }
        for (int i = 0; i < nodes1.size(); i++) {
            if (!nodeEquals(nodes1.get(i), nodes2.get(i))) {
                return false;
            }
        }
        return true;
    }
    private <T extends Node> boolean nodeEquals(final T n1, final T n2) {
        return n1 == n2 ? true : n1 == null || n2 == null ? false : n1.getClass() != n2.getClass() ? false : !commonNodeEquality(n1, n2) ? false : n1.accept(this, n2);
    }
    private boolean objEquals(final Object n1, final Object n2) {
        return n1 == n2 ? true : n1 == null || n2 == null ? false : n1.equals(n2);
    }
    @Override public Boolean visit(final CompilationUnit n1, final Node arg) {
        CompilationUnit n2 = (CompilationUnit) arg;
        return !nodeEquals(n1.getPackage(), n2.getPackage()) ? false : !nodesEquals(n1.getImports(), n2.getImports()) ? false : !nodesEquals(n1.getTypes(), n2.getTypes()) ? false : nodesEquals(n1.getComments(), n2.getComments());
    }
    @Override public Boolean visit(final PackageDeclaration n1, final Node arg) {
        PackageDeclaration n2 = (PackageDeclaration) arg;
        return !nodeEquals(n1.getName(), n2.getName()) ? false : nodesEquals(n1.getAnnotations(), n2.getAnnotations());
    }
    @Override public Boolean visit(final ImportDeclaration n1, final Node arg) {
        ImportDeclaration n2 = (ImportDeclaration) arg;
        return nodeEquals(n1.getName(), n2.getName());
    }
    @Override public Boolean visit(final TypeParameter n1, final Node arg) {
        TypeParameter n2 = (TypeParameter) arg;
        return !objEquals(n1.getName(), n2.getName()) ? false : !nodesEquals(n1.getTypeBound(), n2.getTypeBound()) ? false : nodesEquals(n1.getAnnotations(), n2.getAnnotations());
    }
    @Override public Boolean visit(final LineComment n1, final Node arg) {
        LineComment n2 = (LineComment) arg;
        return !objEquals(n1.getContent(), n2.getContent()) ? false : objEquals(n1.getBegin().line, n2.getBegin().line);
    }
    @Override public Boolean visit(final BlockComment n1, final Node arg) {
        BlockComment n2 = (BlockComment) arg;
        return !objEquals(n1.getContent(), n2.getContent()) ? false : objEquals(n1.getBegin().line, n2.getBegin().line);
    }
    @Override public Boolean visit(final ClassOrInterfaceDeclaration n1, final Node arg) {
        ClassOrInterfaceDeclaration n2 = (ClassOrInterfaceDeclaration) arg;
        return !n1.getModifiers().equals(n2.getModifiers()) ? false : n1.isInterface() != n2.isInterface() ? false : !objEquals(n1.getName(), n2.getName()) ? false : !nodesEquals(n1.getAnnotations(), n2.getAnnotations()) ? false : !nodesEquals(n1.getTypeParameters(), n2.getTypeParameters()) ? false : !nodesEquals(n1.getExtends(), n2.getExtends()) ? false : !nodesEquals(n1.getImplements(), n2.getImplements()) ? false : nodesEquals(n1.getMembers(), n2.getMembers());
    }
    @Override public Boolean visit(final EnumDeclaration n1, final Node arg) {
        EnumDeclaration n2 = (EnumDeclaration) arg;
        return !n1.getModifiers().equals(n2.getModifiers()) ? false : !objEquals(n1.getName(), n2.getName()) ? false : !nodesEquals(n1.getAnnotations(), n2.getAnnotations()) ? false : !nodesEquals(n1.getImplements(), n2.getImplements()) ? false : !nodesEquals(n1.getEntries(), n2.getEntries()) ? false : nodesEquals(n1.getMembers(), n2.getMembers());
    }
    @Override public Boolean visit(final EmptyTypeDeclaration n1, final Node arg) {
        return true;
    }
    @Override public Boolean visit(final EnumConstantDeclaration n1, final Node arg) {
        EnumConstantDeclaration n2 = (EnumConstantDeclaration) arg;
        return !objEquals(n1.getName(), n2.getName()) ? false : !nodesEquals(n1.getAnnotations(), n2.getAnnotations()) ? false : !nodesEquals(n1.getArgs(), n2.getArgs()) ? false : nodesEquals(n1.getClassBody(), n2.getClassBody());
    }
    @Override public Boolean visit(final AnnotationDeclaration n1, final Node arg) {
        AnnotationDeclaration n2 = (AnnotationDeclaration) arg;
        return !n1.getModifiers().equals(n2.getModifiers()) ? false : !objEquals(n1.getName(), n2.getName()) ? false : !nodesEquals(n1.getAnnotations(), n2.getAnnotations()) ? false : nodesEquals(n1.getMembers(), n2.getMembers());
    }
    @Override public Boolean visit(final AnnotationMemberDeclaration n1, final Node arg) {
        AnnotationMemberDeclaration n2 = (AnnotationMemberDeclaration) arg;
        return !n1.getModifiers().equals(n2.getModifiers()) ? false : !objEquals(n1.getName(), n2.getName()) ? false : !nodesEquals(n1.getAnnotations(), n2.getAnnotations()) ? false : !nodeEquals(n1.getDefaultValue(), n2.getDefaultValue()) ? false : nodeEquals(n1.getType(), n2.getType());
    }
    @Override public Boolean visit(final FieldDeclaration n1, final Node arg) {
        FieldDeclaration n2 = (FieldDeclaration) arg;
        return !n1.getModifiers().equals(n2.getModifiers()) ? false : !nodesEquals(n1.getAnnotations(), n2.getAnnotations()) ? false : !nodeEquals(n1.getElementType(), n2.getElementType()) ? false : !nodesEquals(n1.getVariables(), n2.getVariables()) ? false : nodesEquals(n1.getArrayBracketPairsAfterElementType(), n2.getArrayBracketPairsAfterElementType());
    }
    @Override public Boolean visit(final VariableDeclarator n1, final Node arg) {
        VariableDeclarator n2 = (VariableDeclarator) arg;
        return !nodeEquals(n1.getId(), n2.getId()) ? false : nodeEquals(n1.getInit(), n2.getInit());
    }
    @Override public Boolean visit(final VariableDeclaratorId n1, final Node arg) {
        VariableDeclaratorId n2 = (VariableDeclaratorId) arg;
        return !nodesEquals(n1.getArrayBracketPairsAfterId(), n2.getArrayBracketPairsAfterId()) ? false : objEquals(n1.getName(), n2.getName());
    }
    @Override public Boolean visit(final ConstructorDeclaration n1, final Node arg) {
        ConstructorDeclaration n2 = (ConstructorDeclaration) arg;
        return !n1.getModifiers().equals(n2.getModifiers()) ? false : !objEquals(n1.getName(), n2.getName()) ? false : !nodesEquals(n1.getAnnotations(), n2.getAnnotations()) ? false : !nodeEquals(n1.getBody(), n2.getBody()) ? false : !nodesEquals(n1.getParameters(), n2.getParameters()) ? false : !nodesEquals(n1.getThrows(), n2.getThrows()) ? false : nodesEquals(n1.getTypeParameters(), n2.getTypeParameters());
    }
    @Override public Boolean visit(final MethodDeclaration n1, final Node arg) {
        MethodDeclaration n2 = (MethodDeclaration) arg;
        return !n1.getModifiers().equals(n2.getModifiers()) ? false : !nodesEquals(n1.getArrayBracketPairsAfterElementType(), n2.getArrayBracketPairsAfterElementType()) ? false : !nodesEquals(n1.getArrayBracketPairsAfterParameterList(), n2.getArrayBracketPairsAfterParameterList()) ? false : !objEquals(n1.getName(), n2.getName()) ? false : !nodeEquals(n1.getElementType(), n2.getElementType()) ? false : !nodesEquals(n1.getAnnotations(), n2.getAnnotations()) ? false : !nodeEquals(n1.getBody(), n2.getBody()) ? false : !nodesEquals(n1.getParameters(), n2.getParameters()) ? false : !nodesEquals(n1.getThrows(), n2.getThrows()) ? false : !nodesEquals(n1.getTypeParameters(), n2.getTypeParameters()) ? false : n1.isDefault() == n2.isDefault();
    }
    @Override public Boolean visit(final Parameter n1, final Node arg) {
        Parameter n2 = (Parameter) arg;
        return !nodeEquals(n1.getElementType(), n2.getElementType()) ? false : !nodesEquals(n1.getArrayBracketPairsAfterElementType(), n2.getArrayBracketPairsAfterElementType()) ? false : !n1.getModifiers().equals(n2.getModifiers()) ? false : !nodeEquals(n1.getId(), n2.getId()) ? false : nodesEquals(n1.getAnnotations(), n2.getAnnotations());
    }
    @Override public Boolean visit(final EmptyMemberDeclaration n1, final Node arg) {
        return true;
    }
    @Override public Boolean visit(final InitializerDeclaration n1, final Node arg) {
        InitializerDeclaration n2 = (InitializerDeclaration) arg;
        return !nodeEquals(n1.getBlock(), n2.getBlock()) ? false : nodesEquals(n1.getAnnotations(), n2.getAnnotations());
    }
    @Override public Boolean visit(final JavadocComment n1, final Node arg) {
        JavadocComment n2 = (JavadocComment) arg;
        return objEquals(n1.getContent(), n2.getContent());
    }
    @Override public Boolean visit(final ClassOrInterfaceType n1, final Node arg) {
        ClassOrInterfaceType n2 = (ClassOrInterfaceType) arg;
        return !objEquals(n1.getName(), n2.getName()) ? false : !nodeEquals(n1.getScope(), n2.getScope()) ? false : !nodesEquals(n1.getTypeArguments(), n2.getTypeArguments()) ? false : nodesEquals(n1.getAnnotations(), n2.getAnnotations());
    }
    @Override public Boolean visit(final PrimitiveType n1, final Node arg) {
        PrimitiveType n2 = (PrimitiveType) arg;
        return n1.getType() != n2.getType() ? false : nodesEquals(n1.getAnnotations(), n2.getAnnotations());
    }
    @Override
	public Boolean visit(ArrayType n1, Node arg) {
        ArrayType n2 = (ArrayType) arg;
        return !nodeEquals(n1.getComponentType(), n2.getComponentType()) ? false : nodesEquals(n1.getAnnotations(), n2.getAnnotations());
    }
    @Override
	public Boolean visit(ArrayCreationLevel n1, Node arg) {
        ArrayCreationLevel n2 = (ArrayCreationLevel) arg;
        return !nodeEquals(n1.getDimension(), n2.getDimension()) ? false : nodesEquals(n1.getAnnotations(), n2.getAnnotations());
    }
    @Override public Boolean visit(final IntersectionType n1, final Node arg) {
        IntersectionType n2 = (IntersectionType) arg;
        if (!nodesEquals(n1.getAnnotations(), n2.getAnnotations())) {
            return false;
        }
        List<ReferenceType> n1Elements = n1.getElements();
        List<ReferenceType> n2Elements = n2.getElements();
        if (n1Elements != null && n2Elements != null) {
            if (n1Elements.size() != n2Elements.size()) {
                return false;
            } else {
                int i = 0;
                for (ReferenceType aux : n1Elements) {
                    if (aux.accept(this, n2Elements.get(i))) {
                        return false;
                    }
                    i++;
                }
            }
        } else if (n1Elements != n2Elements) {
            return false;
        }
        return true;
    }
    @Override public Boolean visit(final UnionType n1, final Node arg) {
        UnionType n2 = (UnionType) arg;
        if (!nodesEquals(n1.getAnnotations(), n2.getAnnotations())) {
            return false;
        }
        List<ReferenceType> n1Elements = n1.getElements();
        List<ReferenceType> n2Elements = n2.getElements();
        if (n1Elements != null && n2Elements != null) {
            if (n1Elements.size() != n2Elements.size()) {
                return false;
            } else {
                int i = 0;
                for (ReferenceType aux : n1Elements) {
                    if (aux.accept(this, n2Elements.get(i))) {
                        return false;
                    }
                    i++;
                }
            }
        } else if (n1Elements != n2Elements) {
            return false;
        }
        return true;
    }
    @Override
	public Boolean visit(VoidType n1, Node arg) {
        VoidType n2 = (VoidType) arg;
        return nodesEquals(n1.getAnnotations(), n2.getAnnotations());
    }
    @Override public Boolean visit(final WildcardType n1, final Node arg) {
        WildcardType n2 = (WildcardType) arg;
        return !nodeEquals(n1.getExtends(), n2.getExtends()) ? false : !nodeEquals(n1.getSuper(), n2.getSuper()) ? false : nodesEquals(n1.getAnnotations(), n2.getAnnotations());
    }
    @Override public Boolean visit(final UnknownType n1, final Node arg) {
        return true;
    }
    @Override public Boolean visit(final ArrayAccessExpr n1, final Node arg) {
        ArrayAccessExpr n2 = (ArrayAccessExpr) arg;
        return !nodeEquals(n1.getName(), n2.getName()) ? false : nodeEquals(n1.getIndex(), n2.getIndex());
    }
    @Override public Boolean visit(final ArrayCreationExpr n1, final Node arg) {
        ArrayCreationExpr n2 = (ArrayCreationExpr) arg;
        return !nodeEquals(n1.getType(), n2.getType()) ? false : !nodesEquals(n1.getLevels(), n2.getLevels()) ? false : nodeEquals(n1.getInitializer(), n2.getInitializer());
    }
    @Override public Boolean visit(final ArrayInitializerExpr n1, final Node arg) {
        ArrayInitializerExpr n2 = (ArrayInitializerExpr) arg;
        return nodesEquals(n1.getValues(), n2.getValues());
    }
    @Override public Boolean visit(final AssignExpr n1, final Node arg) {
        AssignExpr n2 = (AssignExpr) arg;
        return n1.getOperator() != n2.getOperator() ? false : !nodeEquals(n1.getTarget(), n2.getTarget()) ? false : nodeEquals(n1.getValue(), n2.getValue());
    }
    @Override public Boolean visit(final BinaryExpr n1, final Node arg) {
        BinaryExpr n2 = (BinaryExpr) arg;
        return n1.getOperator() != n2.getOperator() ? false : !nodeEquals(n1.getLeft(), n2.getLeft()) ? false : nodeEquals(n1.getRight(), n2.getRight());
    }
    @Override public Boolean visit(final CastExpr n1, final Node arg) {
        CastExpr n2 = (CastExpr) arg;
        return !nodeEquals(n1.getType(), n2.getType()) ? false : nodeEquals(n1.getExpr(), n2.getExpr());
    }
    @Override public Boolean visit(final ClassExpr n1, final Node arg) {
        ClassExpr n2 = (ClassExpr) arg;
        return nodeEquals(n1.getType(), n2.getType());
    }
    @Override public Boolean visit(final ConditionalExpr n1, final Node arg) {
        ConditionalExpr n2 = (ConditionalExpr) arg;
        return !nodeEquals(n1.getCondition(), n2.getCondition()) ? false : !nodeEquals(n1.getThenExpr(), n2.getThenExpr()) ? false : nodeEquals(n1.getElseExpr(), n2.getElseExpr());
    }
    @Override public Boolean visit(final EnclosedExpr n1, final Node arg) {
        EnclosedExpr n2 = (EnclosedExpr) arg;
        return nodeEquals(n1.getInner(), n2.getInner());
    }
    @Override public Boolean visit(final FieldAccessExpr n1, final Node arg) {
        FieldAccessExpr n2 = (FieldAccessExpr) arg;
        return !nodeEquals(n1.getScope(), n2.getScope()) ? false : !objEquals(n1.getField(), n2.getField()) ? false : nodesEquals(n1.getTypeArguments(), n2.getTypeArguments());
    }
    @Override public Boolean visit(final InstanceOfExpr n1, final Node arg) {
        InstanceOfExpr n2 = (InstanceOfExpr) arg;
        return !nodeEquals(n1.getExpr(), n2.getExpr()) ? false : nodeEquals(n1.getType(), n2.getType());
    }
    @Override public Boolean visit(final StringLiteralExpr n1, final Node arg) {
        StringLiteralExpr n2 = (StringLiteralExpr) arg;
        return objEquals(n1.getValue(), n2.getValue());
    }
    @Override public Boolean visit(final IntegerLiteralExpr n1, final Node arg) {
        IntegerLiteralExpr n2 = (IntegerLiteralExpr) arg;
        return objEquals(n1.getValue(), n2.getValue());
    }
    @Override public Boolean visit(final LongLiteralExpr n1, final Node arg) {
        LongLiteralExpr n2 = (LongLiteralExpr) arg;
        return objEquals(n1.getValue(), n2.getValue());
    }
    @Override public Boolean visit(final IntegerLiteralMinValueExpr n1, final Node arg) {
        IntegerLiteralMinValueExpr n2 = (IntegerLiteralMinValueExpr) arg;
        return objEquals(n1.getValue(), n2.getValue());
    }
    @Override public Boolean visit(final LongLiteralMinValueExpr n1, final Node arg) {
        LongLiteralMinValueExpr n2 = (LongLiteralMinValueExpr) arg;
        return objEquals(n1.getValue(), n2.getValue());
    }
    @Override public Boolean visit(final CharLiteralExpr n1, final Node arg) {
        CharLiteralExpr n2 = (CharLiteralExpr) arg;
        return objEquals(n1.getValue(), n2.getValue());
    }
    @Override public Boolean visit(final DoubleLiteralExpr n1, final Node arg) {
        DoubleLiteralExpr n2 = (DoubleLiteralExpr) arg;
        return objEquals(n1.getValue(), n2.getValue());
    }
    @Override public Boolean visit(final BooleanLiteralExpr n1, final Node arg) {
        BooleanLiteralExpr n2 = (BooleanLiteralExpr) arg;
        return n1.getValue() == n2.getValue();
    }
    @Override public Boolean visit(final NullLiteralExpr n1, final Node arg) {
        return true;
    }
    @Override public Boolean visit(final MethodCallExpr n1, final Node arg) {
        MethodCallExpr n2 = (MethodCallExpr) arg;
        return !nodeEquals(n1.getScope(), n2.getScope()) ? false : !objEquals(n1.getName(), n2.getName()) ? false : !nodesEquals(n1.getArgs(), n2.getArgs()) ? false : nodesEquals(n1.getTypeArguments(), n2.getTypeArguments());
    }
    @Override public Boolean visit(final NameExpr n1, final Node arg) {
        NameExpr n2 = (NameExpr) arg;
        return objEquals(n1.getName(), n2.getName());
    }
    @Override public Boolean visit(final ObjectCreationExpr n1, final Node arg) {
        ObjectCreationExpr n2 = (ObjectCreationExpr) arg;
        return !nodeEquals(n1.getScope(), n2.getScope()) ? false : !nodeEquals(n1.getType(), n2.getType()) ? false : !nodesEquals(n1.getAnonymousClassBody(), n2.getAnonymousClassBody()) ? false : !nodesEquals(n1.getArgs(), n2.getArgs()) ? false : nodesEquals(n1.getTypeArguments(), n2.getTypeArguments());
    }
    @Override public Boolean visit(final QualifiedNameExpr n1, final Node arg) {
        QualifiedNameExpr n2 = (QualifiedNameExpr) arg;
        return !nodeEquals(n1.getQualifier(), n2.getQualifier()) ? false : objEquals(n1.getName(), n2.getName());
    }
    @Override public Boolean visit(final ThisExpr n1, final Node arg) {
        ThisExpr n2 = (ThisExpr) arg;
        return nodeEquals(n1.getClassExpr(), n2.getClassExpr());
    }
    @Override public Boolean visit(final SuperExpr n1, final Node arg) {
        SuperExpr n2 = (SuperExpr) arg;
        return nodeEquals(n1.getClassExpr(), n2.getClassExpr());
    }
    @Override public Boolean visit(final UnaryExpr n1, final Node arg) {
        UnaryExpr n2 = (UnaryExpr) arg;
        return n1.getOperator() != n2.getOperator() ? false : nodeEquals(n1.getExpr(), n2.getExpr());
    }
    @Override public Boolean visit(final VariableDeclarationExpr n1, final Node arg) {
        VariableDeclarationExpr n2 = (VariableDeclarationExpr) arg;
        return !n1.getModifiers().equals(n2.getModifiers()) ? false : !nodesEquals(n1.getAnnotations(), n2.getAnnotations()) ? false : !nodeEquals(n1.getElementType(), n2.getElementType()) ? false : !nodesEquals(n1.getVariables(), n2.getVariables()) ? false : nodesEquals(n1.getArrayBracketPairsAfterElementType(), n2.getArrayBracketPairsAfterElementType());
    }
    @Override public Boolean visit(final MarkerAnnotationExpr n1, final Node arg) {
        MarkerAnnotationExpr n2 = (MarkerAnnotationExpr) arg;
        return nodeEquals(n1.getName(), n2.getName());
    }
    @Override public Boolean visit(final SingleMemberAnnotationExpr n1, final Node arg) {
        SingleMemberAnnotationExpr n2 = (SingleMemberAnnotationExpr) arg;
        return !nodeEquals(n1.getName(), n2.getName()) ? false : nodeEquals(n1.getMemberValue(), n2.getMemberValue());
    }
    @Override public Boolean visit(final NormalAnnotationExpr n1, final Node arg) {
        NormalAnnotationExpr n2 = (NormalAnnotationExpr) arg;
        return !nodeEquals(n1.getName(), n2.getName()) ? false : nodesEquals(n1.getPairs(), n2.getPairs());
    }
    @Override public Boolean visit(final MemberValuePair n1, final Node arg) {
        MemberValuePair n2 = (MemberValuePair) arg;
        return !objEquals(n1.getName(), n2.getName()) ? false : nodeEquals(n1.getValue(), n2.getValue());
    }
    @Override public Boolean visit(final ExplicitConstructorInvocationStmt n1, final Node arg) {
        ExplicitConstructorInvocationStmt n2 = (ExplicitConstructorInvocationStmt) arg;
        return !nodeEquals(n1.getExpr(), n2.getExpr()) ? false : !nodesEquals(n1.getArgs(), n2.getArgs()) ? false : nodesEquals(n1.getTypeArguments(), n2.getTypeArguments());
    }
    @Override public Boolean visit(final TypeDeclarationStmt n1, final Node arg) {
        TypeDeclarationStmt n2 = (TypeDeclarationStmt) arg;
        return nodeEquals(n1.getTypeDeclaration(), n2.getTypeDeclaration());
    }
    @Override public Boolean visit(final AssertStmt n1, final Node arg) {
        AssertStmt n2 = (AssertStmt) arg;
        return !nodeEquals(n1.getCheck(), n2.getCheck()) ? false : nodeEquals(n1.getMessage(), n2.getMessage());
    }
    @Override public Boolean visit(final BlockStmt n1, final Node arg) {
        BlockStmt n2 = (BlockStmt) arg;
        return nodesEquals(n1.getStmts(), n2.getStmts());
    }
    @Override public Boolean visit(final LabeledStmt n1, final Node arg) {
        LabeledStmt n2 = (LabeledStmt) arg;
        return nodeEquals(n1.getStmt(), n2.getStmt());
    }
    @Override public Boolean visit(final EmptyStmt n1, final Node arg) {
        return true;
    }
    @Override public Boolean visit(final ExpressionStmt n1, final Node arg) {
        ExpressionStmt n2 = (ExpressionStmt) arg;
        return nodeEquals(n1.getExpression(), n2.getExpression());
    }
    @Override public Boolean visit(final SwitchStmt n1, final Node arg) {
        SwitchStmt n2 = (SwitchStmt) arg;
        return !nodeEquals(n1.getSelector(), n2.getSelector()) ? false : nodesEquals(n1.getEntries(), n2.getEntries());
    }
    @Override public Boolean visit(final SwitchEntryStmt n1, final Node arg) {
        SwitchEntryStmt n2 = (SwitchEntryStmt) arg;
        return !nodeEquals(n1.getLabel(), n2.getLabel()) ? false : nodesEquals(n1.getStmts(), n2.getStmts());
    }
    @Override public Boolean visit(final BreakStmt n1, final Node arg) {
        BreakStmt n2 = (BreakStmt) arg;
        return objEquals(n1.getId(), n2.getId());
    }
    @Override public Boolean visit(final ReturnStmt n1, final Node arg) {
        ReturnStmt n2 = (ReturnStmt) arg;
        return nodeEquals(n1.getExpr(), n2.getExpr());
    }
    @Override public Boolean visit(final IfStmt n1, final Node arg) {
        IfStmt n2 = (IfStmt) arg;
        return !nodeEquals(n1.getCondition(), n2.getCondition()) ? false : !nodeEquals(n1.getThenStmt(), n2.getThenStmt()) ? false : nodeEquals(n1.getElseStmt(), n2.getElseStmt());
    }
    @Override public Boolean visit(final WhileStmt n1, final Node arg) {
        WhileStmt n2 = (WhileStmt) arg;
        return !nodeEquals(n1.getCondition(), n2.getCondition()) ? false : nodeEquals(n1.getBody(), n2.getBody());
    }
    @Override public Boolean visit(final ContinueStmt n1, final Node arg) {
        ContinueStmt n2 = (ContinueStmt) arg;
        return objEquals(n1.getId(), n2.getId());
    }
    @Override public Boolean visit(final DoStmt n1, final Node arg) {
        DoStmt n2 = (DoStmt) arg;
        return !nodeEquals(n1.getBody(), n2.getBody()) ? false : nodeEquals(n1.getCondition(), n2.getCondition());
    }
    @Override public Boolean visit(final ForeachStmt n1, final Node arg) {
        ForeachStmt n2 = (ForeachStmt) arg;
        return !nodeEquals(n1.getVariable(), n2.getVariable()) ? false : !nodeEquals(n1.getIterable(), n2.getIterable()) ? false : nodeEquals(n1.getBody(), n2.getBody());
    }
    @Override public Boolean visit(final ForStmt n1, final Node arg) {
        ForStmt n2 = (ForStmt) arg;
        return !nodesEquals(n1.getInit(), n2.getInit()) ? false : !nodeEquals(n1.getCompare(), n2.getCompare()) ? false : !nodesEquals(n1.getUpdate(), n2.getUpdate()) ? false : nodeEquals(n1.getBody(), n2.getBody());
    }
    @Override public Boolean visit(final ThrowStmt n1, final Node arg) {
        ThrowStmt n2 = (ThrowStmt) arg;
        return nodeEquals(n1.getExpr(), n2.getExpr());
    }
    @Override public Boolean visit(final SynchronizedStmt n1, final Node arg) {
        SynchronizedStmt n2 = (SynchronizedStmt) arg;
        return !nodeEquals(n1.getExpr(), n2.getExpr()) ? false : nodeEquals(n1.getBody(), n2.getBody());
    }
    @Override public Boolean visit(final TryStmt n1, final Node arg) {
        TryStmt n2 = (TryStmt) arg;
        return !nodeEquals(n1.getTryBlock(), n2.getTryBlock()) ? false : !nodesEquals(n1.getCatchs(), n2.getCatchs()) ? false : !nodesEquals(n1.getResources(), n2.getResources()) ? false : nodeEquals(n1.getFinallyBlock(), n2.getFinallyBlock());
    }
    @Override public Boolean visit(final CatchClause n1, final Node arg) {
        CatchClause n2 = (CatchClause) arg;
        return !nodeEquals(n1.getParam(), n2.getParam()) ? false : nodeEquals(n1.getBody(), n2.getBody());
    }
    @Override
    public Boolean visit(LambdaExpr n1, Node arg) {
        LambdaExpr n2 = (LambdaExpr) arg;
        return !nodesEquals(n1.getParameters(), n2.getParameters()) ? false : n1.isParametersEnclosed() != n2.isParametersEnclosed() ? false : nodeEquals(n1.getBody(), n2.getBody());
    }
    @Override
    public Boolean visit(MethodReferenceExpr n1, Node arg) {
        MethodReferenceExpr n2 = (MethodReferenceExpr) arg;
        return !nodeEquals(n1.getScope(), n2.getScope()) ? false : !nodesEquals(n1.getTypeArguments(), n2.getTypeArguments()) ? false : objEquals(n1.getIdentifier(), n2.getIdentifier());
    }
    @Override
    public Boolean visit(TypeExpr n, Node arg) {
        TypeExpr n2 = (TypeExpr) arg;
        return nodeEquals(n.getType(), n2.getType());
    }
    @Override
	public Boolean visit(ArrayBracketPair n1, Node arg) {
        ArrayBracketPair n2 = (ArrayBracketPair) arg;
        return nodesEquals(n1.getAnnotations(), n2.getAnnotations());
    }
}
