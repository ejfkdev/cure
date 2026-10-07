import java.io.File;
import java.io.IOException;
import java.util.Collections;
import java.util.Date;
import java.util.jar.JarEntry;
import java.util.jar.JarFile;
import javax.tools.*;
import com.sun.tools.javac.file.JavacFileManager;
import com.sun.tools.javac.file.RelativePath.RelativeFile;
import com.sun.tools.javac.util.Context;
import toolbox.JarTask;
import toolbox.ToolBox;

public class T6725036 {
    public static void main(String... args) throws Exception {
        new T6725036().run();
    }
    void run() throws Exception {
        RelativeFile TEST_ENTRY_NAME = new RelativeFile("java/lang/String.class");
        File testJar = createJar("test.jar", "java.lang.*");
        try (JarFile j = new JarFile(testJar)) {
            JarEntry je = j.getJarEntry(TEST_ENTRY_NAME.getPath());
            long jarEntryTime = je.getTime();
            JavacFileManager fm = new JavacFileManager(new Context(), false, null);
            fm.setLocation(StandardLocation.CLASS_PATH, Collections.singletonList(testJar));
            FileObject fo = fm.getFileForInput(StandardLocation.CLASS_PATH, "", TEST_ENTRY_NAME.getPath());
            check(je, jarEntryTime, fo, fo.getLastModified());
            if (errors > 0) 
                throw new Exception(errors + " occurred");
        }
    }
    File createJar(String name, String... paths) throws IOException {
        JavaCompiler comp = ToolProvider.getSystemJavaCompiler();
        try (JavaFileManager fm = comp.getStandardFileManager(null, null, null)) {
            File f = new File(name);
            new JarTask(new ToolBox(), name).files(fm, StandardLocation.PLATFORM_CLASS_PATH, paths).run();
            return f;
        }
    }
    void check(Object ref, long refTime, Object test, long testTime) {
        if (refTime == testTime) 
            return;
        System.err.println("Error: ");
        System.err.println("Expected: " + getText(ref, refTime));
        System.err.println("   Found: " + getText(test, testTime));
        errors++;
    }
    String getText(Object x, long t) {
        return String.format("%14d", t) + " (" + new Date(t) + ") from " + x;
    }
    int errors;
}
