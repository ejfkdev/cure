import java.nio.file.Path;
import java.util.List;
import java.util.Objects;
import toolbox.JavacTask;
import toolbox.Task.Expect;
import toolbox.Task.OutputKind;
import toolbox.TestRunner;
import toolbox.ToolBox;

public class PrimitivePatternsSwitchRequirePreview extends TestRunner {
    ToolBox tb;
    public PrimitivePatternsSwitchRequirePreview() {
        super(System.err);
        tb = new ToolBox();
    }
    public static void main(String[] args) throws Exception {
        new PrimitivePatternsSwitchRequirePreview().runTests();
    }
    @Test
    public void testBoolean() throws Exception {
        Path curPath = Path.of(".");
        List<String> actual = new JavacTask(tb).options("-XDrawDiagnostics", "-XDdev", "-XDshould-stop.at=FLOW").sources("""
                      class C {
                          public static void testBoolean(Boolean value) {
                            switch (value) {
                                case true   -> System.out.println("true");
                                default     -> System.out.println("false");
                            }
                          }
                      }
                      """).outdir(curPath).run(Expect.FAIL).getOutputLines(OutputKind.DIRECT);
        List<String> expected = List.of("C.java:4:16: compiler.err.preview.feature.disabled.plural: (compiler.misc.feature.primitive.patterns)", "1 error");
        if (!Objects.equals(actual, expected)) {
            error("Expected: " + expected + ", but got: " + actual);
        }
    }
    @Test
    public void testLong() throws Exception {
        Path curPath = Path.of(".");
        List<String> actual = new JavacTask(tb).options("-XDrawDiagnostics", "-XDdev", "-XDshould-stop.at=FLOW").sources("""
                      class C {
                          public static void testLong(Long value) {
                             switch (value) {
                                 case 0L      -> System.out.println("zero");
                                 default      -> System.out.println("non-zero");
                             }
                         }
                      }
                      """).outdir(curPath).run(Expect.FAIL).getOutputLines(OutputKind.DIRECT);
        List<String> expected = List.of("C.java:4:17: compiler.err.preview.feature.disabled.plural: (compiler.misc.feature.primitive.patterns)", "1 error");
        if (!Objects.equals(actual, expected)) {
            error("Expected: " + expected + ", but got: " + actual);
        }
    }
    @Test
    public void testFloat() throws Exception {
        Path curPath = Path.of(".");
        List<String> actual = new JavacTask(tb).options("-XDrawDiagnostics", "-XDdev", "-XDshould-stop.at=FLOW").sources("""
                      class C {
                         public static void testFloat(Float value) {
                           switch (value) {
                               case 0f      -> System.out.println("zero");
                               default      -> System.out.println("non-zero");
                           }
                         }
                      }
                      """).outdir(curPath).run(Expect.FAIL).getOutputLines(OutputKind.DIRECT);
        List<String> expected = List.of("C.java:4:15: compiler.err.preview.feature.disabled.plural: (compiler.misc.feature.primitive.patterns)", "1 error");
        if (!Objects.equals(actual, expected)) {
            error("Expected: " + expected + ", but got: " + actual);
        }
    }
    @Test
    public void testDouble() throws Exception {
        Path curPath = Path.of(".");
        List<String> actual = new JavacTask(tb).options("-XDrawDiagnostics", "-XDdev", "-XDshould-stop.at=FLOW").sources("""
                      class C {
                        public static void testDouble(Long value) {
                            switch (value) {
                                case 0L      -> System.out.println("zero");
                                default      -> System.out.println("non-zero");
                            }
                        }
                      }
                      """).outdir(curPath).run(Expect.FAIL).getOutputLines(OutputKind.DIRECT);
        List<String> expected = List.of("C.java:4:16: compiler.err.preview.feature.disabled.plural: (compiler.misc.feature.primitive.patterns)", "1 error");
        if (!Objects.equals(actual, expected)) {
            error("Expected: " + expected + ", but got: " + actual);
        }
    }
}
