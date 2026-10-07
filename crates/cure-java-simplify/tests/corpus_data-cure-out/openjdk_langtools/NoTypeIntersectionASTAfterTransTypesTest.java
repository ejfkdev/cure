import java.util.List;
import com.sun.source.util.JavacTask;
import com.sun.tools.javac.api.JavacTool;
import com.sun.tools.javac.comp.TransTypes;
import com.sun.tools.javac.tree.JCTree;
import com.sun.tools.javac.tree.JCTree.JCTypeIntersection;
import com.sun.tools.javac.util.Context;
import com.sun.tools.javac.util.Context.Factory;
import toolbox.ToolBox;

public class NoTypeIntersectionASTAfterTransTypesTest {
    public static void main(String... args) {
        new NoTypeIntersectionASTAfterTransTypesTest().run();
    }
    void run() {
        Context ctx = new Context();
        MyTransTypes.preRegister(ctx);
        JavacTask task = JavacTool.create().getTask(null, null, null, null, null, List.of(new ToolBox.JavaSource("""
                class Test {
                    interface I {}
                    void test() {
                        Runnable r1 = (Runnable & I)() -> {};
                        Runnable r2 = (I & Runnable)() -> {};
                    }
                }
                """)), ctx);
        if (!task.call()) {
            throw new AssertionError("test failed due to a compilation error");
        }
    }
    static class MyTransTypes extends TransTypes {
        public static void preRegister(Context ctx) {
            ctx.put(transTypesKey, new Factory<TransTypes>() {
                @Override
                public TransTypes make(Context c) {
                    return new MyTransTypes(c);
                }
            });
        }
        public MyTransTypes(Context context) {
            super(context);
        }
        @Override
        public void visitTypeIntersection(JCTypeIntersection tree) {
            super.visitTypeIntersection(tree);
            if (result instanceof JCTypeIntersection) {
                throw new AssertionError("there are unexpected type intersection ASTs");
            }
        }
    }
}
