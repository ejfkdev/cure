import java.io.*;
import java.util.*;
import javax.tools.*;
import java.lang.classfile.*;
import java.lang.classfile.constantpool.*;

public class NoStringToLower {
    public static void main(String... args) throws Exception {
        NoStringToLower c = new NoStringToLower();
        if (c.run(args)) 
            return;
        if (is_jtreg()) 
            throw new Exception(c.errors + " errors occurred"); else 
            System.exit(1);
    }
    static boolean is_jtreg() {
        return System.getProperty("test.src") != null;
    }
    boolean run(String... args) throws Exception {
        JavaCompiler c = ToolProvider.getSystemJavaCompiler();
        try (JavaFileManager fm = c.getStandardFileManager(null, null, null)) {
            JavaFileManager.Location javacLoc = findJavacLocation(fm);
            String[] pkgs = {"javax.annotation.processing", "javax.lang.model", "javax.tools", "com.sun.source", "java.lang.classfile", "jdk.internal.classfile", "com.sun.tools.doclint", "com.sun.tools.javac", "com.sun.tools.javah", "com.sun.tools.javap", "com.sun.tools.jdeps", "jdk.javadoc"};
            for (String pkg : pkgs) {
                for (JavaFileObject fo : fm.list(javacLoc, pkg, EnumSet.of(JavaFileObject.Kind.CLASS), true)) {
                    scan(fo);
                }
            }
            return errors == 0;
        }
    }
    JavaFileManager.Location findJavacLocation(JavaFileManager fm) {
        JavaFileManager.Location[] locns = {StandardLocation.PLATFORM_CLASS_PATH, StandardLocation.CLASS_PATH};
        try {
            for (JavaFileManager.Location l : locns) {
                JavaFileObject fo = fm.getJavaFileForInput(l, "com.sun.tools.javac.Main", JavaFileObject.Kind.CLASS);
                if (fo != null) 
                    return l;
            }
        } catch (IOException e) {
            throw new Error(e);
        }
        throw new IllegalStateException("Cannot find javac");
    }
    void scan(JavaFileObject fo) throws IOException {
        try (InputStream in = fo.openInputStream()) {
            ClassModel cf = ClassFile.of().parse(in.readAllBytes());
            for (PoolEntry pe : cf.constantPool()) {
                if (pe instanceof MethodRefEntry ref) {
                    String methodDesc = ref.owner().name().stringValue() + "." + ref.name().stringValue() + ":" + ref.type().stringValue();
                    if ("java/lang/String.toLowerCase:()Ljava/lang/String;".equals(methodDesc)) {
                        error("found reference to String.toLowerCase() in: " + fo.getName());
                    }
                    if ("java/lang/String.toUpperCase:()Ljava/lang/String;".equals(methodDesc)) {
                        error("found reference to String.toLowerCase() in: " + fo.getName());
                    }
                }
            }
        } catch (ConstantPoolException ignore) {}
    }
    void error(String msg) {
        System.err.println("Error: " + msg);
        errors++;
    }
    int errors;
}
