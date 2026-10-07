import toolbox.ToolBox;
import toolbox.JavacTask;
import toolbox.Task;
import toolbox.TestRunner;

public class T8236490 extends TestRunner {
    ToolBox tb;
    public T8236490() {
        super(System.err);
        tb = new ToolBox();
    }
    public static void main(String[] args) throws Exception {
        new T8236490().runTests();
    }
    @Test
    public void testTypeAnnotationInCatchExpression() throws Exception {
        StringBuilder stringBuilder = new StringBuilder();
        stringBuilder.append("""
                import java.lang.annotation.ElementType;
                import java.lang.annotation.Target;
                public class Test8236490 {
                """);
        for (int i = 0; i < 300; i++) {
            stringBuilder.append("    private class Test" + i + " {}\n");
        }
        stringBuilder.append("""
                    @Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER})
                    private @interface AnnotationTest {}
                    public void test() {
                """);
        for (int i = 0; i < 300; i++) {
            stringBuilder.append("        Test" + i + " test" + i + " = new Test" + i + "();\n");
        }
        stringBuilder.append("""
                        try {
                            System.out.println("Hello");
                        } catch (@AnnotationTest Exception e) {}
                    }
                }
                """);
        new JavacTask(tb).sources(stringBuilder.toString()).outdir(".").run();
    }
}
