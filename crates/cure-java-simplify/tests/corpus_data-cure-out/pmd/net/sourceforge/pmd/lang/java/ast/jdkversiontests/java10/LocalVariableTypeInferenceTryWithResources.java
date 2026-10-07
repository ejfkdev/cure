import java.io.FileInputStream;

public class LocalVariableTypeInferenceTryWithResources {
    public void aMethod() throws Exception {
        try (var in = new FileInputStream("file.txt")) {}
    }
}
