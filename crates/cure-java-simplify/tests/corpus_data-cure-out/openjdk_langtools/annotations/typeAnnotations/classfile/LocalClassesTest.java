import com.sun.source.tree.ClassTree;
import com.sun.source.util.TaskEvent;
import com.sun.source.util.TaskListener;
import com.sun.source.util.TreePathScanner;
import com.sun.source.util.Trees;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.HashMap;
import java.util.Map;
import java.util.Set;
import javax.annotation.processing.AbstractProcessor;
import javax.annotation.processing.RoundEnvironment;
import javax.annotation.processing.SupportedAnnotationTypes;
import javax.lang.model.SourceVersion;
import javax.lang.model.element.Element;
import javax.lang.model.element.TypeElement;
import javax.lang.model.type.DeclaredType;
import javax.lang.model.type.TypeMirror;
import javax.lang.model.util.ElementFilter;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInfo;
import toolbox.JavacTask;
import toolbox.ToolBox;

public class LocalClassesTest {
    ToolBox tb = new ToolBox();
    Path base;
    @Test void test() throws Exception {
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        Map<String, String> local2enclosing = new HashMap<>();
        new JavacTask(tb).options("-d", classes.toString()).sources("""
                         import java.lang.annotation.ElementType;
                         import java.lang.annotation.Target;

                         public class Test {
                            public static void m1() {
                               class Local1 {
                                  @Nullable Local1 l;
                               }
                            }
                            public void m2() {
                               class Local2 {
                                  @Nullable Local2 l;
                               }
                            }
                         }

                         @Target({ElementType.TYPE_USE})
                         @interface Nullable {}
                         """).callback((task) -> {
            task.addTaskListener(new TaskListener() {
                        @Override
                        public void finished(TaskEvent e) {
                            if (e.getKind() == TaskEvent.Kind.ANALYZE) {
                                Trees trees = Trees.instance(task);
                                new TreePathScanner<>() {
                                    @Override
                                    public Object visitClass(ClassTree node, Object p) {
                                        if (node.getSimpleName().toString().startsWith("Local")) {
                                            Element el = trees.getElement(getCurrentPath());
                                            TypeMirror type = trees.getTypeMirror(getCurrentPath());
                                            local2enclosing.put(el.getSimpleName().toString(), ((DeclaredType) type).getEnclosingType().toString());
                                        }
                                        return super.visitClass(node, p);
                                    }
                                }.scan(e.getCompilationUnit(), null);
                            }
                        }
                    });
        }).run().writeAll();
        Path classes2 = base.resolve("classes2");
        Files.createDirectories(classes2);
        ProcessorImpl p = new ProcessorImpl();
        new JavacTask(tb).options("-cp", classes.toString(), "-d", classes2.toString()).processors(p).classes("Test$1Local1", "Test$1Local2").run().writeAll();
        Assertions.assertEquals(local2enclosing.get("Local1"), p.local2enclosing.get("Local1"));
        Assertions.assertEquals(local2enclosing.get("Local2"), p.local2enclosing.get("Local2"));
    }
    @SupportedAnnotationTypes("*")
    private static class ProcessorImpl extends AbstractProcessor {
        private Map<String, String> local2enclosing = new HashMap<>();
        @Override
        public boolean process(Set<? extends TypeElement> annotations, RoundEnvironment roundEnv) {
            for (TypeElement te : ElementFilter.typesIn(roundEnv.getRootElements())) {
                if (te.getSimpleName().toString().startsWith("Local")) {
                    local2enclosing.put(te.getSimpleName().toString(), ((DeclaredType) te.asType()).getEnclosingType().toString());
                }
            }
            return false;
        }
        @Override
        public SourceVersion getSupportedSourceVersion() {
            return SourceVersion.latestSupported();
        }
    }
    @BeforeEach
    public void setup(TestInfo info) {
        base = Paths.get(".").resolve(info.getTestMethod().orElseThrow().getName());
    }
}
