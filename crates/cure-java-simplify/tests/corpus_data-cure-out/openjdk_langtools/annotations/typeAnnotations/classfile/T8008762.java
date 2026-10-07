import java.lang.annotation.*;
import static java.lang.annotation.RetentionPolicy.*;
import static java.lang.annotation.ElementType.*;
import java.lang.classfile.*;

public class T8008762 extends ClassfileTestHelper {
    public static void main(String[] args) throws Exception {
        new T8008762().run();
    }
    public void run() throws Exception {
        expected_tinvisibles = 0;
        expected_tvisibles = 4;
        ClassModel cm = getClassFile("T8008762$Test$1$InnerAnon.class");
        test(cm);
        for (FieldModel fm : cm.fields()) {
            test(fm, false);
        }
        for (MethodModel mm : cm.methods()) {
            test(mm, false);
        }
        countAnnotations();
        if (errors > 0) 
            throw new Exception(errors + " errors found");
        System.out.println("PASSED");
    }
    static class Test {
        Object mtest(Test t) {
            return null;
        }
        public void test() {
            mtest(new Test() {
                class InnerAnon { // Test1$1$InnerAnon.class
                  @A @B String ai_data = null;
                  @A @B String ai_m(){ return null; };
                }
               InnerAnon IA = new InnerAnon();
            });
        }
        @Retention(RUNTIME) @Target(TYPE_USE) @interface A {
        }
        @Retention(RUNTIME) @Target(TYPE_USE) @interface B {
        }
    }
}
