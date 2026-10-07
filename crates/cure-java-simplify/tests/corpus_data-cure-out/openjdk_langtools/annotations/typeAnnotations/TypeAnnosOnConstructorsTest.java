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

public class TypeAnnosOnConstructorsTest extends TestRunner {
    protected ToolBox tb;
    TypeAnnosOnConstructorsTest() {
        super(System.err);
        tb = new ToolBox();
    }
    public static void main(String... args) throws Exception {
        new TypeAnnosOnConstructorsTest().runTests();
    }
    protected void runTests() throws Exception {
        runTests((m) -> new Object[] {Paths.get(m.getName())});
    }
    Path[] findJavaFiles(Path... paths) throws IOException {
        return tb.findJavaFiles(paths);
    }
    @Test
    public void testAnnoOnConstructors(Path base) throws Exception {
        Path src = base.resolve("src");
        Path y = src.resolve("Y.java");
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        tb.writeJavaFiles(src, """
                import java.lang.annotation.Target;
                import java.lang.annotation.ElementType;
                import java.lang.annotation.Retention;
                import java.lang.annotation.RetentionPolicy;

                class Y {
                    @TA public Y() {}
                }

                @Target(ElementType.TYPE_USE)
                @Retention(RetentionPolicy.RUNTIME)
                @interface TA {}
                """);
        new JavacTask(tb).files(y).outdir(classes).run();
        Path classDir = getClassDir();
        new JavacTask(tb).classpath(classes, classDir).options("-processor", SimpleProcessor.class.getName()).classes("Y").outdir(classes).run(Task.Expect.SUCCESS);
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
