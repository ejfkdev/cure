import com.sun.source.tree.NewClassTree;
import com.sun.source.util.TaskEvent;
import com.sun.source.util.TaskListener;
import com.sun.source.util.TreePathScanner;
import com.sun.source.util.Trees;
import toolbox.JavacTask;
import toolbox.Task;
import toolbox.TestRunner;
import toolbox.ToolBox;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicBoolean;
import javax.lang.model.type.TypeMirror;

public class NewClassTypeAnnotation extends TestRunner {
    private ToolBox tb;
    public static void main(String[] args) throws Exception {
        new NewClassTypeAnnotation().runTests();
    }
    NewClassTypeAnnotation() {
        super(System.err);
        tb = new ToolBox();
    }
    public void runTests() throws Exception {
        runTests((m) -> new Object[] {Paths.get(m.getName())});
    }
    @Test
    public void testTypeAnnotations(Path base) throws Exception {
        Path current = base.resolve(".");
        Path src = current.resolve("src");
        Path classes = current.resolve("classes");
        tb.writeJavaFiles(src, """
                package test;

                import java.lang.annotation.ElementType;
                import java.lang.annotation.Retention;
                import java.lang.annotation.RetentionPolicy;
                import java.lang.annotation.Target;

                class Test<T> {

                  @Target(ElementType.TYPE_USE)
                  @Retention(RetentionPolicy.RUNTIME)
                  @interface TypeAnnotation {}

                  public void testMethod() {
                    new Test<@TypeAnnotation String>();
                    new Test<@TypeAnnotation String>() {};
                  }
                }
                """);
        Files.createDirectories(classes);
        AtomicBoolean seenAnnotationMirror = new AtomicBoolean();
        List<String> actual = new ArrayList<>();
        class Scanner extends TreePathScanner<Void, Void> {

                    private final Trees trees;

                    Scanner(Trees trees) {
                        this.trees = trees;
                    }

                    @Override
                    public Void visitNewClass(final NewClassTree node, final Void unused) {
                        TypeMirror type = trees.getTypeMirror(getCurrentPath());
                        actual.add(String.format("Type: %s", type));
                        return null;
                    }
                }

                new JavacTask(tb)
                        .outdir(classes)
                        .callback(
                                task -> {
                                    task.addTaskListener(
                                            new TaskListener() {
                                                @Override
                                                public void finished(TaskEvent e) {
                                                    if (e.getKind() != TaskEvent.Kind.ANALYZE) {
                                                        return;
                                                    }
                                                    System.err.println(e);
                                                    new Scanner(Trees.instance(task))
                                                            .scan(e.getCompilationUnit(), null);
                                                }
                                            });
                                })
                        .files(tb.findJavaFiles(src))
                        .run(Task.Expect.SUCCESS)
                        .writeAll();
        List<String> expected = List.of("Type: test.Test<java.lang.@test.Test.TypeAnnotation String>", "Type: <anonymous test.Test<java.lang.@test.Test.TypeAnnotation String>>");
        if (!expected.equals(actual)) {
            throw new AssertionError("expected: " + expected + ", actual: " + actual);
        }
    }
}
