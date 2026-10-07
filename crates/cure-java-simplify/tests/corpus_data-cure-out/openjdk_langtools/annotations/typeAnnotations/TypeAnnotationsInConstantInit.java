import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import toolbox.JavacTask;
import toolbox.ToolBox;

public class TypeAnnotationsInConstantInit {
    public static void main(String... args) throws Exception {
        new TypeAnnotationsInConstantInit().run();
    }
    ToolBox tb = new ToolBox();
    void run() throws Exception {
        typeAnnotationInConstantExpressionFieldInit(Paths.get("."));
    }
    void typeAnnotationInConstantExpressionFieldInit(Path base) throws Exception {
        Path src = base.resolve("src");
        Path classes = base.resolve("classes");
        tb.writeJavaFiles(src, """
                          import java.lang.annotation.*;

                          @SuppressWarnings(Decl.VALUE)
                          public class Decl {
                              public static final @Nullable String VALUE = (@Nullable String) "";
                          }

                          @Retention(RetentionPolicy.RUNTIME)
                          @Target({ ElementType.TYPE_USE })
                          @interface Nullable {}
                          """);
        Files.createDirectories(classes);
        new JavacTask(tb).options("-d", classes.toString()).files(tb.findJavaFiles(src)).run().writeAll();
    }
}
