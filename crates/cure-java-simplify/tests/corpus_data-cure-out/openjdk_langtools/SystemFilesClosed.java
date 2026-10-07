import java.io.BufferedReader;
import java.io.File;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.net.URI;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;
import javax.tools.JavaCompiler;
import javax.tools.JavaFileObject;
import javax.tools.SimpleJavaFileObject;
import javax.tools.StandardJavaFileManager;
import javax.tools.ToolProvider;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInfo;

public class SystemFilesClosed {
    private Path base;
    @Test void testSystemFilesClosed() throws Exception {
        if (!lsofCommand().isPresent()) {
            Assumptions.abort("lsof command is not available on this system");
        }
        String targetSystem = base.toString();
        int ret = java.util.spi.ToolProvider.findFirst("jlink").orElseThrow().run(System.out, System.err, "--add-modules", "java.base", "--output", targetSystem);
        if (ret != 0) {
            System.out.println("It is most probably an exploded build. Skip testing.");
            return;
        }
        JavaCompiler compiler = ToolProvider.getSystemJavaCompiler();
        SimpleJavaFileObject compilationUnit = new SimpleJavaFileObject(URI.create("string:///Test.java"), JavaFileObject.Kind.SOURCE) {
            @Override
            public CharSequence getCharContent(boolean ignoreEncodingErrors) {
                return """
               public class Test {
                    public static void main(String[] args) {
                        System.out.println("Hello, World!");
                    }
               }
               """;
            }
        };
        try (StandardJavaFileManager fileManager = compiler.getStandardFileManager(null, null, null)) {
            Assertions.assertEquals(true, compiler.getTask(null, fileManager, null, List.of("--system", targetSystem), null, List.of(compilationUnit)).call(), "Compilation task failed");
        }
        Process process = new ProcessBuilder().command(lsofCommand().orElseThrow(() -> new RuntimeException("lsof command is not available on this system")), "-p", String.valueOf(ProcessHandle.current().pid())).redirectOutput(ProcessBuilder.Redirect.PIPE).redirectError(ProcessBuilder.Redirect.INHERIT).start();
        List<String> lines;
        String realPath = base.toRealPath().toString();
        try (InputStream stdout = process.getInputStream(); BufferedReader reader = new BufferedReader(new InputStreamReader(stdout))) {
            lines = reader.lines().filter((line) -> line.contains(realPath)).toList();
        }
        process.waitFor();
        Assertions.assertEquals(0, lines.size(), "File(s) remain opened: " + lines);
    }
    @BeforeEach
    public void setUp(TestInfo info) {
        base = Paths.get(".").resolve(info.getTestMethod().orElseThrow().getName());
    }
    static Optional<String> lsofCommandCache = Arrays.stream(new String[] {"/usr/bin/lsof", "/usr/sbin/lsof", "/bin/lsof", "/sbin/lsof", "/usr/local/bin/lsof"}).filter((args) -> new File(args).exists()).findFirst();
    static Optional<String> lsofCommand() {
        return lsofCommandCache;
    }
}
