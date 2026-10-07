import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInfo;
import toolbox.JavacTask;
import toolbox.Task;
import toolbox.ToolBox;

public class CrashOnVoidType {
    Path base;
    ToolBox tb = new ToolBox();
    @Test void testVoidParam() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        List<String> out = new JavacTask(tb).options("-d", classes.toString(), "-XDrawDiagnostics").sources("""
                         import java.lang.annotation.ElementType;
                         import java.lang.annotation.Target;

                         public class Test {
                             public void op(@Ann void p) {}
                         }

                         @Target(ElementType.TYPE_USE)
                         @interface Ann {}
                         """).run(Task.Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        tb.checkEqual(out, List.of("Test.java:5:25: compiler.err.void.not.allowed.here", "1 error"));
    }
    @Test void testVoidLocalVarType() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        List<String> out = new JavacTask(tb).options("-d", classes.toString(), "-XDrawDiagnostics").sources("""
                         import java.lang.annotation.ElementType;
                         import java.lang.annotation.Target;

                         public class Test {
                             public void op() {
                                 @Ann void l;
                             }
                         }

                         @Target(ElementType.TYPE_USE)
                         @interface Ann {}
                         """).run(Task.Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        tb.checkEqual(out, List.of("Test.java:6:14: compiler.err.void.not.allowed.here", "1 error"));
    }
    @Test void testVoidRecordComponent() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        List<String> out = new JavacTask(tb).options("-d", classes.toString(), "-XDrawDiagnostics").sources("""
                         import java.lang.annotation.ElementType;
                         import java.lang.annotation.Target;

                         public record R(@Ann void v) {}

                         @Target(ElementType.TYPE_USE)
                         @interface Ann {}
                         """).run(Task.Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        tb.checkEqual(out, List.of("R.java:4:22: compiler.err.void.not.allowed.here", "1 error"));
    }
    @Test void testVoidExceptionParam() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        List<String> out = new JavacTask(tb).options("-d", classes.toString(), "-XDrawDiagnostics").sources("""
                         import java.lang.annotation.ElementType;
                         import java.lang.annotation.Target;

                         public class Test {
                             public void op() {
                                 try {
                                 } catch (@Ann void e) {}
                             }
                         }

                         @Target(ElementType.TYPE_USE)
                         @interface Ann {}
                         """).run(Task.Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        tb.checkEqual(out, List.of("Test.java:7:23: compiler.err.void.not.allowed.here", "1 error"));
    }
    @Test void testVoidUnion() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        List<String> out = new JavacTask(tb).options("-d", classes.toString(), "-XDrawDiagnostics").sources("""
                         import java.lang.annotation.ElementType;
                         import java.lang.annotation.Target;

                         public class Test {
                             public void op() {
                                 try {
                                 } catch (Exception | @Ann void e) {}
                             }
                         }

                         @Target(ElementType.TYPE_USE)
                         @interface Ann {}
                         """).run(Task.Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        tb.checkEqual(out, List.of("Test.java:7:30: compiler.err.illegal.start.of.type", "1 error"));
    }
    @Test void testVoidTypeParam() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        List<String> out = new JavacTask(tb).options("-d", classes.toString(), "-XDrawDiagnostics").sources("""
                         import java.lang.annotation.ElementType;
                         import java.lang.annotation.Target;
                         import java.util.List;

                         public class Test {
                             public List<@Ann void> l;
                         }

                         @Target(ElementType.TYPE_USE)
                         @interface Ann {}
                         """).run(Task.Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        tb.checkEqual(out, List.of("Test.java:6:17: compiler.err.illegal.start.of.type", "Test.java:6:22: compiler.err.void.not.allowed.here", "2 errors"));
    }
    @Test void testVoidWildcardBound() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        List<String> out = new JavacTask(tb).options("-d", classes.toString(), "-XDrawDiagnostics").sources("""
                         import java.lang.annotation.ElementType;
                         import java.lang.annotation.Target;
                         import java.util.List;

                         public class Test {
                             public List<? extends @Ann void> l;
                         }

                         @Target(ElementType.TYPE_USE)
                         @interface Ann {}
                         """).run(Task.Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        tb.checkEqual(out, List.of("Test.java:6:17: compiler.err.illegal.start.of.type", "Test.java:6:32: compiler.err.void.not.allowed.here", "2 errors"));
    }
    @Test void testVoidTypeParamBound() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        List<String> out = new JavacTask(tb).options("-d", classes.toString(), "-XDrawDiagnostics").sources("""
                         import java.lang.annotation.ElementType;
                         import java.lang.annotation.Target;

                         public class Test {
                             public <T extends @Ann void> void op() {}
                         }

                         @Target(ElementType.TYPE_USE)
                         @interface Ann {}
                         """).run(Task.Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        tb.checkEqual(out, List.of("Test.java:5:23: compiler.err.illegal.start.of.type", "Test.java:5:28: compiler.err.void.not.allowed.here", "2 errors"));
    }
    @BeforeEach
    public void setUp(TestInfo info) {
        base = Paths.get(".").resolve(info.getTestMethod().orElseThrow().getName());
    }
}
