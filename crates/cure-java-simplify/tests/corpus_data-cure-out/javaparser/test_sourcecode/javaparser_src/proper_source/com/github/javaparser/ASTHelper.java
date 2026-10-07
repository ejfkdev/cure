package com.github.javaparser;

import com.github.javaparser.ast.CompilationUnit;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.body.BodyDeclaration;
import com.github.javaparser.ast.body.FieldDeclaration;
import com.github.javaparser.ast.body.MethodDeclaration;
import com.github.javaparser.ast.body.Parameter;
import com.github.javaparser.ast.body.TypeDeclaration;
import com.github.javaparser.ast.body.VariableDeclarator;
import com.github.javaparser.ast.body.VariableDeclaratorId;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.expr.MethodCallExpr;
import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.ast.expr.QualifiedNameExpr;
import com.github.javaparser.ast.expr.VariableDeclarationExpr;
import com.github.javaparser.ast.stmt.BlockStmt;
import com.github.javaparser.ast.stmt.ExpressionStmt;
import com.github.javaparser.ast.stmt.Statement;
import com.github.javaparser.ast.type.ClassOrInterfaceType;
import com.github.javaparser.ast.type.PrimitiveType;
import com.github.javaparser.ast.type.ReferenceType;
import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.type.VoidType;
import com.github.javaparser.ast.type.PrimitiveType.Primitive;
import java.util.ArrayList;
import java.util.List;

public final class ASTHelper {
    public static final PrimitiveType BYTE_TYPE = new PrimitiveType(Primitive.Byte);
    public static final PrimitiveType SHORT_TYPE = new PrimitiveType(Primitive.Short);
    public static final PrimitiveType INT_TYPE = new PrimitiveType(Primitive.Int);
    public static final PrimitiveType LONG_TYPE = new PrimitiveType(Primitive.Long);
    public static final PrimitiveType FLOAT_TYPE = new PrimitiveType(Primitive.Float);
    public static final PrimitiveType DOUBLE_TYPE = new PrimitiveType(Primitive.Double);
    public static final PrimitiveType BOOLEAN_TYPE = new PrimitiveType(Primitive.Boolean);
    public static final PrimitiveType CHAR_TYPE = new PrimitiveType(Primitive.Char);
    public static final VoidType VOID_TYPE = new VoidType();
    private ASTHelper() {}
    public static NameExpr createNameExpr(String qualifiedName) {
        String[] split = qualifiedName.split("\\.");
        NameExpr ret = new NameExpr(split[0]);
        for (int i = 1; i < split.length; i++) {
            ret = new QualifiedNameExpr(ret, split[i]);
        }
        return ret;
    }
    public static Parameter createParameter(Type type, String name) {
        return new Parameter(type, new VariableDeclaratorId(name));
    }
    public static FieldDeclaration createFieldDeclaration(int modifiers, Type type, VariableDeclarator variable) {
        List<VariableDeclarator> variables = new ArrayList<VariableDeclarator>();
        variables.add(variable);
        return new FieldDeclaration(modifiers, type, variables);
    }
    public static FieldDeclaration createFieldDeclaration(int modifiers, Type type, String name) {
        return createFieldDeclaration(modifiers, type, new VariableDeclarator(new VariableDeclaratorId(name)));
    }
    public static VariableDeclarationExpr createVariableDeclarationExpr(Type type, String name) {
        List<VariableDeclarator> vars = new ArrayList<VariableDeclarator>();
        vars.add(new VariableDeclarator(new VariableDeclaratorId(name)));
        return new VariableDeclarationExpr(type, vars);
    }
    public static void addParameter(MethodDeclaration method, Parameter parameter) {
        List<Parameter> parameters = method.getParameters();
        if (parameters == null) {
            parameters = new ArrayList<Parameter>();
            method.setParameters(parameters);
        }
        parameters.add(parameter);
    }
    public static void addArgument(MethodCallExpr call, Expression arg) {
        List<Expression> args = call.getArgs();
        if (args == null) {
            args = new ArrayList<Expression>();
            call.setArgs(args);
        }
        args.add(arg);
    }
    public static void addTypeDeclaration(CompilationUnit cu, TypeDeclaration type) {
        List<TypeDeclaration> types = cu.getTypes();
        if (types == null) {
            types = new ArrayList<TypeDeclaration>();
            cu.setTypes(types);
        }
        types.add(type);
    }
    public static ReferenceType createReferenceType(String name, int arrayCount) {
        return new ReferenceType(new ClassOrInterfaceType(name), arrayCount);
    }
    public static ReferenceType createReferenceType(PrimitiveType type, int arrayCount) {
        return new ReferenceType(type, arrayCount);
    }
    public static void addStmt(BlockStmt block, Statement stmt) {
        List<Statement> stmts = block.getStmts();
        if (stmts == null) {
            stmts = new ArrayList<Statement>();
            block.setStmts(stmts);
        }
        stmts.add(stmt);
    }
    public static void addStmt(BlockStmt block, Expression expr) {
        addStmt(block, new ExpressionStmt(expr));
    }
    public static void addMember(TypeDeclaration type, BodyDeclaration decl) {
        List<BodyDeclaration> members = type.getMembers();
        if (members == null) {
            members = new ArrayList<BodyDeclaration>();
            type.setMembers(members);
        }
        members.add(decl);
    }
    public static <N extends Node> List<N> getNodesByType(Node container, Class<N> clazz) {
        List<N> nodes = new ArrayList<N>();
        for (Node child : container.getChildrenNodes()) {
            if (clazz.isInstance(child)) {
                nodes.add(clazz.cast(child));
            }
            nodes.addAll(getNodesByType(child, clazz));
        }
        return nodes;
    }
}
