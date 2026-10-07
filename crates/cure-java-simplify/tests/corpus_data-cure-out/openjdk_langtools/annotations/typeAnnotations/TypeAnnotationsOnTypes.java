import com.sun.source.tree.VariableTree;
import com.sun.source.util.TaskEvent;
import com.sun.source.util.TaskListener;
import com.sun.source.util.TreePath;
import com.sun.source.util.TreePathScanner;
import com.sun.source.util.Trees;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;
import javax.lang.model.type.TypeKind;
import javax.lang.model.type.TypeMirror;
import javax.lang.model.type.UnionType;
import toolbox.JavacTask;
import toolbox.ToolBox;

public class TypeAnnotationsOnTypes {
    public static void main(String... args) throws Exception {
        new TypeAnnotationsOnTypes().run();
    }
    ToolBox tb = new ToolBox();
    void run() throws Exception {
        typeAnnotationInConstantExpressionFieldInit(Paths.get("."));
    }
    void typeAnnotationInConstantExpressionFieldInit(Path base) throws Exception {
        Path src = base.resolve("src");
        Path classes = base.resolve("classes");
        tb.writeJavaFiles(src, """
                          import java.lang.annotation.ElementType;
                          import java.util.List;
                          import java.lang.annotation.Target;

                          class Test<TypeVar> {

                              void f() {
                                  @TA List<String> l1;
                                  @TA String[] l2;
                                  @TA TypeVar l3;
                                  try {
                                  } catch (@TA IllegalStateException |  NullPointerException | @TA IllegalArgumentException ex) {}
                              }

                              @Target(ElementType.TYPE_USE)
                              @interface TA {}
                          }
                          """);
        Files.createDirectories(classes);
        List<String> actual = new ArrayList<>();
        new JavacTask(tb).options("-d", classes.toString()).files(tb.findJavaFiles(src)).callback((task) -> {
            task.addTaskListener(new TaskListener() {
                        @Override
                        public void finished(TaskEvent e) {
                            if (e.getKind() != TaskEvent.Kind.ANALYZE) {
                                return ;
                            }
                            Trees trees = Trees.instance(task);
                            new TreePathScanner<Void, Void>() {
                                @Override
                                public Void visitVariable(VariableTree node, Void p) {
                                    TreePath typePath =
                                            new TreePath(getCurrentPath(), node.getType());
                                    actual.add(node.getName() +
                                               ": type on variable: " +
                                               typeToString(trees.getTypeMirror(getCurrentPath())) +
                                               ": type on type: " +
                                               typeToString(trees.getTypeMirror(typePath)));
                                    return super.visitVariable(node, p);
                                }
                            }.scan(e.getCompilationUnit(), null);
                        }
                    });
        }).run().writeAll();
        List<String> expected = List.of("l1: type on variable: java.util.@Test.TA List<java.lang.String>: type on type: java.util.@Test.TA List<java.lang.String>", "l2: type on variable: java.lang.@Test.TA String[]: type on type: java.lang.@Test.TA String[]", "l3: type on variable: @Test.TA TypeVar: type on type: @Test.TA TypeVar", "ex: type on variable: java.lang.@Test.TA IllegalStateException | java.lang.NullPointerException | java.lang.@Test.TA IllegalArgumentException: type on type: java.lang.@Test.TA IllegalStateException | java.lang.NullPointerException | java.lang.@Test.TA IllegalArgumentException");
        actual.forEach(System.out::println);
        if (!expected.equals(actual)) {
            throw new AssertionError("Expected: " + expected + ", but got: " + actual);
        }
    }
    static String typeToString(TypeMirror type) {
        return type != null && type.getKind() == TypeKind.UNION ? ((UnionType) type).getAlternatives().stream().map((t) -> typeToString(t)).collect(Collectors.joining(" | ")) : String.valueOf(type);
    }
}
