import java.lang.annotation.ElementType;
import java.lang.annotation.Target;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import toolbox.JavapTask;
import toolbox.Task;
import toolbox.ToolBox;

public class IncorrectCastOffsetTest {
    @Target(ElementType.TYPE_USE)
    @Retention(RetentionPolicy.RUNTIME) @interface TypeUse {
    }
    @Target(ElementType.TYPE_USE)
    @Retention(RetentionPolicy.RUNTIME) @interface TypeUse2 {
    }
    class AnnotatedCast1 {
        private static String checkcast(boolean test, Object obj, Object obj2) {
            return (String) (test ? obj : obj2);
        }
    }
    class AnnotatedCast2 {
        private static String checkcast(Object obj) {
            return (String) obj;
        }
    }
    class AnnotatedCast3 {
        private static String checkcast(boolean test, Object obj, Object obj2) {
            return (String) (test ? obj : obj2);
        }
    }
    class AnnotatedCast4 {
        private static String checkcast(Object obj) {
            return (String) (CharSequence) obj;
        }
    }
    ToolBox tb;
    IncorrectCastOffsetTest() {
        tb = new ToolBox();
    }
    public static void main(String[] args) {
        new IncorrectCastOffsetTest().run();
    }
    void run() {
        test("IncorrectCastOffsetTest$AnnotatedCast1.class", List.of("RuntimeVisibleTypeAnnotations:", "0: #24(): CAST, offset=9, type_index=0", "IncorrectCastOffsetTest$TypeUse"));
        test("IncorrectCastOffsetTest$AnnotatedCast2.class", List.of("RuntimeVisibleTypeAnnotations:", "0: #23(): CAST, offset=1, type_index=0", "IncorrectCastOffsetTest$TypeUse"));
        test("IncorrectCastOffsetTest$AnnotatedCast3.class", List.of("RuntimeVisibleTypeAnnotations:", "0: #24(): CAST, offset=9, type_index=0", "IncorrectCastOffsetTest$TypeUse", "1: #25(): CAST, offset=9, type_index=0", "IncorrectCastOffsetTest$TypeUse2"));
        test("IncorrectCastOffsetTest$AnnotatedCast4.class", List.of("RuntimeVisibleTypeAnnotations:", "0: #25(): CAST, offset=4, type_index=0", "IncorrectCastOffsetTest$TypeUse", "1: #26(): CAST, offset=1, type_index=0", "IncorrectCastOffsetTest$TypeUse2"));
    }
    void test(String clazz, List<String> expectedOutput) {
        Path pathToClass = Paths.get(ToolBox.testClasses, clazz);
        String javapOut = new JavapTask(tb).options("-v", "-p").classes(pathToClass.toString()).run().getOutput(Task.OutputKind.DIRECT);
        for (String expected : expectedOutput) {
            if (!javapOut.contains(expected)) {
                throw new AssertionError("unexpected output");
            }
        }
    }
}
