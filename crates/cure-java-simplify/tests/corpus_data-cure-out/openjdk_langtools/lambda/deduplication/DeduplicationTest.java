import static java.nio.charset.StandardCharsets.UTF_8;
import static java.util.stream.Collectors.joining;
import static java.util.stream.Collectors.toList;
import static java.util.stream.Collectors.toMap;
import static java.util.stream.Collectors.toSet;
import com.sun.source.util.JavacTask;
import com.sun.source.util.TaskEvent;
import com.sun.source.util.TaskEvent.Kind;
import com.sun.source.util.TaskListener;
import java.lang.classfile.BootstrapMethodEntry;
import java.lang.classfile.*;
import java.lang.classfile.attribute.BootstrapMethodsAttribute;
import java.lang.classfile.constantpool.MethodHandleEntry;
import com.sun.tools.javac.api.ClientCodeWrapper.Trusted;
import com.sun.tools.javac.api.JavacTaskImpl;
import com.sun.tools.javac.api.JavacTool;
import com.sun.tools.javac.code.Symbol;
import com.sun.tools.javac.code.Symbol.MethodSymbol;
import com.sun.tools.javac.code.Types;
import com.sun.tools.javac.comp.TreeDiffer;
import com.sun.tools.javac.comp.TreeHasher;
import com.sun.tools.javac.file.JavacFileManager;
import com.sun.tools.javac.tree.JCTree.JCCompilationUnit;
import com.sun.tools.javac.tree.JCTree.JCExpression;
import com.sun.tools.javac.tree.JCTree.JCIdent;
import com.sun.tools.javac.tree.JCTree.JCLambda;
import com.sun.tools.javac.tree.JCTree.JCMethodInvocation;
import com.sun.tools.javac.tree.JCTree.JCTypeCast;
import com.sun.tools.javac.tree.JCTree.Tag;
import com.sun.tools.javac.tree.TreeScanner;
import com.sun.tools.javac.util.Context;
import com.sun.tools.javac.util.JCDiagnostic;
import java.io.InputStream;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.TreeSet;
import javax.tools.Diagnostic;
import javax.tools.DiagnosticListener;
import javax.tools.JavaFileObject;

