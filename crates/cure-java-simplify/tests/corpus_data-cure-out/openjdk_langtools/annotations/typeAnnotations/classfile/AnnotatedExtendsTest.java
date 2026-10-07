import java.nio.file.Path;
import java.nio.file.Paths;
import java.lang.annotation.ElementType;
import java.lang.annotation.Target;
import toolbox.JavapTask;
import toolbox.Task;
import toolbox.ToolBox;

public class AnnotatedExtendsTest {
    @Target(ElementType.TYPE_USE)
    public @interface TA {
    }
    public class Inner extends @TA Object {
    }
    public static strictfp void main(String[] args) throws Exception {
        ToolBox tb = new ToolBox();
        Path classPath = Paths.get(ToolBox.testClasses, "AnnotatedExtendsTest$Inner.class");
        String javapOut = new JavapTask(tb).options("-v", "-p").classes(classPath.toString()).run().getOutput(Task.OutputKind.DIRECT);
        if (!javapOut.contains("0: #27(): CLASS_EXTENDS, type_index=65535")) 
            throw new AssertionError("Expected output missing: " + javapOut);
    }
}
