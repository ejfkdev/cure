package com.google.googlejavaformat.java;

import static java.nio.charset.StandardCharsets.UTF_8;
import com.google.common.base.Throwables;
import com.google.common.collect.ImmutableList;
import com.sun.source.tree.CompilationUnitTree;
import com.sun.source.tree.CompoundAssignmentTree;
import com.sun.source.tree.ExpressionTree;
import com.sun.source.tree.IdentifierTree;
import com.sun.source.tree.MemberSelectTree;
import com.sun.source.tree.MethodInvocationTree;
import com.sun.source.tree.ParenthesizedTree;
import com.sun.source.tree.Tree;
import com.sun.source.util.TreePath;
import com.sun.tools.javac.file.JavacFileManager;
import com.sun.tools.javac.parser.JavacParser;
import com.sun.tools.javac.parser.ParserFactory;
import com.sun.tools.javac.tree.JCTree;
import com.sun.tools.javac.tree.JCTree.JCCompilationUnit;
import com.sun.tools.javac.tree.Pretty;
import com.sun.tools.javac.tree.TreeInfo;
import com.sun.tools.javac.util.Context;
import com.sun.tools.javac.util.Log;
import com.sun.tools.javac.util.Options;
import java.io.IOError;
import java.io.IOException;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.MethodType;
import java.lang.invoke.VarHandle;
import java.net.URI;
import java.util.List;
import javax.lang.model.element.Name;
import javax.tools.Diagnostic;
import javax.tools.DiagnosticListener;
import javax.tools.JavaFileObject;
import javax.tools.SimpleJavaFileObject;
import javax.tools.StandardLocation;
import org.jspecify.annotations.Nullable;

class Trees {
    static int getLength(Tree tree, TreePath path) {
        return getEndPosition(tree, path) - getStartPosition(tree);
    }
    static int getStartPosition(Tree expression) {
        return ((JCTree) expression).getStartPosition();
    }
    static int getEndPosition(Tree expression, TreePath path) {
        return getEndPosition(expression, path.getCompilationUnit());
    }
    static int getEndPosition(Tree tree, CompilationUnitTree unit) {
        try {
            return (int) GET_END_POS_HANDLE.invokeExact((JCTree) tree, (JCCompilationUnit) unit);
        } catch (Throwable e) {
            Throwables.throwIfUnchecked(e);
            throw new AssertionError(e);
        }
    }
    static String getSourceForNode(Tree node, TreePath path) {
        CharSequence source;
        try {
            source = path.getCompilationUnit().getSourceFile().getCharContent(false);
        } catch (IOException e) {
            throw new IOError(e);
        }
        return source.subSequence(getStartPosition(node), getEndPosition(node, path)).toString();
    }
    static Name getMethodName(MethodInvocationTree methodInvocation) {
        ExpressionTree select = methodInvocation.getMethodSelect();
        return switch (select) {
            case MemberSelectTree memberSelect -> memberSelect.getIdentifier();
            case IdentifierTree identifier -> identifier.getName();
            default -> throw new AssertionError(select);
        };
    }
    static @Nullable ExpressionTree getMethodReceiver(MethodInvocationTree methodInvocation) {
        ExpressionTree select = methodInvocation.getMethodSelect();
        return select instanceof MemberSelectTree memberSelectTree ? memberSelectTree.getExpression() : null;
    }
    static String operatorName(ExpressionTree expression) {
        JCTree.Tag tag = ((JCTree) expression).getTag();
        if (tag == JCTree.Tag.ASSIGN) {
            return "=";
        }
        boolean assignOp = expression instanceof CompoundAssignmentTree;
        if (assignOp) {
            tag = tag.noAssignOp();
        }
        String name = new Pretty(null, true).operatorName(tag);
        return assignOp ? name + "=" : name;
    }
    static int precedence(ExpressionTree expression) {
        return TreeInfo.opPrec(((JCTree) expression).getTag());
    }
    static ExpressionTree skipParen(ExpressionTree node) {
        return ((ParenthesizedTree) node).getExpression();
    }
    static JCCompilationUnit parse(Context context, List<Diagnostic<? extends JavaFileObject>> errorDiagnostics, boolean allowStringFolding, String javaInput) {
        DiagnosticListener<JavaFileObject> diagnostics = (diagnostic) -> {
            if (errorDiagnostic(diagnostic)) {
                errorDiagnostics.add(diagnostic);
            }
        };
        context.put(DiagnosticListener.class, diagnostics);
        Options.instance(context).put("--enable-preview", "true");
        Options.instance(context).put("allowStringFolding", Boolean.toString(allowStringFolding));
        JavacFileManager fileManager = new JavacFileManager(context, true, UTF_8);
        try {
            fileManager.setLocation(StandardLocation.PLATFORM_CLASS_PATH, ImmutableList.of());
        } catch (IOException e) {
            throw new IOError(e);
        }
        SimpleJavaFileObject source = new SimpleJavaFileObject(URI.create("source"), JavaFileObject.Kind.SOURCE) {
          @Override
          public String getCharContent(boolean ignoreEncodingErrors) {
            return javaInput;
          }
        };
        Log.instance(context).useSource(source);
        ParserFactory parserFactory = ParserFactory.instance(context);
        JavacParser parser;
        try {
            parser = newParser(parserFactory, javaInput, true, true, true);
        } catch (Throwable e) {
            Throwables.throwIfUnchecked(e);
            throw new AssertionError(e);
        }
        JCCompilationUnit unit = parser.parseCompilationUnit();
        unit.sourcefile = source;
        return unit;
    }
    private static JavacParser newParser(ParserFactory parserFactory, CharSequence source, boolean keepDocComments, boolean keepEndPos, boolean keepLineMap) {
        return END_POS_TABLE_CLASS != null ? parserFactory.newParser(source, keepDocComments, keepEndPos, keepLineMap) : parserFactory.newParser(source, keepDocComments, keepLineMap, false);
    }
    private static boolean errorDiagnostic(Diagnostic<?> input) {
        return input.getKind() != Diagnostic.Kind.ERROR ? false : !input.getCode().equals("compiler.err.invalid.meth.decl.ret.type.req");
    }
    private static final @Nullable Class<?> END_POS_TABLE_CLASS = getEndPosTableClass();
    private static @Nullable Class<?> getEndPosTableClass() {
        try {
            return Class.forName("com.sun.tools.javac.tree.EndPosTable");
        } catch (ClassNotFoundException e) {
            return null;
        }
    }
    private static final MethodHandle GET_END_POS_HANDLE = getEndPosMethodHandle();
    private static MethodHandle getEndPosMethodHandle() {
        MethodHandles.Lookup lookup = MethodHandles.lookup();
        if (END_POS_TABLE_CLASS == null) {
            try {
                return MethodHandles.dropArguments(lookup.findVirtual(JCTree.class, "getEndPosition", MethodType.methodType(int.class)), 1, JCCompilationUnit.class);
            } catch (ReflectiveOperationException e1) {
                throw new LinkageError(e1.getMessage(), e1);
            }
        }
        try {
            return MethodHandles.filterArguments(lookup.findVirtual(JCTree.class, "getEndPosition", MethodType.methodType(int.class, END_POS_TABLE_CLASS)), 1, lookup.findVarHandle(JCCompilationUnit.class, "endPositions", END_POS_TABLE_CLASS).toMethodHandle(VarHandle.AccessMode.GET));
        } catch (ReflectiveOperationException e) {
            throw new LinkageError(e.getMessage(), e);
        }
    }
}
