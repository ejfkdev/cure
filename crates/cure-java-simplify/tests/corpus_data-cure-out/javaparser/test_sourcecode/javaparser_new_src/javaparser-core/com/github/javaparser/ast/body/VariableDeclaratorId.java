package com.github.javaparser.ast.body;

import com.github.javaparser.Range;
import com.github.javaparser.ast.ArrayBracketPair;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.nodeTypes.NodeWithName;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import java.util.List;
import static com.github.javaparser.utils.Utils.ensureNotNull;

public final class VariableDeclaratorId extends Node implements NodeWithName<VariableDeclaratorId> {
    private String name;
    private List<ArrayBracketPair> arrayBracketPairsAfterId;
    public VariableDeclaratorId() {}
    public VariableDeclaratorId(String name) {
        setName(name);
    }
    public VariableDeclaratorId(Range range, String name, List<ArrayBracketPair> arrayBracketPairsAfterId) {
        super(range);
        setName(name);
        setArrayBracketPairsAfterId(arrayBracketPairsAfterId);
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
    public String getName() {
        return name;
    }
    @Override
    public VariableDeclaratorId setName(String name) {
        this.name = name;
        return this;
    }
    public List<ArrayBracketPair> getArrayBracketPairsAfterId() {
        arrayBracketPairsAfterId = ensureNotNull(arrayBracketPairsAfterId);
        return arrayBracketPairsAfterId;
    }
    public VariableDeclaratorId setArrayBracketPairsAfterId(List<ArrayBracketPair> arrayBracketPairsAfterId) {
        this.arrayBracketPairsAfterId = arrayBracketPairsAfterId;
        setAsParentNodeOf(arrayBracketPairsAfterId);
        return this;
    }
}
