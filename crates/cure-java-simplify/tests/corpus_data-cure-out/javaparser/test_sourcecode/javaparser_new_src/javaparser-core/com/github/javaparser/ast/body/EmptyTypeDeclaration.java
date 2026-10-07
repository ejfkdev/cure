package com.github.javaparser.ast.body;

import java.util.EnumSet;
import com.github.javaparser.Range;
import com.github.javaparser.ast.Modifier;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;

public final class EmptyTypeDeclaration extends TypeDeclaration<EmptyTypeDeclaration> {
    public EmptyTypeDeclaration() {
        super(null, EnumSet.noneOf(Modifier.class), null, null);
    }
    public EmptyTypeDeclaration(Range range) {
        super(range, null, EnumSet.noneOf(Modifier.class), null, null);
    }
    @Override
    public <R, A> R accept(GenericVisitor<R, A> v, A arg) {
        return v.visit(this, arg);
    }
    @Override
    public <A> void accept(VoidVisitor<A> v, A arg) {
        v.visit(this, arg);
    }
}
