import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import toolbox.ToolBox;

public class SourceToSourceTranslationTest {
    public static void main(String[] args) throws Exception {
        ToolBox tb = new ToolBox();
        Path path1 = Paths.get(ToolBox.testClasses, "Universe.java");
        List<String> file1 = tb.readAllLines(path1);
        Path path2 = Paths.get(ToolBox.testSrc, "Universe.java.out");
        List<String> file2 = tb.readAllLines(path2);
        tb.checkEqual(file1, file2);
    }
}
