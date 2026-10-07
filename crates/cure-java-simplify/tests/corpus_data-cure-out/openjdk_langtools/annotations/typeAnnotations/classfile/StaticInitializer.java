import java.lang.annotation.*;
import java.util.ArrayList;
import java.lang.classfile.*;

public class StaticInitializer extends ClassfileTestHelper {
    public static void main(String[] args) throws Exception {
        new StaticInitializer().run();
    }
    public void run() throws Exception {
        expected_tinvisibles = 4;
        expected_tvisibles = 0;
        ClassModel cm = getClassFile("StaticInitializer$Test.class");
        test(cm);
        for (FieldModel fm : cm.fields()) {
            test(fm);
        }
        for (MethodModel mm : cm.methods()) {
            test(mm, true);
        }
        countAnnotations();
        if (errors > 0) 
            throw new Exception(errors + " errors found");
        System.out.println("PASSED");
    }
    static class Test {
        @Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface T {
        }
        static {
            @T String s = null;
            Runnable r = () -> new ArrayList<@T String>();
        }
        @T static String s = null;
        static Runnable r = () -> new ArrayList<@T String>();
    }
}
