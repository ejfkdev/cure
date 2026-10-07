import java.io.*;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import java.util.jar.JarEntry;
import javax.tools.*;

public class T8271079 {
    public static void main(String[] args) throws Exception {
        new T8271079().run();
    }
    final PrintStream out;
    T8271079() {
        this.out = System.out;
    }
    void run() throws Exception {
        Path mr = generateMultiReleaseJar();
        try {
            testT8271079(mr);
        } finally {
            Files.deleteIfExists(mr);
        }
    }
    Path generateMultiReleaseJar() throws Exception {
        Files.writeString(Path.of("module-info.java"), "module hello {}");
        java.util.spi.ToolProvider.findFirst("javac").orElseThrow().run(out, System.err, "-d", "classes", "--release", "9", "module-info.java");
        Path mr = Path.of("mr.jar");
        java.util.spi.ToolProvider.findFirst("jar").orElseThrow().run(out, System.err, "--create", "--file", mr.toString(), "--release", "9", "-C", "classes", ".");
        out.println("Created: " + mr.toUri());
        out.println(" Exists: " + Files.exists(mr));
        return mr;
    }
    void testT8271079(Path path) throws Exception {
        StandardJavaFileManager fileManager = ToolProvider.getSystemJavaCompiler().getStandardFileManager(null, Locale.ENGLISH, StandardCharsets.UTF_8);
        fileManager.setLocationFromPaths(StandardLocation.CLASS_PATH, List.of(path));
        Iterator<String> options = Arrays.asList("--multi-release", "9").iterator();
        fileManager.handleOption(options.next(), options);
        Iterable<JavaFileObject> list = fileManager.list(StandardLocation.CLASS_PATH, "", EnumSet.allOf(JavaFileObject.Kind.class), false);
        for (JavaFileObject f : list) {
            out.println("JavaFileObject#getName: " + f.getName());
            out.println("JavaFileObject#toUri: " + f.toUri());
            openUsingUri(f.toUri());
        }
        System.gc();
    }
    void openUsingUri(URI uri) throws IOException {
        URLConnection connection = uri.toURL().openConnection();
        connection.setUseCaches(false);
        if (connection instanceof JarURLConnection jar) {
            try {
                JarEntry entry = jar.getJarEntry();
                out.println("JarEntry#getName: " + entry.getName());
                connection.getInputStream().close();
            } catch (FileNotFoundException e) {
                throw e;
            }
        }
    }
}
