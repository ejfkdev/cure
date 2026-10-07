package com.github.javaparser.symbolsolver.model.resolution;

import com.github.javaparser.symbolsolver.model.declarations.Declaration;
import java.util.Optional;

public class SymbolReference<S extends Declaration> {
    private Optional<? extends S> correspondingDeclaration;
    private SymbolReference(Optional<? extends S> correspondingDeclaration) {
        this.correspondingDeclaration = correspondingDeclaration;
    }
    public static <S extends Declaration, S2 extends S> SymbolReference<S> solved(S2 symbolDeclaration) {
        return new SymbolReference<S>(Optional.of(symbolDeclaration));
    }
    public static <S extends Declaration, S2 extends S> SymbolReference<S> unsolved(Class<S2> clazz) {
        return new SymbolReference<S>(Optional.empty());
    }
    @Override
    public String toString() {
        return "SymbolReference{" + correspondingDeclaration + "}";
    }
    public S getCorrespondingDeclaration() {
        if (!isSolved()) {
            throw new UnsupportedOperationException();
        }
        return correspondingDeclaration.get();
    }
    public boolean isSolved() {
        return correspondingDeclaration.isPresent();
    }
    public static <O extends Declaration> SymbolReference<O> adapt(SymbolReference<? extends O> ref, Class<O> clazz) {
        return ref.isSolved() ? SymbolReference.solved(ref.getCorrespondingDeclaration()) : SymbolReference.unsolved(clazz);
    }
}
