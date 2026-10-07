import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInfo;
import toolbox.JavacTask;
import toolbox.ToolBox;

public class TypeAnnotatedCastsInFieldGroup {
    private Path base;
    private ToolBox tb = new ToolBox();
    @Test
    public void testCompactDiags() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        new JavacTask(tb).options("-d", classes.toString(), "-XDrawDiagnostics", "-Xdiags:compact").sources("""
                         import java.lang.annotation.ElementType;
                         import java.lang.annotation.Target;

                         class Test {
                             int x = (@A int) 0, y = (@A int) 1;
                         }

                         @Target({ElementType.TYPE_USE})
                         @interface A {}
                         """).run().writeAll();
    }
    @BeforeEach
    public void setUp(TestInfo info) {
        base = Paths.get(".").resolve(info.getTestMethod().orElseThrow().getName());
    }
}
