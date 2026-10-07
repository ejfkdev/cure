import java.util.List;
import java.util.Arrays;
import toolbox.ToolBox;
import toolbox.TestRunner;
import toolbox.JavacTask;
import toolbox.Task;

public class TestParameterAnnotationOnReceiverType extends TestRunner {
    ToolBox tb;
    public TestParameterAnnotationOnReceiverType() {
        super(System.err);
        tb = new ToolBox();
    }
    public static void main(String[] args) throws Exception {
        new TestParameterAnnotationOnReceiverType().runTests();
    }
    @Test
    public void testReceiverTypeDoesNotCauseError() throws Exception {
        List<String> output = new JavacTask(tb).sources("""
                import java.lang.annotation.ElementType;
                import java.lang.annotation.Retention;
                import java.lang.annotation.RetentionPolicy;
                import java.lang.annotation.Target;
                class Test8239596 {
                    @Retention(RetentionPolicy.RUNTIME)
                    @Target({ElementType.TYPE_USE})
                    @interface TypeUse { }

                    @Retention(RetentionPolicy.RUNTIME)
                    @Target({ElementType.PARAMETER})
                    @interface Param { }

                    public void test(@TypeUse @Param Test8239596 this) { }
                }""").classpath(".").options("-XDrawDiagnostics").run(Task.Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        List<String> expected = Arrays.asList("Test8239596.java:14:31: compiler.err.annotation.type.not.applicable.to.type: Test8239596.Param", "1 error");
        tb.checkEqual(expected, output);
    }
}
