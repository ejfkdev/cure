import com.sun.source.util.TaskEvent;
import com.sun.source.util.TaskListener;
import com.sun.tools.javac.code.Symbol;
import com.sun.tools.javac.tree.JCTree.JCCompilationUnit;
import com.sun.tools.javac.tree.JCTree.JCFieldAccess;
import com.sun.tools.javac.tree.TreeScanner;
import combo.ComboInstance;
import combo.ComboParameter;
import combo.ComboTestHelper;

public class ClassFieldDeduplication extends ComboInstance<ClassFieldDeduplication> {
    enum Type implements ComboParameter {
        OBJECT("Object"), PRIMITIVE("int"), BOXED_PRIMITIVE("Integer"), VOID("void"), BOXED_VOID("Void"), OBJECT_ARRAY("Object[]"), PRIMITIVE_ARRAY("int[]"), BOXED_PRIMITIVE_ARRAY("Integer[]"), BOXED_VOID_ARRAY("Void[]");
        String type;
        Type(String type) {
            this.type = type;
        }
        @Override
        public String expand(String optParameter) {
            return type;
        }
    }
    public static void main(String... args) throws Exception {
        new ComboTestHelper<ClassFieldDeduplication>().withDimension("TYPE", Type.values()).run(ClassFieldDeduplication::new);
    }
    private static final String TEMPLATE = "class Test { void t() { Object o1 = #{TYPE}.class; Object o2 = #{TYPE}.class; } }";
    @Override
    protected void doWork() throws Throwable {
        newCompilationTask().withSourceFromTemplate(TEMPLATE).withListener(new TaskListener() {
                    JCCompilationUnit cut;
                        @Override
                        public void finished(TaskEvent e) {
                            if (e.getKind() == TaskEvent.Kind.PARSE) {
                                if (cut != null)
                                    throw new AssertionError();
                                cut = (JCCompilationUnit) e.getCompilationUnit();
                            }
                            if (e.getKind() == TaskEvent.Kind.ANALYZE) {
                                cut.accept(new TreeScanner() {
                                    Symbol s;
                                    @Override
                                    public void visitSelect(JCFieldAccess tree) {
                                        if (tree.name.contentEquals("class")) {
                                            if (s == null) {
                                                s = tree.sym;
                                            } else if (s != tree.sym) {
                                                throw new AssertionError("Duplicated field symbol.");
                                            }
                                        }
                                        super.visitSelect(tree);
                                    }
                                });
                            }
                        }

                }).analyze((els) -> {});
    }
}
