import java.io.*;
import java.util.*;
import javax.annotation.processing.*;
import javax.lang.model.element.*;
import javax.tools.*;
import com.sun.tools.javac.api.JavacTaskImpl;
import com.sun.tools.javac.api.JavacTool;
import com.sun.tools.javac.file.JavacFileManager;
import com.sun.tools.javac.main.JavaCompiler;
import com.sun.tools.javac.util.Context;

@SupportedAnnotationTypes("*")
public class T6358166 extends AbstractProcessor {
    public static void main(String... args) throws Throwable {
        String self = T6358166.class.getName();
        String testSrc = System.getProperty("test.src");
        JavacFileManager fm = new JavacFileManager(new Context(), false, null);
        JavaFileObject f = fm.getJavaFileObject(testSrc + File.separatorChar + self + ".java");
        List<String> addExports = Arrays.asList("--add-exports", "jdk.compiler/com.sun.tools.javac.api=ALL-UNNAMED", "--add-exports", "jdk.compiler/com.sun.tools.javac.file=ALL-UNNAMED", "--add-exports", "jdk.compiler/com.sun.tools.javac.main=ALL-UNNAMED", "--add-exports", "jdk.compiler/com.sun.tools.javac.util=ALL-UNNAMED");
        test(fm, f, addExports, "-verbose", "-d", ".");
        test(fm, f, addExports, "-verbose", "-d", ".", "-XprintRounds", "-processorpath", ".", "-processor", self);
    }
    static void test(JavacFileManager fm, JavaFileObject f, List<String> addExports, String... args) throws Throwable {
        List<String> allArgs = new ArrayList<>();
        allArgs.addAll(addExports);
        allArgs.addAll(Arrays.asList(args));
        Context context = new Context();
        JavacTaskImpl task = (JavacTaskImpl) JavacTool.create().getTask(null, fm, null, allArgs, null, List.of(f), context);
        if (!task.call()) {
            throw new AssertionError("test failed due to a compilation error");
        }
        JavaCompiler c = JavaCompiler.instance(context);
        if (c.errorCount() != 0) 
            throw new AssertionError("compilation failed");
        long msec = c.elapsed_msec;
        if (msec < 0 || msec > 300000) 
            throw new AssertionError("elapsed time is suspect: " + msec);
    }
    public boolean process(Set<? extends TypeElement> tes, RoundEnvironment renv) {
        return true;
    }
}
