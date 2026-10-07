import toolbox.*;
import toolbox.Task.*;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;

public class CompletionErrorOnEnclosingType {
    ToolBox tb = new ToolBox();
    public static void main(String... args) throws Exception {
        new CompletionErrorOnEnclosingType().testMissingEnclosingType();
    }
    void testMissingEnclosingType() throws Exception {
        Path base = Paths.get(".");
        Path src = base.resolve("src");
        tb.createDirectories(src);
        tb.writeJavaFiles(src, """
                import static java.lang.annotation.ElementType.TYPE_USE;
                import java.lang.annotation.Target;
                @Target(TYPE_USE)
                @interface Anno {}

                class A<E> {}

                class B {
                  private @Anno A<String> a;
                }
                """, """
                class C {
                  B b;
                }
                """);
        Path out = base.resolve("out");
        tb.createDirectories(out);
        new JavacTask(tb).outdir(out).files(tb.findJavaFiles(src)).run();
        tb.deleteFiles(out.resolve("A.class"));
        List<String> log = new JavacTask(tb).outdir(out).classpath(out).options("-XDrawDiagnostics").files(src.resolve("C.java")).run(Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        var expectedOutput = List.of("B.class:-:-: compiler.err.cant.attach.type.annotations: @Anno, B, a, (compiler.misc.class.file.not.found: A)", "1 error");
        if (!expectedOutput.equals(log)) {
            throw new Exception("expected output not found: " + log);
        }
    }
}
