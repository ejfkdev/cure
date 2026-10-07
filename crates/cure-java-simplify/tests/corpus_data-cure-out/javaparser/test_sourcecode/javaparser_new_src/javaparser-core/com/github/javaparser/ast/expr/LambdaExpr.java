package com.github.javaparser.ast.expr;

import com.github.javaparser.Range;
import com.github.javaparser.ast.body.Parameter;
import com.github.javaparser.ast.stmt.Statement;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;
import static com.github.javaparser.utils.Utils.*;

public class LambdaExpr extends Expression {
    private List<Parameter> parameters;
    private boolean parametersEnclosed;
    private Statement body;
    public LambdaExpr() {}
    public LambdaExpr(Range range, List<Parameter> parameters, Statement body, boolean parametersEnclosed) {
        super(range);
        setParameters(parameters);
        setBody(body);
        setParametersEnclosed(parametersEnclosed);
    }
    public List<Parameter> getParameters() {
        parameters = ensureNotNull(parameters);
        return parameters;
    }
    public LambdaExpr setParameters(List<Parameter> parameters) {
        this.parameters = parameters;
        setAsParentNodeOf(this.parameters);
        return this;
    }
    public Statement getBody() {
        return body;
    }
    public LambdaExpr setBody(Statement body) {
        this.body = body;
        setAsParentNodeOf(this.body);
        return this;
    }
    @Override
	public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
	public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public boolean isParametersEnclosed() {
        return parametersEnclosed;
    }
    public LambdaExpr setParametersEnclosed(boolean parametersEnclosed) {
        this.parametersEnclosed = parametersEnclosed;
        return this;
    }
}
