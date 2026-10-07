package com.github.javaparser.ast.body;

import static com.github.javaparser.ast.expr.NameExpr.*;
import static com.github.javaparser.utils.Utils.ensureNotNull;
import java.util.List;
import com.github.javaparser.Range;
import com.github.javaparser.ast.comments.JavadocComment;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.ast.nodeTypes.NodeWithJavaDoc;
import com.github.javaparser.ast.nodeTypes.NodeWithName;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;
import static com.github.javaparser.utils.Utils.ensureNotNull;

public final class EnumConstantDeclaration extends BodyDeclaration<EnumConstantDeclaration> implements NodeWithJavaDoc<EnumConstantDeclaration>, NodeWithName<EnumConstantDeclaration> {
    private String name;
    private List<Expression> args;
    private List<BodyDeclaration<?>> classBody;
    public EnumConstantDeclaration() {}
    public EnumConstantDeclaration(String name) {
        setName(name);
    }
    public EnumConstantDeclaration(List<AnnotationExpr> annotations, String name, List<Expression> args, List<BodyDeclaration<?>> classBody) {
        super(annotations);
        setName(name);
        setArgs(args);
        setClassBody(classBody);
    }
    public EnumConstantDeclaration(Range range, List<AnnotationExpr> annotations, String name, List<Expression> args, List<BodyDeclaration<?>> classBody) {
        super(range, annotations);
        setName(name);
        setArgs(args);
        setClassBody(classBody);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public List<Expression> getArgs() {
        args = ensureNotNull(args);
        return args;
    }
    public List<BodyDeclaration<?>> getClassBody() {
        classBody = ensureNotNull(classBody);
        return classBody;
    }
    @Override
    public String getName() {
        return name;
    }
    public EnumConstantDeclaration setArgs(List<Expression> args) {
        this.args = args;
        setAsParentNodeOf(this.args);
        return this;
    }
    public EnumConstantDeclaration setClassBody(List<BodyDeclaration<?>> classBody) {
        this.classBody = classBody;
        setAsParentNodeOf(this.classBody);
        return this;
    }
    @Override
    public EnumConstantDeclaration setName(String name) {
        this.name = name;
        return this;
    }
    @Override
    public JavadocComment getJavaDoc() {
        return getComment() instanceof JavadocComment ? (JavadocComment) getComment() : null;
    }
    public EnumConstantDeclaration addArgument(String valueExpr) {
        getArgs().add(name(valueExpr));
        return this;
    }
}
