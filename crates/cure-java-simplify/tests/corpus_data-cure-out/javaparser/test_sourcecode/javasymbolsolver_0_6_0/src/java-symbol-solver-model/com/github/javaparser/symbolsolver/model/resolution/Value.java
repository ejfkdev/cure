package com.github.javaparser.symbolsolver.model.resolution;

import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import com.github.javaparser.symbolsolver.model.typesystem.Type;

public class Value {
    private Type type;
    private String name;
    public Value(Type type, String name) {
        this.type = type;
        this.name = name;
    }
    public static Value from(ValueDeclaration decl) {
        return new Value(decl.getType(), decl.getName());
    }
    @Override
    public String toString() {
        return "Value{typeUsage=" + type + ", name='" + name + '\'' + '}';
    }
    public String getName() {
        return name;
    }
    public Type getType() {
        return type;
    }
}
