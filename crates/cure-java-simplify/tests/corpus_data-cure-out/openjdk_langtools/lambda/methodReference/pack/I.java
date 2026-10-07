package pack;

import java.nio.file.Path;

public class I {
    protected String readFile(Path file) {
        return file.toString();
    }
    protected static String readFile2(Path file) {
        return file.toString();
    }
}
