import com.sun.source.tree.ClassTree;
import com.sun.source.util.TaskEvent;
import com.sun.source.util.TaskEvent.Kind;
import com.sun.source.util.TaskListener;
import com.sun.source.util.TreePathScanner;
import com.sun.source.util.Trees;
import java.nio.file.Files;
import toolbox.*;
import java.nio.file.Path;
import java.nio.file.Paths;
import javax.lang.model.element.AnnotationMirror;
import javax.lang.model.element.AnnotationValue;
import javax.lang.model.element.Element;
import javax.lang.model.util.Elements;
import javax.lang.model.util.SimpleAnnotationValueVisitorPreview;

public class TestAnnotationValuesResolved extends TestRunner {
    final toolbox.ToolBox tb = new ToolBox();
    public TestAnnotationValuesResolved() {
        super(System.err);
    }
    public static void main(String[] args) throws Exception {
        new TestAnnotationValuesResolved().runTests();
    }
    protected void runTests() throws Exception {
        runTests((m) -> new Object[] {Path.of(m.getName())});
    }
    @Test
    public void test(Path base) throws Exception {
        Path lib = Paths.get("lib");
        Path libSrc = lib.resolve("src");
        Path libClasses = lib.resolve("classes");
        tb.writeJavaFiles(libSrc, """
                          package org.example;

                          public @interface MyFirstAnnotation {
                              MySecondAnnotation secondAnnotation() default @MySecondAnnotation;
                          }
                          """, """
                          package org.example;

                          public @interface MySecondAnnotation {
                              String[] stringArray() default "";
                          }
                          """);
        Files.createDirectories(libClasses);
        new toolbox.JavacTask(tb).outdir(libClasses).files(tb.findJavaFiles(libSrc)).run();
        Path test = Paths.get("test");
        Path testSrc = test.resolve("src");
        Path testClasses = test.resolve("classes");
        tb.writeJavaFiles(testSrc, """
                          package org.example;

                          @MyFirstAnnotation
                          public class AnnotatedClass {
                          }
                          """);
        Files.createDirectories(testClasses);
        new toolbox.JavacTask(tb).classpath(libClasses).outdir(testClasses).files(tb.findJavaFiles(testSrc)).callback((task) -> {
            task.addTaskListener(new TaskListener() {
                        @Override
                        public void finished(TaskEvent e) {
                            if (e.getKind() == Kind.ENTER) {
                                new TreePathScanner<>() {
                                    @Override
                                    public Object visitClass(ClassTree node, Object p) {
                                        Trees trees = Trees.instance(task);
                                        Element el = trees.getElement(getCurrentPath());
                                        verifyAnnotationValuesResolved(task, el);
                                        return super.visitClass(node, p);
                                    }
                                }.scan(e.getCompilationUnit(), null);
                            }
                        }
                    });
        }).run().writeAll();
    }
    private void verifyAnnotationValuesResolved(com.sun.source.util.JavacTask task, Element forElement) {
        Elements elements = task.getElements();
        class SearchAnnotationValues extends SimpleAnnotationValueVisitorPreview {
                    @Override
                    public Object visitAnnotation(AnnotationMirror a, Object p) {
                        for (AnnotationValue av : elements.getElementValuesWithDefaults(a).values()) {
                            av.accept(this, null);
                        }
                        return super.visitAnnotation(a, p);
                    }
                }

                for (AnnotationMirror mirror : forElement.getAnnotationMirrors()) {
                    new SearchAnnotationValues().visitAnnotation(mirror, null);
                }
    }
}
