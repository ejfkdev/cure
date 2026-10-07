import java.lang.annotation.*;
import java.io.*;
import java.net.URL;
import java.util.List;
import java.lang.classfile.*;

public class DeadCode extends ClassfileTestHelper {
    public static void main(String[] args) throws Exception {
        new DeadCode().run();
    }
    public void run() throws Exception {
        ClassModel cm = getClassFile("DeadCode$Test.class");
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
        @Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface A {
        }
        void test() {
            List<? extends @A Object> o = null;
            o.toString();
            @A String m;
        }
    }
}
