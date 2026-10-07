package com.github.javaparser.ast.type;

import com.github.javaparser.Range;
import com.github.javaparser.ast.ArrayBracketPair;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.nodeTypes.NodeWithAnnotations;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import com.github.javaparser.utils.Pair;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

public class ArrayType extends ReferenceType<ArrayType> implements NodeWithAnnotations<ArrayType> {
    private Type componentType;
    public ArrayType(Type componentType, List<AnnotationExpr> annotations) {
        setComponentType(componentType);
        setAnnotations(annotations);
    }
    public ArrayType(Range range, Type componentType, List<AnnotationExpr> annotations) {
        super(range);
        setComponentType(componentType);
        setAnnotations(annotations);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public Type getComponentType() {
        return componentType;
    }
    public ArrayType setComponentType(final Type type) {
        this.componentType = type;
        setAsParentNodeOf(this.componentType);
        return this;
    }
    @SafeVarargs
    public static Type wrapInArrayTypes(Type type, List<ArrayBracketPair>... arrayBracketPairLists) {
        for (int i = arrayBracketPairLists.length - 1; i >= 0; i--) {
            List<ArrayBracketPair> arrayBracketPairList = arrayBracketPairLists[i];
            if (arrayBracketPairList != null) {
                for (int j = arrayBracketPairList.size() - 1; j >= 0; j--) {
                    type = new ArrayType(type, arrayBracketPairList.get(j).getAnnotations());
                }
            }
        }
        return type;
    }
    public static Pair<Type, List<ArrayBracketPair>> unwrapArrayTypes(Type type) {
        List<ArrayBracketPair> arrayBracketPairs = new ArrayList<>();
        while (type instanceof ArrayType) {
            ArrayType arrayType = (ArrayType) type;
            arrayBracketPairs.add(new ArrayBracketPair(Range.UNKNOWN, arrayType.getAnnotations()));
            type = arrayType.getComponentType();
        }
        return new Pair<>(type, arrayBracketPairs);
    }
    public static ArrayType arrayOf(Type type, AnnotationExpr... annotations) {
        return new ArrayType(type, Arrays.asList(annotations));
    }
}
