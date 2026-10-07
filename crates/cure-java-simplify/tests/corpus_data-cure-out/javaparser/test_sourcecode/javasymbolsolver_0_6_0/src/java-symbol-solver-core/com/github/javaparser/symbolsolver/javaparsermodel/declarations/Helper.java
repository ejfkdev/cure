package com.github.javaparser.symbolsolver.javaparsermodel.declarations;

import com.github.javaparser.ast.CompilationUnit;
import com.github.javaparser.ast.Modifier;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.PackageDeclaration;
import com.github.javaparser.symbolsolver.model.declarations.AccessLevel;
import java.util.EnumSet;
import java.util.Optional;
import static com.github.javaparser.symbolsolver.javaparser.Navigator.getParentNode;

class Helper {
    public static AccessLevel toAccessLevel(EnumSet<Modifier> modifiers) {
        return modifiers.contains(Modifier.PRIVATE) ? AccessLevel.PRIVATE : modifiers.contains(Modifier.PROTECTED) ? AccessLevel.PROTECTED : modifiers.contains(Modifier.PUBLIC) ? AccessLevel.PUBLIC : AccessLevel.PACKAGE_PROTECTED;
    }
    static String containerName(Node container) {
        String packageName = getPackageName(container);
        String className = getClassName("", container);
        return packageName + (!packageName.isEmpty() && !className.isEmpty() ? "." : "") + className;
    }
    static String getPackageName(Node container) {
        if (container instanceof CompilationUnit) {
            Optional<PackageDeclaration> p = ((CompilationUnit) container).getPackageDeclaration();
            if (p.isPresent()) {
                return p.get().getName().toString();
            }
        } else if (container != null) {
            return getPackageName(getParentNode(container));
        }
        return "";
    }
    static String getClassName(String base, Node container) {
        if (container instanceof com.github.javaparser.ast.body.ClassOrInterfaceDeclaration) {
            String b = getClassName(base, getParentNode(container));
            String cn = ((com.github.javaparser.ast.body.ClassOrInterfaceDeclaration) container).getName().getId();
            return b.isEmpty() ? cn : b + "." + cn;
        } else if (container instanceof com.github.javaparser.ast.body.EnumDeclaration) {
            String b = getClassName(base, getParentNode(container));
            String cn = ((com.github.javaparser.ast.body.EnumDeclaration) container).getName().getId();
            return b.isEmpty() ? cn : b + "." + cn;
        } else if (container != null) {
            return getClassName(base, getParentNode(container));
        }
        return base;
    }
}
