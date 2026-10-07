import com.sun.source.util.TaskEvent;
import com.sun.source.util.TaskListener;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import toolbox.JavacTask;
import toolbox.Task.Expect;
import toolbox.Task.OutputKind;
import toolbox.TestRunner;
import toolbox.ToolBox;

public class ReadingMethodWithTypeAnno extends TestRunner {
    public static void main(String... args) throws Exception {
        new ReadingMethodWithTypeAnno().runTests((m) -> new Object[] {Paths.get(m.getName())});
    }
    private final ToolBox tb = new ToolBox();
    public ReadingMethodWithTypeAnno() throws IOException {
        super(System.err);
    }
    @Test
    public void test_DeclNone_UseNone(Path base) throws IOException {
        Path libSrc = base.resolve("lib-src");
        Path libClasses = Files.createDirectories(base.resolve("lib-classes"));
        tb.writeJavaFiles(libSrc, """
                          public class Lib {
                              public void test(java.lang.@Ann String s) {
                                  new Object() {};
                              }
                          }
                          """, """
                          import java.lang.annotation.ElementType;
                          import java.lang.annotation.Target;
                          @Target(ElementType.TYPE_USE)
                          public @interface Ann {}
                          """);
        new JavacTask(tb).outdir(libClasses).files(tb.findJavaFiles(libSrc)).run(Expect.SUCCESS).writeAll().getOutput(OutputKind.DIRECT);
        Path src = base.resolve("src");
        Path classes = Files.createDirectories(base.resolve("classes"));
        tb.writeJavaFiles(src, """
                          public class Test {
                          }
                          """);
        new JavacTask(tb).outdir(classes).classpath(libClasses).files(tb.findJavaFiles(src)).callback((task) -> {
            task.addTaskListener(new TaskListener() {
                        @Override
                        public void finished(TaskEvent e) {
                            if (e.getKind() == TaskEvent.Kind.ENTER) {
                                task.getElements().getTypeElement("Lib");
                                task.getElements().getTypeElement("Lib$1");
                            }
                        }
                    });
        }).run(Expect.SUCCESS).writeAll().getOutput(OutputKind.DIRECT);
    }
}