public class DeduplicationTest {
    public static void main(String[] args) throws Exception {
        JavacFileManager fileManager = new JavacFileManager(new Context(), false, UTF_8);
        JavacTool javacTool = JavacTool.create();
        Listener diagnosticListener = new Listener();
        Path file = Paths.get(System.getProperty("test.src")).resolve("Deduplication.java");
        String sourceVersion = Integer.toString(Runtime.version().feature());
        JavacTask task = javacTool.getTask(null, null, diagnosticListener, Arrays.asList("-d", ".", "-g:none", "-XDdebug.dumpLambdaToMethodDeduplication", "-XDdebug.dumpLambdaToMethodStats", "--enable-preview", "-source", System.getProperty("java.specification.version")), null, fileManager.getJavaFileObjects(file));
        Context context = ((JavacTaskImpl) task).getContext();
        Types types = Types.instance(context);
        Map<JCLambda, JCLambda> dedupedLambdas = new LinkedHashMap<>();
        task.addTaskListener(new TreeDiffHashTaskListener(dedupedLambdas, types));
        Iterable<? extends JavaFileObject> generated = task.generate();
        if (!diagnosticListener.unexpected.isEmpty()) {
            throw new AssertionError(diagnosticListener.unexpected.stream().map((d) -> String.format("%s: %s", d.getCode(), d.getMessage(Locale.getDefault()))).collect(joining(", ", "unexpected diagnostics: ", "")));
        }
        Map<JCLambda, JCLambda> actual = diagnosticListener.deduplicationTargets();
        dedupedLambdas.forEach((k, v) -> {
            if (!actual.containsKey(k)) {
                throw new AssertionError("expected " + k + " to be deduplicated");
            }
            if (!v.equals(actual.get(k))) {
                throw new AssertionError(String.format("expected %s to be deduplicated to:\n  %s\nwas:  %s", k, v, actual.get(v)));
            }
        });
        Set<String> bootstrapMethodNames = new TreeSet<>();
        for (JavaFileObject output : generated) {
            ClassModel cm;
            try (InputStream input = output.openInputStream()) {
                cm = ClassFile.of().parse(input.readAllBytes());
            }
            if (cm.thisClass().asInternalName().equals("com/sun/tools/javac/comp/Deduplication$R") || cm.thisClass().asInternalName().equals("com/sun/tools/javac/comp/Deduplication$1C") || cm.thisClass().asInternalName().equals("com/sun/tools/javac/comp/Deduplication$2C")) {
                continue;
            }
            BootstrapMethodsAttribute bsm = cm.findAttribute(Attributes.bootstrapMethods()).orElseThrow();
            for (BootstrapMethodEntry b : bsm.bootstrapMethods()) {
                if (b.bootstrapMethod().asSymbol().methodName().equals("metafactory")) {
                    bootstrapMethodNames.add(((MethodHandleEntry) b.arguments().get(1)).reference().name().stringValue());
                }
            }
        }
        Set<String> deduplicatedNames = diagnosticListener.expectedLambdaMethods().stream().map((s) -> s.getSimpleName().toString()).sorted().collect(toSet());
        if (!deduplicatedNames.equals(bootstrapMethodNames)) {
            throw new AssertionError(String.format("expected deduplicated methods: %s, but saw: %s", deduplicatedNames, bootstrapMethodNames));
        }
    }
    private static List<Symbol> paramSymbols(JCLambda lambda) {
        return lambda.params.stream().map((x) -> x.sym).collect(toList());
    }
    @Trusted
    static class Listener implements DiagnosticListener<JavaFileObject> {
        final Map<MethodSymbol, JCLambda> lambdaMethodSymbolsToTrees = new LinkedHashMap<>();
        final Map<JCLambda, MethodSymbol> deduped = new LinkedHashMap<>();
        final List<Diagnostic<? extends JavaFileObject>> unexpected = new ArrayList<>();
        @Override
        public void report(Diagnostic<? extends JavaFileObject> diagnostic) {
            JCDiagnostic d = (JCDiagnostic) diagnostic;
            switch (d.getCode()) {
                case "compiler.note.lambda.stat":
                    lambdaMethodSymbolsToTrees.put((MethodSymbol) d.getArgs()[1], (JCLambda) d.getDiagnosticPosition().getTree());
                    break;
                case "compiler.note.verbose.l2m.deduplicate":
                    deduped.put((JCLambda) d.getDiagnosticPosition().getTree(), (MethodSymbol) d.getArgs()[0]);
                    break;
                case "compiler.note.preview.filename":
                case "compiler.note.preview.recompile":
                    break;
                default:
                    unexpected.add(diagnostic);
            }
        }
        Set<MethodSymbol> expectedLambdaMethods() {
            return lambdaMethodSymbolsToTrees.entrySet().stream().filter((e) -> !deduped.containsKey(e.getValue())).map(Map.Entry::getKey).collect(toSet());
        }
        Map<JCLambda, JCLambda> deduplicationTargets() {
            return deduped.entrySet().stream().collect(toMap(Map.Entry::getKey, (e) -> lambdaMethodSymbolsToTrees.get(e.getValue()), (a, b) -> {
                throw new AssertionError();
            }, LinkedHashMap::new));
        }
    }
    private static class TreeDiffHashTaskListener implements TaskListener {
        private final Map<JCLambda, JCLambda> dedupedLambdas;
        private final Types types;
        public TreeDiffHashTaskListener(Map<JCLambda, JCLambda> dedupedLambdas, Types types) {
            this.dedupedLambdas = dedupedLambdas;
            this.types = types;
        }
        @Override
        public void finished(TaskEvent e) {
            if (e.getKind() != Kind.ANALYZE) {
                return;
            }
            List<List<JCLambda>> lambdaEqualsGroups = new ArrayList<>();
            List<List<JCLambda>> lambdaNotEqualsGroups = new ArrayList<>();
            new TreeScanner() {
                @Override
                public void visitApply(JCMethodInvocation tree) {
                    if (isMethodWithName(tree, "groupEquals")) {
                        addToGroup(tree, lambdaEqualsGroups);
                    } else if (isMethodWithName(tree, "groupNotEquals")) {
                        addToGroup(tree, lambdaNotEqualsGroups);
                    }
                    super.visitApply(tree);
                }
            }.scan((JCCompilationUnit) e.getCompilationUnit());
            for (int i = 0; i < lambdaEqualsGroups.size(); i++) {
                List<JCLambda> curr = lambdaEqualsGroups.get(i);
                JCLambda first = null;
                for (JCLambda lhs : curr) {
                    if (first == null) {
                        first = lhs;
                    } else {
                        dedupedLambdas.put(lhs, first);
                    }
                    for (JCLambda rhs : curr) {
                        if (rhs != lhs) {
                            if (!new TreeDiffer(types, paramSymbols(lhs), paramSymbols(rhs)).scan(lhs.body, rhs.body)) {
                                throw new AssertionError(String.format("expected lambdas to be equal\n%s\n%s", lhs, rhs));
                            }
                            if (TreeHasher.hash(types, lhs, paramSymbols(lhs)) != TreeHasher.hash(types, rhs, paramSymbols(rhs))) {
                                throw new AssertionError(String.format("expected lambdas to hash to the same value\n%s\n%s", lhs, rhs));
                            }
                        }
                    }
                }
                assertNotEqualsWithinGroup(lambdaEqualsGroups, i, curr, types);
            }
            lambdaEqualsGroups.clear();
            for (int i = 0; i < lambdaNotEqualsGroups.size(); i++) {
                assertNotEqualsWithinGroup(lambdaNotEqualsGroups, i, lambdaNotEqualsGroups.get(i), types);
            }
            lambdaNotEqualsGroups.clear();
        }
        private void assertNotEqualsWithinGroup(List<List<JCLambda>> lambdaNotEqualsGroups, int i, List<JCLambda> curr, Types types) {
            for (int j = 0; j < lambdaNotEqualsGroups.size(); j++) {
                if (i == j) {
                    continue;
                }
                for (JCLambda lhs : curr) {
                    for (JCLambda rhs : lambdaNotEqualsGroups.get(j)) {
                        if (new TreeDiffer(types, paramSymbols(lhs), paramSymbols(rhs)).scan(lhs.body, rhs.body)) {
                            throw new AssertionError(String.format("expected lambdas to not be equal\n%s\n%s", lhs, rhs));
                        }
                        if (TreeHasher.hash(types, lhs, paramSymbols(lhs)) == TreeHasher.hash(types, rhs, paramSymbols(rhs))) {
                            throw new AssertionError(String.format("expected lambdas to hash to different values\n%s\n%s", lhs, rhs));
                        }
                    }
                }
            }
        }
        private boolean isMethodWithName(JCMethodInvocation tree, String markerMethodName) {
            return tree.getMethodSelect().getTag() == Tag.IDENT && ((JCIdent) tree.getMethodSelect()).getName().contentEquals(markerMethodName);
        }
        private void addToGroup(JCMethodInvocation tree, List<List<JCLambda>> groupToAdd) {
            List<JCLambda> xs = new ArrayList<>();
            for (JCExpression arg : tree.getArguments()) {
                if (arg instanceof JCTypeCast) {
                    arg = ((JCTypeCast) arg).getExpression();
                }
                xs.add((JCLambda) arg);
            }
            groupToAdd.add(xs);
        }
    }
}
