import toolbox.JavacTask;
import toolbox.Task;
import toolbox.TestRunner;
import toolbox.ToolBox;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;

public class DiagnosticFormatterCompletionFailureTest extends TestRunner {
    ToolBox tb;
    public DiagnosticFormatterCompletionFailureTest() {
        super(System.err);
        tb = new ToolBox();
    }
    public static void main(String[] args) throws Exception {
        new DiagnosticFormatterCompletionFailureTest().runTests((m) -> new Object[] {Paths.get(m.getName())});
    }
    @Test
    public void test(Path base) throws Exception {
        Path libClasses = base.resolve("libclasses");
        Files.createDirectories(libClasses);
        new JavacTask(tb).outdir(libClasses).sources("""
                        package lib;

                        import java.lang.annotation.ElementType;
                        import java.lang.annotation.Retention;
                        import java.lang.annotation.RetentionPolicy;
                        import java.lang.annotation.Target;

                        @Retention(RetentionPolicy.RUNTIME)
                        @Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER})
                        @interface A {}
                        """, """
                        package lib;

                        public class I {
                          public final void g(@A Integer other) {}
                        }
                        """, """
                        package lib;

                        class NoSuch<T> {}
                        """, """
                        package lib;

                        public class Lib {
                          public static I f(@A Integer actual)   { return null; }
                          public static I f(@A NoSuch<?> actual) { return null; }
                        }
                        """).run().writeAll();
        Path lib = libClasses.resolve("lib");
        Files.delete(lib.resolve("NoSuch.class"));
        Files.delete(lib.resolve("A.class"));
        List<String> lines = new JavacTask(tb).classpath(libClasses).sources("""
                import static lib.Lib.f;
                class T {
                  void f(L l) {
                    f(2).g(2);
                  }
                }
                """).run(Task.Expect.FAIL).getOutputLines(Task.OutputKind.DIRECT);
        String output = String.join("\n", lines);
        if (!output.contains("Cannot attach type annotations @A to Lib.f") && !output.contains("class file for lib.NoSuch not found")) {
            throw new AssertionError(output);
        }
    }
}
