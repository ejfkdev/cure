package com.github.javaparser.ast.type;

import com.github.javaparser.Range;

public abstract class ReferenceType<T extends ReferenceType> extends Type<T> {
    public ReferenceType() {}
    public ReferenceType(final Range range) {
        super(range);
    }
}
