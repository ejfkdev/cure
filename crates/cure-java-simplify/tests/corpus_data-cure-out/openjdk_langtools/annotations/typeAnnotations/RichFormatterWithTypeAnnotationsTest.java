import toolbox.JavacTask;
import toolbox.Task;
import toolbox.TestRunner;
import toolbox.ToolBox;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Arrays;
import java.util.List;

public class RichFormatterWithTypeAnnotationsTest extends TestRunner {
    ToolBox tb;
    public RichFormatterWithTypeAnnotationsTest() {
        super(System.err);
        tb = new ToolBox();
    }
    public static void main(String[] args) throws Exception {
        new RichFormatterWithTypeAnnotationsTest().runTests((m) -> new Object[] {Paths.get(m.getName())});
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
                        @Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER})
                        @Foo(Bar.BAZ)
                        @interface A {}
                        """, """
                        package lib;
                        public interface M<K, V> {
                          @A
                          V f(K k, V v);
                        }
                        """).options().run().writeAll();
        Files.delete(libClasses.resolve("lib").resolve("Bar.class"));
        List<String> output = new JavacTask(tb).classpath(libClasses).sources("""
                import lib.M;
                class T {
                  protected M m;

                  public void f() {
                    m.f(null, 0);
                  }
                }
                """).options("-Xlint:all", "-Werror", "-XDrawDiagnostics").run(Task.Expect.FAIL).writeAll().getOutputLines(Task.OutputKind.DIRECT);
        List<String> expected = Arrays.asList("T.java:3:13: compiler.warn.raw.class.use: lib.M, lib.M<K,V>", "T.java:6:8: compiler.warn.unchecked.call.mbr.of.raw.type: f(K,V), lib.M", "- compiler.err.warnings.and.werror", "1 error", "2 warnings");
        tb.checkEqual(expected, output);
    }
}
