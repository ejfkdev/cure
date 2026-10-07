import com.sun.source.tree.CompilationUnitTree;
import com.sun.source.tree.InstanceOfTree;
import com.sun.source.util.JavacTask;
import com.sun.source.util.TreeScanner;
import java.net.URI;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import javax.tools.JavaCompiler;
import javax.tools.JavaFileObject;
import javax.tools.SimpleJavaFileObject;
import javax.tools.ToolProvider;

public class InstanceOfModelTest {
    private final JavaCompiler compiler = ToolProvider.getSystemJavaCompiler();
    public static void main(String... args) throws Exception {
        new InstanceOfModelTest().run();
    }
    private void run() throws Exception {
        JavaFileObject input = SimpleJavaFileObject.forSource(URI.create("mem://Test.java"), """
                                               public class Test {
                                                   void test(Object o) {
                                                       boolean _ = o instanceof R;
                                                       boolean _ = o instanceof R r;
                                                       boolean _ = o instanceof R(var v);
                                                   }
                                                   record R(int i) {}
                                               }
                                               """);
        JavacTask task = (JavacTask) compiler.getTask(null, null, null, null, null, List.of(input));
        CompilationUnitTree cut = task.parse().iterator().next();
        task.analyze();
        List<String> instanceOf = new ArrayList<>();
        new TreeScanner<Void, Void>() {
            @Override
            public Void visitInstanceOf(InstanceOfTree node, Void p) {
                instanceOf.add(node.getPattern() + ":" + node.getType());
                return super.visitInstanceOf(node, p);
            }
        }.scan(cut, null);
        List<String> expectedInstanceOf = List.of("null:R", "R r:R", "R(var v):null");
        if (!Objects.equals(expectedInstanceOf, instanceOf)) {
            throw new AssertionError("Expected: " + expectedInstanceOf + ",\ngot: " + instanceOf);
        }
    }
}
