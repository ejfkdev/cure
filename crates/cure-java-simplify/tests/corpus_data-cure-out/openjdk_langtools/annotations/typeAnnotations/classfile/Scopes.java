import java.lang.annotation.*;
import java.io.*;
import java.net.URL;
import java.util.List;
import java.lang.classfile.*;
import java.lang.classfile.attribute.*;

@Scopes.UniqueInner
public class Scopes<T extends @Scopes.UniqueInner Object> extends ClassfileTestHelper {
    public static void main(String[] args) throws Exception {
        new Scopes().run();
    }
    public void run() throws Exception {
        test(getClassFile("Scopes.class"));
        countAnnotations();
        if (errors > 0) 
            throw new Exception(errors + " errors found");
        System.out.println("PASSED");
    }
    @Target({ElementType.TYPE_USE}) @interface UniqueInner {
    }
}
