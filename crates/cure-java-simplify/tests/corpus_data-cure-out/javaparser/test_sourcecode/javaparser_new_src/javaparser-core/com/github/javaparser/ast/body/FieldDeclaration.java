package com.github.javaparser.ast.body;

import static com.github.javaparser.ast.expr.NameExpr.*;
import static com.github.javaparser.utils.Utils.ensureNotNull;
import java.util.ArrayList;
import java.util.EnumSet;
import java.util.List;
import com.github.javaparser.Range;
import com.github.javaparser.ast.ArrayBracketPair;
import com.github.javaparser.ast.Modifier;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.expr.AssignExpr;
import com.github.javaparser.ast.expr.AssignExpr.Operator;
import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.ast.nodeTypes.*;
import com.github.javaparser.ast.stmt.BlockStmt;
import com.github.javaparser.ast.stmt.ReturnStmt;
import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import static com.github.javaparser.ast.Modifier.PUBLIC;
import static com.github.javaparser.ast.type.VoidType.VOID_TYPE;

public final class FieldDeclaration extends BodyDeclaration<FieldDeclaration> implements NodeWithJavaDoc<FieldDeclaration>, NodeWithElementType<FieldDeclaration>, NodeWithModifiers<FieldDeclaration>, NodeWithVariables<FieldDeclaration> {
    private EnumSet<Modifier> modifiers = EnumSet.noneOf(Modifier.class);
    private Type elementType;
    private List<VariableDeclarator> variables;
    private List<ArrayBracketPair> arrayBracketPairsAfterElementType;
    public FieldDeclaration() {}
    public FieldDeclaration(EnumSet<Modifier> modifiers, Type elementType, VariableDeclarator variable) {
        setModifiers(modifiers);
        setElementType(elementType);
        List<VariableDeclarator> aux = new ArrayList<>();
        aux.add(variable);
        setVariables(aux);
    }
    public FieldDeclaration(EnumSet<Modifier> modifiers, Type elementType, List<VariableDeclarator> variables) {
        setModifiers(modifiers);
        setElementType(elementType);
        setVariables(variables);
    }
    public FieldDeclaration(EnumSet<Modifier> modifiers, List<AnnotationExpr> annotations, Type elementType, List<ArrayBracketPair> arrayBracketPairsAfterElementType, List<VariableDeclarator> variables) {
        super(annotations);
        setModifiers(modifiers);
        setElementType(elementType);
        setVariables(variables);
        setArrayBracketPairsAfterElementType(arrayBracketPairsAfterElementType);
    }
    public FieldDeclaration(Range range, EnumSet<Modifier> modifiers, List<AnnotationExpr> annotations, Type elementType, List<VariableDeclarator> variables, List<ArrayBracketPair> arrayBracketPairsAfterElementType) {
        super(range, annotations);
        setModifiers(modifiers);
        setElementType(elementType);
        setVariables(variables);
        setArrayBracketPairsAfterElementType(arrayBracketPairsAfterElementType);
    }
    public static FieldDeclaration create(EnumSet<Modifier> modifiers, Type type, VariableDeclarator variable) {
        List<VariableDeclarator> variables = new ArrayList<>();
        variables.add(variable);
        return new FieldDeclaration(modifiers, type, variables);
    }
    public static FieldDeclaration create(EnumSet<Modifier> modifiers, Type type, String name) {
        return create(modifiers, type, new VariableDeclarator(new VariableDeclaratorId(name)));
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    @Override
    public EnumSet<Modifier> getModifiers() {
        return modifiers;
    }
    @Override
    public List<VariableDeclarator> getVariables() {
        variables = ensureNotNull(variables);
        return variables;
    }
    @Override
    public FieldDeclaration setModifiers(EnumSet<Modifier> modifiers) {
        this.modifiers = modifiers;
        return this;
    }
    @Override
    public FieldDeclaration setVariables(List<VariableDeclarator> variables) {
        this.variables = variables;
        setAsParentNodeOf(this.variables);
        return this;
    }
    @Override
    public JavadocComment getJavaDoc() {
        return getComment() instanceof JavadocComment ? (JavadocComment) getComment() : null;
    }
    public MethodDeclaration createGetter() {
        if (getVariables().size() != 1) 
            throw new IllegalStateException("You can use this only when the field declares only 1 variable name");
        ClassOrInterfaceDeclaration parentClass = getParentNodeOfType(ClassOrInterfaceDeclaration.class);
        EnumDeclaration parentEnum = getParentNodeOfType(EnumDeclaration.class);
        if (parentClass == null && parentEnum == null || parentClass != null && parentClass.isInterface()) 
            throw new IllegalStateException("You can use this only when the field is attached to a class or an enum");
        VariableDeclarator variable = getVariables().get(0);
        String fieldName = variable.getId().getName();
        String fieldNameUpper = fieldName.toUpperCase().substring(0, 1) + fieldName.substring(1, fieldName.length());
        MethodDeclaration getter = parentClass != null ? parentClass.addMethod("get" + fieldNameUpper, PUBLIC) : parentEnum.addMethod("get" + fieldNameUpper, PUBLIC);
        getter.setType(variable.getType());
        BlockStmt blockStmt = new BlockStmt();
        getter.setBody(blockStmt);
        blockStmt.addStatement(new ReturnStmt(name(fieldName)));
        return getter;
    }
    public MethodDeclaration createSetter() {
        if (getVariables().size() != 1) 
            throw new IllegalStateException("You can use this only when the field declares only 1 variable name");
        ClassOrInterfaceDeclaration parentClass = getParentNodeOfType(ClassOrInterfaceDeclaration.class);
        EnumDeclaration parentEnum = getParentNodeOfType(EnumDeclaration.class);
        if (parentClass == null && parentEnum == null || parentClass != null && parentClass.isInterface()) 
            throw new IllegalStateException("You can use this only when the field is attached to a class or an enum");
        VariableDeclarator variable = getVariables().get(0);
        String fieldName = variable.getId().getName();
        String fieldNameUpper = fieldName.toUpperCase().substring(0, 1) + fieldName.substring(1, fieldName.length());
        MethodDeclaration setter = parentClass != null ? parentClass.addMethod("set" + fieldNameUpper, PUBLIC) : parentEnum.addMethod("set" + fieldNameUpper, PUBLIC);
        setter.setType(VOID_TYPE);
        setter.getParameters().add(new Parameter(variable.getType(), new VariableDeclaratorId(fieldName)));
        BlockStmt blockStmt2 = new BlockStmt();
        setter.setBody(blockStmt2);
        blockStmt2.addStatement(new AssignExpr(new NameExpr("this." + fieldName), new NameExpr(fieldName), Operator.assign));
        return setter;
    }
    @Override
    public Type getElementType() {
        return elementType;
    }
    @Override
    public FieldDeclaration setElementType(final Type elementType) {
        this.elementType = elementType;
        setAsParentNodeOf(this.elementType);
        return this;
    }
    public List<ArrayBracketPair> getArrayBracketPairsAfterElementType() {
        arrayBracketPairsAfterElementType = ensureNotNull(arrayBracketPairsAfterElementType);
        return arrayBracketPairsAfterElementType;
    }
    @Override
    public FieldDeclaration setArrayBracketPairsAfterElementType(List<ArrayBracketPair> arrayBracketPairsAfterType) {
        this.arrayBracketPairsAfterElementType = arrayBracketPairsAfterType;
        setAsParentNodeOf(arrayBracketPairsAfterType);
        return this;
    }
}
