import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.*;
import javax.tools.JavaCompiler;
import javax.tools.StandardJavaFileManager;
import javax.tools.ToolProvider;
import toolbox.*;

public class MissingAnnotationClassFile {
    public static void main(String[] args) throws Exception {
        ToolBox tb = new ToolBox();
        Path base = Paths.get(".");
        Path testSrc = base.resolve("test-src");
        tb.createDirectories(testSrc);
        Path libSrc = testSrc.resolve("lib-src");
        tb.createDirectories(libSrc);
        tb.writeJavaFiles(libSrc, "package lib;\npublic @interface I {\n    String value();\n}\n", "package lib;\n@I(\"Foo\")\npublic class Foo {}\n");
        Path libClasses = base.resolve("lib-classes");
        tb.createDirectories(libClasses);
        new JavacTask(tb).outdir(libClasses.toString()).sourcepath(libSrc.toString()).files(tb.findJavaFiles(libSrc)).run().writeAll();
        Files.delete(libClasses.resolve("lib/I.class"));
        tb.writeJavaFiles(testSrc, "import lib.Foo;\npublic class Bar {\n@lib.I(\"Bar\")\npublic void bar() {}\n}\n");
        Path testClasses = base.resolve("test-classes");
        tb.createDirectories(testClasses);
        Path bar = testSrc.resolve("Bar.java");
        JavaCompiler compiler = ToolProvider.getSystemJavaCompiler();
        List<String> errors = new ArrayList<>();
        try (StandardJavaFileManager fm = compiler.getStandardFileManager(null, null, null)) {
            com.sun.source.util.JavacTask task = (com.sun.source.util.JavacTask) compiler.getTask(null, null, (d) -> errors.add(d.getCode()), Arrays.asList("-XDrawDiagnostics", "-classpath", libClasses.toString()), null, fm.getJavaFileObjects(bar));
            task.parse();
            task.analyze();
            task.generate();
        }
        List<String> expected = Arrays.asList("compiler.err.cant.resolve.location");
        if (!expected.equals(errors)) {
            throw new IllegalStateException("Expected error not found!");
        }
    }
}
