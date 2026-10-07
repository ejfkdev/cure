import java.lang.classfile.*;
import java.lang.annotation.*;
import java.util.ArrayList;

public class InstanceInitializer extends ClassfileTestHelper {
    public static void main(String[] args) throws Exception {
        new InstanceInitializer().run();
    }
    public void run() throws Exception {
        ClassModel cm = getClassFile("InstanceInitializer$Test.class");
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
        {
            @T String s = null;
            Runnable r = () -> new ArrayList<@T String>();
        }
        @T String s = null;
        Runnable r = () -> new ArrayList<@T String>();
    }
}
