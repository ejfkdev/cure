package com.github.javaparser.symbolsolver.resolution;

import com.github.javaparser.symbolsolver.model.declarations.ValueDeclaration;
import java.util.List;

public interface SymbolDeclarator {
    List<ValueDeclaration> getSymbolDeclarations();
}
