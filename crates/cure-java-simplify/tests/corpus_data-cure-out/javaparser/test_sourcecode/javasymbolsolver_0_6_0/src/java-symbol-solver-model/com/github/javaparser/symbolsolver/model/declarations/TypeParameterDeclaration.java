package com.github.javaparser.symbolsolver.model.declarations;

import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.typesystem.Type;
import java.util.List;
import java.util.Optional;

public interface TypeParameterDeclaration extends TypeDeclaration {
    static TypeParameterDeclaration onType(final String name, String classQName, List<Bound> bounds) {
        return new TypeParameterDeclaration() {
            @Override
            public String getName() {
                return name;
            }

            @Override
            public boolean declaredOnType() {
                return true;
            }

            @Override
            public boolean declaredOnMethod() {
                return false;
            }

            @Override
            public boolean declaredOnConstructor() {
                return false;
            }

            @Override
            public String getContainerQualifiedName() {
                return classQName;
            }

            @Override
            public String getContainerId() {
                return classQName;
            }
            
            @Override
            public TypeParametrizable getContainer() {
                return null;
            }

            @Override
            public List<Bound> getBounds(TypeSolver typeSolver) {
                return bounds;
            }

            @Override
            public String toString() {
                return "TypeParameter onType " + name;
            }

            @Override
            public Optional<ReferenceTypeDeclaration> containerType() {
                throw new UnsupportedOperationException();
            }
        };
    }
    String getName();
    default boolean declaredOnType() {
        return getContainer() instanceof ReferenceTypeDeclaration;
    }
    default boolean declaredOnMethod() {
        return getContainer() instanceof MethodDeclaration;
    }
    default boolean declaredOnConstructor() {
        return getContainer() instanceof ConstructorDeclaration;
    }
    default String getPackageName() {
        throw new UnsupportedOperationException();
    }
    default String getClassName() {
        throw new UnsupportedOperationException();
    }
    default String getQualifiedName() {
        return String.format("%s.%s", getContainerId(), getName());
    }
    String getContainerQualifiedName();
    String getContainerId();
    TypeParametrizable getContainer();
    List<Bound> getBounds(TypeSolver typeSolver);
    class Bound {
        private boolean extendsBound;
        private Type type;
        private Bound(boolean extendsBound, Type type) {
            this.extendsBound = extendsBound;
            this.type = type;
        }
        public static Bound extendsBound(Type type) {
            return new Bound(true, type);
        }
        public static Bound superBound(Type type) {
            return new Bound(false, type);
        }
        public Type getType() {
            return type;
        }
        public boolean isExtends() {
            return extendsBound;
        }
        public boolean isSuper() {
            return !isExtends();
        }
    }
}
