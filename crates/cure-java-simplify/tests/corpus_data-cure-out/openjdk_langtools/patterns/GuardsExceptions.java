import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import toolbox.JavacTask;
import toolbox.Task;
import toolbox.ToolBox;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInfo;

public class GuardsExceptions {
    Path base;
    ToolBox tb = new ToolBox();
    @Test void testUnreportedExceptionReported() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        List<String> out = new JavacTask(tb).options("-d", classes.toString(), "-XDrawDiagnostics", "-nowarn").sources("""
                         package test;

                         import java.io.IOException;

                         public class Test {
                             private static boolean guard() throws IOException {
                                 throw new IOException();
                             }

                             public static void test(Object o) {
                                 switch (o) {
                                     case String s when guard() -> {}
                                     default -> {}
                                 }
                             }
                         }
                         """).run(Task.Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        tb.checkEqual(out, List.of("Test.java:12:37: compiler.err.unreported.exception.need.to.catch.or.throw: java.io.IOException", "1 error"));
    }
    @Test void testThrownExceptionHandled() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        new JavacTask(tb).options("-d", classes.toString(), "-XDrawDiagnostics", "-nowarn").sources("""
                         package test;

                         import java.io.IOException;

                         public class Test {
                             private static boolean guard() throws IOException {
                                 throw new IOException();
                             }

                             public static void test(Object o) {
                                 try {
                                     switch (o) {
                                         case String s when guard() -> {}
                                         default -> {}
                                     }
                                 } catch (IOException e) {}
                             }
                         }
                         """).run().writeAll();
    }
    @Test void testUnreachableStatementReported() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        List<String> out = new JavacTask(tb).options("-d", classes.toString(), "-XDrawDiagnostics", "-nowarn").sources("""
                         package test;

                         import java.io.IOException;

                         public class Test {
                             public static void test(Object o) {
                                 switch (o) {
                                     case String s when switch(s.length()) {
                                         case 0 -> { yield false; System.out.println(); }
                                         default -> true;
                                     } -> {}
                                     default -> {}
                                 }
                             }
                         }
                         """).run(Task.Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        tb.checkEqual(out, List.of("Test.java:9:42: compiler.err.unreachable.stmt", "1 error"));
    }
    @Test void testUnreachableStatementReportedFromSwitchExpression() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        List<String> out = new JavacTask(tb).options("-d", classes.toString(), "-XDrawDiagnostics", "-nowarn").sources("""
                         package test;

                         import java.io.IOException;

                         public class Test {
                             public static int test(Object o) {
                                 return switch (o) {
                                     case String s when switch(s.length()) {
                                         case 0 -> { yield false; System.out.println(); }
                                         default -> true;
                                     } -> 1;
                                     default -> 0;
                                 };
                             }
                         }
                         """).run(Task.Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        tb.checkEqual(out, List.of("Test.java:9:42: compiler.err.unreachable.stmt", "1 error"));
    }
    @BeforeEach
    public void setUp(TestInfo info) {
        base = Paths.get(".").resolve(info.getTestMethod().orElseThrow().getName());
    }
}
