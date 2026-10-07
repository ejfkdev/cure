import com.sun.source.util.JavacTask;
import java.util.Arrays;
import javax.tools.Diagnostic;
import javax.tools.JavaCompiler;
import javax.tools.JavaFileObject;
import javax.tools.SimpleJavaFileObject;
import javax.tools.ToolProvider;

public class Abort {
    public static void main(String... args) throws Exception {
        String SCRATCH_DIR = System.getProperty("user.dir");
        JavaCompiler javacTool = ToolProvider.getSystemJavaCompiler();
        java.io.File testDir = new java.io.File(SCRATCH_DIR);
        sourceA.dumpTo(testDir);
        sourceB.dumpTo(testDir);
        DiagnosticChecker diagChecker = new DiagnosticChecker();
        JavacTask ct = (JavacTask) javacTool.getTask(null, null, diagChecker, Arrays.asList("-XDrawDiagnostics", "-cp", testDir.getAbsolutePath()), null, Arrays.asList(sourceA.asJFO(testDir)));
        try {
            ct.analyze();
        } catch (Throwable ex) {}
        if (!diagChecker.errorFound) {
            throw new AssertionError("Missing diagnostic");
        }
    }
    static class JavaSource {
        String contents;
        String filename;
        public JavaSource(String filename, String contents) {
            this.filename = filename;
            this.contents = contents;
        }
        void dumpTo(java.io.File loc) throws Exception {
            java.io.BufferedWriter bw = new java.io.BufferedWriter(new java.io.FileWriter(new java.io.File(loc, filename)));
            bw.append(contents);
            bw.close();
        }
        JavaFileObject asJFO(java.io.File dir) {
            return SimpleJavaFileObject.forSource(new java.io.File(dir, filename).toURI(), contents);
        }
    }
    static JavaSource sourceA = new JavaSource("Abort.java", "public class Abort {\n    public static void main(String[] args) {\n        System.out.println(C.m());\n    }\n}");
    static JavaSource sourceB = new JavaSource("C.java", "package com.example;\npublic class C {\n    public static String m() { return null; }\n}");
    static class DiagnosticChecker implements javax.tools.DiagnosticListener<JavaFileObject> {
        boolean errorFound;
        public void report(Diagnostic<? extends JavaFileObject> diagnostic) {
            if (diagnostic.getKind() == Diagnostic.Kind.ERROR && diagnostic.getCode().contains("compiler.err.cant.access")) {
                errorFound = true;
            }
        }
    }
}
