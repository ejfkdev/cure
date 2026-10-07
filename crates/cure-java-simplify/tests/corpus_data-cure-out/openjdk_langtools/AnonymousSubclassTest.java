import toolbox.JavacTask;
import toolbox.Task;
import toolbox.ToolBox;

public class AnonymousSubclassTest {
    public static void main(String... args) throws Exception {
        new AnonymousSubclassTest().run();
    }
    ToolBox tb = new ToolBox();
    final String foo = "public class Foo {  void m() { Foo f = new Foo() {}; }" + "}";
    final String test1 = "public class Test1 {  void m() {    Foo f1 = new Foo();    Foo f2 = new Foo$1(f1) {};  }" + "}";
    final String test2 = "public class Test2 {  class T extends Foo$1 {    public T(Foo f) { super(f); }  }" + "}";
    void compOk(String code) throws Exception {
        new JavacTask(tb).sources(code).run();
    }
    void compFail(String code) throws Exception {
        String errs = new JavacTask(tb).sources(code).classpath(".").options("-XDrawDiagnostics").run(Task.Expect.FAIL).writeAll().getOutput(Task.OutputKind.DIRECT);
        if (!errs.contains("cant.inherit.from.anon")) {
            throw new Exception("test failed");
        }
    }
    void run() throws Exception {
        compOk(foo);
        compFail(test1);
        compFail(test2);
    }
}
