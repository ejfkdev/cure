import toolbox.JavacTask;
import toolbox.Task;
import toolbox.TestRunner;
import toolbox.ToolBox;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Arrays;
import java.util.List;

public class RichFormatterWithTypeAnnotationsReentrantTest extends TestRunner {
    ToolBox tb;
    public RichFormatterWithTypeAnnotationsReentrantTest() {
        super(System.err);
        tb = new ToolBox();
    }
    public static void main(String[] args) throws Exception {
        new RichFormatterWithTypeAnnotationsReentrantTest().runTests((m) -> new Object[] {Paths.get(m.getName())});
    }
    @Test
    public void test(Path base) throws Exception {
        Path libClasses = base.resolve("libclasses");
        Files.createDirectories(libClasses);
        new JavacTask(tb).outdir(libClasses).sources("""
                        package lib;
                        enum Bar {
                          BAZ
                        }
                        """, """
                        package lib;
                        import java.lang.annotation.ElementType;
                        import java.lang.annotation.Retention;
                        import java.lang.annotation.RetentionPolicy;
                        import java.lang.annotation.Target;

                        @Retention(RetentionPolicy.RUNTIME)
                        @interface Foo {
                          Bar value();
                        }
                        """, """
                        package lib;
                        import java.lang.annotation.ElementType;
                        import java.lang.annotation.Retention;
                        import java.lang.annotation.RetentionPolicy;
                        import java.lang.annotation.Target;

                        @Retention(RetentionPolicy.RUNTIME)
                        @Target({ElementType.TYPE_USE, ElementType.PARAMETER})
                        @Foo(Bar.BAZ)
                        @interface A {}
                        """, """
                        package lib;
                        public interface M {
                          String f(@A String k, @A String v);
                        }
                        """).run().writeAll();
        Files.delete(libClasses.resolve("lib").resolve("Bar.class"));
        new JavacTask(tb).classpath(libClasses).sources("""
                import lib.M;
                class T implements M {
                }
                """).run(Task.Expect.FAIL);
    }
}
