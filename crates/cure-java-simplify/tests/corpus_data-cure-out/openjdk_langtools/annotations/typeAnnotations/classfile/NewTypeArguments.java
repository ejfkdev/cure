import java.lang.classfile.*;
import java.lang.annotation.*;
import java.io.*;
import java.net.URL;
import java.util.List;

public class NewTypeArguments extends ClassfileTestHelper {
    public static void main(String[] args) throws Exception {
        new NewTypeArguments().run();
    }
    public void run() throws Exception {
        ClassModel cm = getClassFile("NewTypeArguments$Test.class");
        test(cm);
        for (FieldModel fm : cm.fields()) {
            test(fm);
        }
        for (MethodModel m : cm.methods()) {
            test(m, true);
        }
        countAnnotations();
        if (errors > 0) 
            throw new Exception(errors + " errors found");
        System.out.println("PASSED");
    }
    static class Test {
        @Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface A {
        }
        <E> Test(E e) {}
        void test() {
            new <@A String> Test(null);
            new <@A List<@A String>> Test(null);
        }
    }
}
