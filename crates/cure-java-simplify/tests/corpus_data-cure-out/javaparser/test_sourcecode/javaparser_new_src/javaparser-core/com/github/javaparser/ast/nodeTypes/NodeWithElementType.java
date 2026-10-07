package com.github.javaparser.ast.nodeTypes;

import com.github.javaparser.ast.ArrayBracketPair;
import com.github.javaparser.ast.CompilationUnit;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.body.MethodDeclaration;
import com.github.javaparser.ast.type.ClassOrInterfaceType;
import com.github.javaparser.ast.type.Type;
import java.util.List;

public interface NodeWithElementType<T> {
    Type<?> getElementType();
    T setElementType(Type<?> elementType);
    List<ArrayBracketPair> getArrayBracketPairsAfterElementType();
    T setArrayBracketPairsAfterElementType(List<ArrayBracketPair> arrayBracketPairsAfterType);
    default T setElementType(Class<?> typeClass) {
        ((Node) this).tryAddImportToParentCompilationUnit(typeClass);
        return setElementType(new ClassOrInterfaceType(typeClass.getSimpleName()));
    }
    default T setElementType(final String type) {
        return setElementType(new ClassOrInterfaceType(type));
    }
}
