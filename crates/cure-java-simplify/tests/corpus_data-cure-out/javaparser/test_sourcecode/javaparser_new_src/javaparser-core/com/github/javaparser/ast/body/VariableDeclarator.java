package com.github.javaparser.ast.body;

import com.github.javaparser.Range;
import com.github.javaparser.ast.ArrayBracketPair;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.ast.nodeTypes.NodeWithElementType;
import com.github.javaparser.ast.nodeTypes.NodeWithType;
import com.github.javaparser.ast.type.ArrayType;
import com.github.javaparser.ast.type.Type;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import com.github.javaparser.utils.Pair;
import java.util.List;
import static com.github.javaparser.ast.type.ArrayType.wrapInArrayTypes;

public final class VariableDeclarator extends Node implements NodeWithType<VariableDeclarator> {
    private VariableDeclaratorId id;
    private Expression init;
    public VariableDeclarator() {}
    public VariableDeclarator(VariableDeclaratorId id) {
        setId(id);
    }
    public VariableDeclarator(String variableName) {
        setId(new VariableDeclaratorId(variableName));
    }
    public VariableDeclarator(VariableDeclaratorId id, Expression init) {
        setId(id);
        setInit(init);
    }
    public VariableDeclarator(String variableName, Expression init) {
        setId(new VariableDeclaratorId(variableName));
        setInit(init);
    }
    public VariableDeclarator(Range range, VariableDeclaratorId id, Expression init) {
        super(range);
        setId(id);
        setInit(init);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
    public VariableDeclaratorId getId() {
        return id;
    }
    public Expression getInit() {
        return init;
    }
    public VariableDeclarator setId(VariableDeclaratorId id) {
        this.id = id;
        setAsParentNodeOf(this.id);
        return this;
    }
    public VariableDeclarator setInit(Expression init) {
        this.init = init;
        setAsParentNodeOf(this.init);
        return this;
    }
    public VariableDeclarator setInit(String init) {
        this.init = new NameExpr(init);
        setAsParentNodeOf(this.init);
        return this;
    }
    @Override
    public Type getType() {
        NodeWithElementType<?> elementType = getParentNodeOfType(NodeWithElementType.class);
        return wrapInArrayTypes(elementType.getElementType(), elementType.getArrayBracketPairsAfterElementType(), getId().getArrayBracketPairsAfterId());
    }
    @Override
    public VariableDeclarator setType(Type type) {
        Pair<Type, List<ArrayBracketPair>> unwrapped = ArrayType.unwrapArrayTypes(type);
        NodeWithElementType<?> nodeWithElementType = getParentNodeOfType(NodeWithElementType.class);
        if (nodeWithElementType == null) {
            throw new IllegalStateException("Cannot set type without a parent");
        }
        nodeWithElementType.setElementType(unwrapped.a);
        nodeWithElementType.setArrayBracketPairsAfterElementType(null);
        getId().setArrayBracketPairsAfterId(unwrapped.b);
        return this;
    }
}
