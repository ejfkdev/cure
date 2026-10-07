import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.util.Random;
import com.sun.tools.javac.Main;
import toolbox.JarTask;
import toolbox.JavacTask;
import toolbox.ToolBox;

public class T6558476 {
    private static final String classFoo = "class Foo {}";
    private static final String classMyFoo = "class MyFoo {\n  public static void main(String[] args) {\n    new Foo();\n  }\n" + "}";
    public static void main(String[] args) throws IOException {
        ToolBox tb = new ToolBox();
        new JavacTask(tb).sources(classFoo).run();
        new JarTask(tb, "foo.jar").files("Foo.class").run();
        new JavacTask(tb).classpath("foo.jar").sources(classMyFoo).run();
        File foo_jar = new File("foo.jar");
        if (foo_jar.delete()) {
            System.out.println("jar file successfully deleted");
        } else {
            throw new Error("Error deleting file \"" + foo_jar.getPath() + "\"");
        }
    }
}
