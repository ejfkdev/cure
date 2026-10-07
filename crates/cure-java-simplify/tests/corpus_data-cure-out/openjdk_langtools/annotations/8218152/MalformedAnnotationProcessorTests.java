import java.io.RandomAccessFile;
import java.nio.ByteBuffer;
import java.nio.channels.FileChannel;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import javax.annotation.processing.Processor;
import toolbox.JavacTask;
import toolbox.Task;
import toolbox.Task.Expect;
import toolbox.TestRunner;
import toolbox.ToolBox;

public class MalformedAnnotationProcessorTests extends TestRunner {
    public static void main(String... args) throws Exception {
        new MalformedAnnotationProcessorTests().runTests((m) -> new Object[] {Paths.get(m.getName())});
    }
    private ToolBox tb = new ToolBox();
    public MalformedAnnotationProcessorTests() {
        super(System.err);
    }
    @Test
    public void testBadAnnotationProcessor(Path base) throws Exception {
        Path apDir = base.resolve("annoprocessor");
        tb.writeFile(apDir.resolve("META-INF").resolve("services").resolve(Processor.class.getCanonicalName()), "BadAnnoProcessor");
        tb.writeFile(apDir.resolve("BadAnnoProcessor.class"), "badannoprocessor");
        Path src = base.resolve("src");
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        tb.writeJavaFiles(src, "package test; public class Test {}");
        List<String> actualErrors = new JavacTask(tb).options("-XDrawDiagnostics", "-classpath", "", "-sourcepath", src.toString(), "-processorpath", apDir.toString()).outdir(classes).files(tb.findJavaFiles(src)).run(Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        System.out.println(actualErrors.get(0));
        if (!actualErrors.get(0).contains("- compiler.err.proc.bad.config.file: javax.annotation.processing.Processor: Provider BadAnnoProcessor not found")) {
            throw new AssertionError("Unexpected errors reported: " + actualErrors);
        }
    }
    @Test
    public void testMissingAnnotationProcessor(Path base) throws Exception {
        Path apDir = base.resolve("annoprocessor");
        tb.writeFile(apDir.resolve("META-INF").resolve("services").resolve(Processor.class.getCanonicalName()), "MissingAnnoProcessor");
        Path src = base.resolve("src");
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        tb.writeJavaFiles(src, "package test; public class Test {}");
        List<String> actualErrors = new JavacTask(tb).options("-XDrawDiagnostics", "-classpath", "", "-sourcepath", src.toString(), "-processorpath", apDir.toString(), "-Xlint:-options").outdir(classes).files(tb.findJavaFiles(src)).run(Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        if (!actualErrors.get(0).contains("- compiler.err.proc.bad.config.file: javax.annotation.processing.Processor: Provider MissingAnnoProcessor not found")) {
            throw new AssertionError("Unexpected errors reported: " + actualErrors);
        }
    }
    @Test
    public void testWrongClassFileVersion(Path base) throws Exception {
        Path apDir = base.resolve("ap");
        tb.writeFile(apDir.resolve("META-INF").resolve("services").resolve(Processor.class.getCanonicalName()), "WrongClassFileVersion");
        new JavacTask(tb).outdir(apDir).sources("class WrongClassFileVersion {}").run().writeAll();
        increaseMajor(apDir.resolve("WrongClassFileVersion.class"), 1);
        Path src = base.resolve("src");
        Path classes = base.resolve("classes");
        Files.createDirectories(classes);
        tb.writeJavaFiles(src, "package test; public class Test {}");
        List<String> actualErrors = new JavacTask(tb).options("-XDrawDiagnostics", "-classpath", "", "-sourcepath", src.toString(), "-processorpath", apDir.toString()).outdir(classes).files(tb.findJavaFiles(src)).run(Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        if (!actualErrors.get(0).contains("- compiler.err.proc.bad.config.file: javax.annotation.processing.Processor: Provider WrongClassFileVersion not found")) {
            throw new AssertionError("Unexpected errors reported: " + actualErrors);
        }
    }
    static void increaseMajor(Path cfile, int delta) {
        try (RandomAccessFile cls = new RandomAccessFile(cfile.toFile(), "rw"); FileChannel fc = cls.getChannel()) {
            ByteBuffer rbuf = ByteBuffer.allocate(2);
            fc.read(rbuf, 6);
            ByteBuffer wbuf = ByteBuffer.allocate(2);
            wbuf.putShort(0, (short) (rbuf.getShort(0) + delta));
            fc.write(wbuf, 6);
            fc.force(false);
        } catch (Exception e) {
            throw new RuntimeException("Failed: unexpected exception");
        }
    }
}
