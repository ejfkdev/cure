import java.nio.file.Path;
import java.nio.file.Paths;
import java.lang.annotation.ElementType;
import java.lang.annotation.Target;
import toolbox.JavapTask;
import toolbox.Task;
import toolbox.ToolBox;

public class NestedLambdasCastedTest {
    static class ExpectedOutputHolder {
        public String[] outputs = {"public static strictfp void main(java.lang.String[])", "private static strictfp void lambda$main$3();", "private static strictfp void lambda$main$2();", "private static strictfp void lambda$main$1();", "private static strictfp void lambda$main$0();", "0: #111(#112=s#113): CAST, offset=5, type_index=0", "0: #111(#112=s#119): CAST, offset=5, type_index=0", "0: #111(#112=s#122): CAST, offset=5, type_index=0", "0: #111(#112=s#125): CAST, offset=5, type_index=0"};
    }
    @Target(ElementType.TYPE_USE)
    public @interface TA {
        String value() default "";
    }
    public static strictfp void main(String[] args) throws Exception {
        Runnable one = (Runnable) (() -> {
            Runnable two = (Runnable) (() -> {
                Runnable three = (Runnable) (() -> {
                    Runnable four = (Runnable) (() -> {});
                });
            });
        });
        ToolBox tb = new ToolBox();
        Path classPath = Paths.get(ToolBox.testClasses, "NestedLambdasCastedTest.class");
        String javapOut = new JavapTask(tb).options("-v", "-p").classes(classPath.toString()).run().getOutput(Task.OutputKind.DIRECT);
        ExpectedOutputHolder holder = new ExpectedOutputHolder();
        for (String s : holder.outputs) {
            if (!javapOut.contains(s)) 
                throw new AssertionError("Expected type annotation on LOCAL_VARIABLE missing");
        }
    }
}
