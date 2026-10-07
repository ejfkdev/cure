package com.github.javaparser.ast.nodeTypes;

import static com.github.javaparser.ast.type.VoidType.VOID_TYPE;
import static java.util.Collections.unmodifiableList;
import static java.util.stream.Collectors.toCollection;
import static java.util.stream.Collectors.toList;
import static java.util.stream.Collectors.toSet;
import java.util.Arrays;
import java.util.EnumSet;
import java.util.List;
import java.util.stream.Stream;
import com.github.javaparser.ast.Modifier;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.body.BodyDeclaration;
import com.github.javaparser.ast.body.ConstructorDeclaration;
import com.github.javaparser.ast.body.FieldDeclaration;
import com.github.javaparser.ast.body.InitializerDeclaration;
import com.github.javaparser.ast.body.MethodDeclaration;
import com.github.javaparser.ast.body.TypeDeclaration;
import com.github.javaparser.ast.body.VariableDeclarator;
import com.github.javaparser.ast.body.VariableDeclaratorId;
import com.github.javaparser.ast.stmt.BlockStmt;
import com.github.javaparser.ast.type.ClassOrInterfaceType;
import com.github.javaparser.ast.type.Type;

public interface NodeWithMembers<T> {
    List<BodyDeclaration<?>> getMembers();
    T setMembers(List<BodyDeclaration<?>> members);
    default FieldDeclaration addField(Class<?> typeClass, String name, Modifier... modifiers) {
        ((Node) this).tryAddImportToParentCompilationUnit(typeClass);
        return addField(typeClass.getSimpleName(), name, modifiers);
    }
    default FieldDeclaration addField(String type, String name, Modifier... modifiers) {
        return addField(new ClassOrInterfaceType(type), name, modifiers);
    }
    default FieldDeclaration addField(Type<?> type, String name, Modifier... modifiers) {
        FieldDeclaration fieldDeclaration = new FieldDeclaration();
        fieldDeclaration.setParentNode((Node) this);
        VariableDeclarator variable = new VariableDeclarator(new VariableDeclaratorId(name));
        fieldDeclaration.getVariables().add(variable);
        variable.setParentNode(fieldDeclaration);
        fieldDeclaration.setModifiers(Arrays.stream(modifiers).collect(toCollection(() -> EnumSet.noneOf(Modifier.class))));
        variable.setType(type);
        getMembers().add(fieldDeclaration);
        return fieldDeclaration;
    }
    default FieldDeclaration addPrivateField(Class<?> typeClass, String name) {
        return addField(typeClass, name, Modifier.PRIVATE);
    }
    default FieldDeclaration addPrivateField(String type, String name) {
        return addField(type, name, Modifier.PRIVATE);
    }
    default FieldDeclaration addPublicField(Class<?> typeClass, String name) {
        return addField(typeClass, name, Modifier.PUBLIC);
    }
    default FieldDeclaration addPublicField(String type, String name) {
        return addField(type, name, Modifier.PUBLIC);
    }
    default FieldDeclaration addProtectedField(Class<?> typeClass, String name) {
        return addField(typeClass, name, Modifier.PROTECTED);
    }
    default FieldDeclaration addProtectedField(String type, String name) {
        return addField(type, name, Modifier.PROTECTED);
    }
    default MethodDeclaration addMethod(String methodName, Modifier... modifiers) {
        MethodDeclaration methodDeclaration = new MethodDeclaration();
        methodDeclaration.setName(methodName);
        methodDeclaration.setType(VOID_TYPE);
        methodDeclaration.setModifiers(Arrays.stream(modifiers).collect(toCollection(() -> EnumSet.noneOf(Modifier.class))));
        getMembers().add(methodDeclaration);
        methodDeclaration.setParentNode((Node) this);
        return methodDeclaration;
    }
    default ConstructorDeclaration addCtor(Modifier... modifiers) {
        ConstructorDeclaration constructorDeclaration = new ConstructorDeclaration();
        constructorDeclaration.setModifiers(Arrays.stream(modifiers).collect(toCollection(() -> EnumSet.noneOf(Modifier.class))));
        constructorDeclaration.setName(((TypeDeclaration<?>) this).getName());
        getMembers().add(constructorDeclaration);
        constructorDeclaration.setParentNode((Node) this);
        return constructorDeclaration;
    }
    default BlockStmt addInitializer() {
        BlockStmt block = new BlockStmt();
        InitializerDeclaration initializerDeclaration = new InitializerDeclaration(false, block);
        getMembers().add(initializerDeclaration);
        initializerDeclaration.setParentNode((Node) this);
        return block;
    }
    default BlockStmt addStaticInitializer() {
        BlockStmt block = new BlockStmt();
        InitializerDeclaration initializerDeclaration = new InitializerDeclaration(true, block);
        getMembers().add(initializerDeclaration);
        initializerDeclaration.setParentNode((Node) this);
        return block;
    }
    default List<MethodDeclaration> getMethodsByName(String name) {
        return getMembers().stream().filter((m) -> m instanceof MethodDeclaration && ((MethodDeclaration) m).getName().equals(name)).map((m) -> (MethodDeclaration) m).collect(toList());
    }
    default List<MethodDeclaration> getMethods() {
        return unmodifiableList(getMembers().stream().filter((m) -> m instanceof MethodDeclaration).map((m) -> (MethodDeclaration) m).collect(toList()));
    }
    default List<MethodDeclaration> getMethodsByParameterTypes(String... paramTypes) {
        return getMembers().stream().filter((m) -> m instanceof MethodDeclaration && ((MethodDeclaration) m).getParameters().stream().map((p) -> p.getType().toString()).collect(toSet()).equals(Stream.of(paramTypes).collect(toSet()))).map((m) -> (MethodDeclaration) m).collect(toList());
    }
    default List<MethodDeclaration> getMethodsByParameterTypes(Class<?>... paramTypes) {
        return getMembers().stream().filter((m) -> m instanceof MethodDeclaration && ((MethodDeclaration) m).getParameters().stream().map((p) -> p.getType().toString()).collect(toSet()).equals(Stream.of(paramTypes).map(Class::getSimpleName).collect(toSet()))).map((m) -> (MethodDeclaration) m).collect(toList());
    }
    default FieldDeclaration getFieldByName(String name) {
        return (FieldDeclaration) getMembers().stream().filter((m) -> m instanceof FieldDeclaration && ((FieldDeclaration) m).getVariables().stream().anyMatch((var) -> var.getId().getName().equals(name))).findFirst().orElse(null);
    }
    default List<FieldDeclaration> getFields() {
        return unmodifiableList(getMembers().stream().filter((m) -> m instanceof FieldDeclaration).map((m) -> (FieldDeclaration) m).collect(toList()));
    }
}
