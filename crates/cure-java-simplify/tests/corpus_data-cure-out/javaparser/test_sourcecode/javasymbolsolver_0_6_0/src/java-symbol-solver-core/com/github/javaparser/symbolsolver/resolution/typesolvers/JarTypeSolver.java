package com.github.javaparser.symbolsolver.resolution.typesolvers;

import com.github.javaparser.symbolsolver.javassistmodel.JavassistFactory;
import com.github.javaparser.symbolsolver.model.declarations.ReferenceTypeDeclaration;
import com.github.javaparser.symbolsolver.model.resolution.SymbolReference;
import com.github.javaparser.symbolsolver.model.resolution.TypeSolver;
import com.github.javaparser.symbolsolver.model.resolution.UnsolvedSymbolException;
import javassist.ClassPool;
import javassist.CtClass;
import javassist.NotFoundException;
import java.io.IOException;
import java.io.InputStream;
import java.util.Enumeration;
import java.util.HashMap;
import java.util.Map;
import java.util.jar.JarEntry;
import java.util.jar.JarFile;

public class JarTypeSolver implements TypeSolver {
    private static JarTypeSolver instance;
    private TypeSolver parent;
    private Map<String, ClasspathElement> classpathElements = new HashMap<>();
    private ClassPool classPool = new ClassPool(false);
    public JarTypeSolver(String pathToJar) throws IOException {
        addPathToJar(pathToJar);
    }
    public static JarTypeSolver getJarTypeSolver(String pathToJar) throws IOException {
        if (instance == null) {
            instance = new JarTypeSolver(pathToJar);
        } else {
            instance.addPathToJar(pathToJar);
        }
        return instance;
    }
    private void addPathToJar(String pathToJar) throws IOException {
        try {
            classPool.appendClassPath(pathToJar);
            classPool.appendSystemPath();
        } catch (NotFoundException e) {
            throw new RuntimeException(e);
        }
        JarFile jarFile = new JarFile(pathToJar);
        JarEntry entry = null;
        Enumeration<JarEntry> e = jarFile.entries();
        while (e.hasMoreElements()) {
            entry = e.nextElement();
            if (entry != null && !entry.isDirectory() && entry.getName().endsWith(".class")) {
                String name = entryPathToClassName(entry.getName());
                classpathElements.put(name, new ClasspathElement(jarFile, entry, name));
            }
        }
    }
    @Override
    public TypeSolver getParent() {
        return parent;
    }
    @Override
    public void setParent(TypeSolver parent) {
        this.parent = parent;
    }
    private String entryPathToClassName(String entryPath) {
        if (!entryPath.endsWith(".class")) {
            throw new IllegalStateException();
        }
        String className = entryPath.substring(0, entryPath.length() - 6);
        className = className.replace('/', '.');
        className = className.replace('$', '.');
        return className;
    }
    @Override
    public SymbolReference<ReferenceTypeDeclaration> tryToSolveType(String name) {
        try {
            return classpathElements.containsKey(name) ? SymbolReference.solved(JavassistFactory.toTypeDeclaration(classpathElements.get(name).toCtClass(), getRoot())) : SymbolReference.unsolved(ReferenceTypeDeclaration.class);
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }
    @Override
    public ReferenceTypeDeclaration solveType(String name) throws UnsolvedSymbolException {
        SymbolReference<ReferenceTypeDeclaration> ref = tryToSolveType(name);
        if (ref.isSolved()) {
            return ref.getCorrespondingDeclaration();
        } else {
            throw new UnsolvedSymbolException(name);
        }
    }
    private class ClasspathElement {
        private JarFile jarFile;
        private JarEntry entry;
        private String path;
        ClasspathElement(JarFile jarFile, JarEntry entry, String path) {
            this.jarFile = jarFile;
            this.entry = entry;
            this.path = path;
        }
        CtClass toCtClass() throws IOException {
            try (InputStream is = jarFile.getInputStream(entry)) {
                return classPool.makeClass(is);
            }
        }
    }
}
