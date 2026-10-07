import java.net.URL;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import toolbox.JavapTask;
import toolbox.Task;
import toolbox.ToolBox;

public class ConstFoldTest {
    public static void main(String... args) throws Exception {
        new ConstFoldTest().run();
    }
    class CFTest {
        void m() {
            boolean b = true;
        }
    }
    final String regex = "\\sif(?:null|nonnull|eq|ne){1}\\s";
    void run() throws Exception {
        ToolBox tb = new ToolBox();
        URL url = ConstFoldTest.class.getResource("ConstFoldTest$CFTest.class");
        Path file = Paths.get(url.toURI());
        List<String> result = new JavapTask(tb).options("-c").classes(file.toString()).run().write(Task.OutputKind.DIRECT).getOutputLines(Task.OutputKind.DIRECT);
        List<String> bad_codes = tb.grep(regex, result);
        if (!bad_codes.isEmpty()) {
            for (String code : bad_codes) 
                System.out.println("Bad OpCode Found: " + code);
            throw new Exception("constant folding failed");
        }
    }
}
