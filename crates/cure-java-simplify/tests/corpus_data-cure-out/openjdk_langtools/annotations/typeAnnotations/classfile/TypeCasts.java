import java.lang.annotation.*;
import java.io.*;
import java.net.URL;
import java.util.List;
import java.lang.classfile.*;

public class TypeCasts extends ClassfileTestHelper {
    public static void main(String[] args) throws Exception {
        new TypeCasts().run();
    }
    public void run() throws Exception {
        ClassModel cm = getClassFile("TypeCasts$Test.class");
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
        void emit() {
            Object o = null;
            String s = null;
            String a0 = (String) o;
            Object a1 = (Object) o;
            String b0 = (String) s;
            Object b1 = (Object) s;
        }
        void alldeadcode() {}
    }
}
