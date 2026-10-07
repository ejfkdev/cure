import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.Set;
import javax.annotation.processing.AbstractProcessor;
import javax.annotation.processing.RoundEnvironment;
import javax.annotation.processing.SupportedAnnotationTypes;
import javax.lang.model.SourceVersion;
import javax.lang.model.element.TypeElement;
import toolbox.JavacTask;
import toolbox.Task;
import toolbox.Task.Mode;
import toolbox.Task.OutputKind;
import toolbox.TestRunner;
import toolbox.ToolBox;

public class CrashDuplicateAnnotationDeclarationTest extends TestRunner {
    protected ToolBox tb;
    CrashDuplicateAnnotationDeclarationTest() {
        super(System.err);
        tb = new ToolBox();
    }
    public static void main(String... args) throws Exception {
        new CrashDuplicateAnnotationDeclarationTest().runTests();
    }
    protected void runTests() throws Exception {
        runTests((m) -> new Object[] {Paths.get(m.getName())});
    }
    Path[] findJavaFiles(Path... paths) throws IOException {
        return tb.findJavaFiles(paths);
    }
    @Test
    public void testDupAnnoDeclaration(Path base) throws Exception {
        Path src = base.resolve("src");
        Path pkg = src.resolve("pkg");
        Path y = pkg.resolve("Y.java");
        Path t = pkg.resolve("T.java");
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        tb.writeJavaFiles(src, """
                package pkg;
                @SuppressWarnings("deprecation")
                class Y {
                    @interface A {}
                    @interface A {} // error: class A is already defined
                    T t;
                }
                """);
        tb.writeJavaFiles(src, """
                package pkg;
                @Deprecated class T {}
                """);
        new JavacTask(tb).files(t).outdir(classes).run();
        List<String> expected = List.of("Y.java:5:6: compiler.err.already.defined: kindname.class, pkg.Y.A, kindname.class, pkg.Y", "1 error");
        Path classDir = getClassDir();
        checkOutputAcceptable(expected, new JavacTask(tb).classpath(classes, classDir).options("-processor", SimpleProcessor.class.getName(), "-XDrawDiagnostics").files(y, t).outdir(classes).run(Task.Expect.FAIL, 1).writeAll().getOutputLines(Task.OutputKind.DIRECT));
    }
    void checkOutputAcceptable(List<String> expected, List<String> found) {
        if (found.size() != expected.size()) {
            throw new AssertionError("Unexpected output: " + found);
        } else {
            for (int i = 0; i < expected.size(); i++) {
                if (!found.get(i).contains(expected.get(i))) {
                    throw new AssertionError("Unexpected output: " + found);
                }
            }
        }
    }
    public Path getClassDir() {
        String classes = ToolBox.testClasses;
        return classes == null ? Paths.get("build") : Paths.get(classes);
    }
    @SupportedAnnotationTypes("*")
    public static final class SimpleProcessor extends AbstractProcessor {
        @Override
        public SourceVersion getSupportedSourceVersion() {
            return SourceVersion.latestSupported();
        }
        @Override
        public boolean process(Set<? extends TypeElement> annotations, RoundEnvironment roundEnv) {
            return false;
        }
    }
}
