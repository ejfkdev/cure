import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Arrays;
import java.util.List;
import toolbox.*;
import toolbox.Task.*;

public class CompletionErrorOnRepeatingAnnosTest {
    ToolBox tb = new ToolBox();
    public static void main(String... args) throws Exception {
        new CompletionErrorOnRepeatingAnnosTest().testMissingContainerTypeAnno();
    }
    void testMissingContainerTypeAnno() throws Exception {
        doTest("""
                import java.lang.annotation.*;
                import static java.lang.annotation.RetentionPolicy.*;
                import static java.lang.annotation.ElementType.*;
                @Target({TYPE_USE,FIELD}) @Repeatable( As.class) @interface A { }
                @Target({TYPE_USE,FIELD}) @interface As { A[] value(); }
                """, """
                class T {
                    @A @A String data = "test";
                }
                """, List.of("T.java:2:5: compiler.err.cant.access: As, (compiler.misc.class.file.not.found: As)", "T.java:2:8: compiler.err.invalid.repeatable.annotation.no.value: As", "2 errors"));
    }
    void testMissingContainerAnno() throws Exception {
        doTest("""
                import java.lang.annotation.Repeatable;
                @Repeatable(As.class)
                @interface A {}
                @interface As {
                    A[] value();
                }
                """, "@A @A class T {}", List.of("T.java:1:1: compiler.err.cant.access: As, (compiler.misc.class.file.not.found: As)", "T.java:1:4: compiler.err.invalid.repeatable.annotation.no.value: As", "2 errors"));
    }
    private void doTest(String annosSrc, String annotatedSrc, List<String> expectedOutput) throws Exception {
        Path base = Paths.get(".");
        Path src = base.resolve("src");
        tb.createDirectories(src);
        tb.writeJavaFiles(src, annosSrc);
        Path out = base.resolve("out");
        tb.createDirectories(out);
        new JavacTask(tb).outdir(out).files(tb.findJavaFiles(src)).run();
        tb.deleteFiles(src.resolve("A.java"));
        tb.writeJavaFiles(src, annotatedSrc);
        new JavacTask(tb).outdir(out).classpath(out).files(tb.findJavaFiles(src)).run();
        tb.deleteFiles(out.resolve("As.class"));
        List<String> log = new JavacTask(tb).outdir(out).classpath(out).options("-XDrawDiagnostics").files(tb.findJavaFiles(src)).run(Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        if (!expectedOutput.equals(log)) 
            throw new Exception("expected output not found: " + log);
    }
}
