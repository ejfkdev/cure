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

public class IncompatibleArgTypesInLambda {
    private static String SOURCE = """
                                   import java.util.function.Supplier;
                                   import javax.swing.JButton;

                                   public class Test {

                                       public void test(Supplier<String> sup) {
                                           JButton button = new JButton("test");
                                           button.addActionListener(() -> {
                                               String s = (sup != null)
                                                       ? sup.get()
                                                       : "";
                                               IO.println(s);
                                           });
                                       }
                                   }
                                   """;
    private Path base;
    private ToolBox tb = new ToolBox();
    @Test
    public void testCompactDiags() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        List<String> log = new JavacTask(tb).options("-d", classes.toString(), "-XDrawDiagnostics", "-Xdiags:compact").sources(SOURCE).run(Task.Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        List<String> expected = List.of("Test.java:8:34: compiler.err.prob.found.req: (compiler.misc.wrong.number.args.in.lambda: java.awt.event.ActionListener)", "- compiler.note.compressed.diags", "1 error");
        tb.checkEqual(expected, log);
    }
    @Test
    public void testVerboseDiags() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        List<String> log = new JavacTask(tb).options("-d", classes.toString(), "-XDrawDiagnostics", "-Xdiags:verbose").sources(SOURCE).run(Task.Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        List<String> expected = List.of("Test.java:8:15: compiler.err.cant.apply.symbol: kindname.method, addActionListener, java.awt.event.ActionListener, @34, kindname.class, javax.swing.AbstractButton, (compiler.misc.no.conforming.assignment.exists: (compiler.misc.wrong.number.args.in.lambda: java.awt.event.ActionListener))", "1 error");
        tb.checkEqual(expected, log);
    }
    @BeforeEach
    public void setUp(TestInfo info) {
        base = Paths.get(".").resolve(info.getTestMethod().orElseThrow().getName());
    }
}
