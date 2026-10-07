import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import toolbox.JavacTask;
import toolbox.ToolBox;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInfo;

public class TextBlockU2028 {
    Path base;
    ToolBox tb = new ToolBox();
    @Test void testNoFalseTrailingWhitespaceWarning() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        new JavacTask(tb).options("-d", classes.toString(), "-Xlint:text-blocks", "-XDrawDiagnostics", "-Werror").sources("""
                         public class Test {
                             String s = \"\"\"
                                        foo \\u2028 bar
                                        \"\"\";
                         }
                         """).run().writeAll();
    }
    @BeforeEach
    public void setUp(TestInfo info) {
        base = Paths.get(".").resolve(info.getTestMethod().orElseThrow().getName());
    }
}
