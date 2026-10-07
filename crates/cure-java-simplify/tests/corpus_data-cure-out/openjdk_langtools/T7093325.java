import java.io.IOException;
import java.io.InputStream;
import java.lang.classfile.*;
import java.lang.classfile.attribute.CodeAttribute;
import java.lang.classfile.constantpool.ClassEntry;
import javax.tools.JavaFileObject;
import combo.ComboInstance;
import combo.ComboParameter;
import combo.ComboTask.Result;
import combo.ComboTestHelper;

public class T7093325 extends ComboInstance<T7093325> {
    enum StatementKind implements ComboParameter {
        NONE(null, false, false), THROW("throw new RuntimeException();", false, false), RETURN_NONEMPTY("System.out.println(); return;", true, false), RETURN_EMPTY("return;", true, true), APPLY("System.out.println();", true, false);
        String stmt;
        boolean canInline;
        boolean empty;
        StatementKind(String stmt, boolean canInline, boolean empty) {
            this.stmt = stmt;
            this.canInline = canInline;
            this.empty = empty;
        }
        @Override
        public String expand(String optParameter) {
            return stmt;
        }
    }
    enum CatchArity implements ComboParameter {
        NONE(""), ONE("catch (A a) { #{STMT[1]} }"), TWO("catch (B b) { #{STMT[2]} }"), THREE("catch (C c) { #{STMT[3]} }"), FOUR("catch (D d) { #{STMT[4]} }");
        String catchStr;
        CatchArity(String catchStr) {
            this.catchStr = catchStr;
        }
        @Override
        public String expand(String optParameter) {
            return this.ordinal() == 0 ? catchStr : CatchArity.values()[this.ordinal() - 1].expand(optParameter) + catchStr;
        }
    }
    public static void main(String... args) throws Exception {
        new ComboTestHelper<T7093325>().withFilter(T7093325::testFilter).withDimension("CATCH", (x, ca) -> x.ca = ca, CatchArity.values()).withArrayDimension("STMT", (x, stmt, idx) -> x.stmts[idx] = stmt, 5, StatementKind.values()).run(T7093325::new);
    }
    CatchArity ca;
    StatementKind[] stmts = new StatementKind[5];
    boolean testFilter() {
        int lastPos = ca.ordinal() + 1;
        for (int i = 0; i < stmts.length; i++) {
            boolean isSet = stmts[i] != StatementKind.NONE;
            if (i < lastPos != isSet) {
                return false;
            }
        }
        return true;
    }
    @Override
    public void doWork() throws IOException {
        newCompilationTask().withSourceFromTemplate(source_template).generate(this::verifyBytecode);
    }
    void verifyBytecode(Result<Iterable<? extends JavaFileObject>> result) {
        boolean lastInlined = false;
        boolean hasCode = false;
        int gapsCount = 0;
        for (int i = 0; i < ca.ordinal() + 1; i++) {
            lastInlined = stmts[i].canInline;
            hasCode = hasCode || !stmts[i].empty;
            if (lastInlined && hasCode) {
                hasCode = false;
                gapsCount++;
            }
        }
        if (!lastInlined) {
            gapsCount++;
        }
        try (InputStream is = result.get().iterator().next().openInputStream()) {
            ClassModel cf = ClassFile.of().parse(is.readAllBytes());
            if (cf == null) {
                fail("ClassFile not found: " + result.compilationInfo());
                return;
            }
            MethodModel test_method = null;
            for (MethodModel m : cf.methods()) {
                if (m.methodName().equalsString("test")) {
                    test_method = m;
                    break;
                }
            }
            if (test_method == null) {
                fail("Method test() not found in class Test" + result.compilationInfo());
                return;
            }
            CodeAttribute code = test_method.findAttribute(Attributes.code()).orElse(null);
            if (code == null) {
                fail("Code attribute not found in method test()");
                return;
            }
            int actualGapsCount = 0;
            for (int i = 0; i < code.exceptionHandlers().size(); i++) {
                ClassEntry catchType = code.exceptionHandlers().get(i).catchType().orElse(null);
                if (catchType == null) {
                    actualGapsCount++;
                }
            }
            if (actualGapsCount != gapsCount) {
                fail("Bad exception table for test()\nexpected gaps: " + gapsCount + "\nfound gaps: " + actualGapsCount + "\n" + result.compilationInfo());
            }
        } catch (IOException e) {
            e.printStackTrace();
            fail("error reading classfile: " + e);
        }
    }
    static final String source_template = "class Test {\n   void test() {\n   try { #{STMT[0]} } #{CATCH} finally { System.out.println(); }\n   }\n}\nclass A extends RuntimeException {} \nclass B extends RuntimeException {} \nclass C extends RuntimeException {} \nclass D extends RuntimeException {} \n" + "class E extends RuntimeException {}";
}
