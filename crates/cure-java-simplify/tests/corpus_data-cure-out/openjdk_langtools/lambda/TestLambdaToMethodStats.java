import java.io.IOException;
import javax.tools.Diagnostic;
import javax.tools.JavaFileObject;
import com.sun.tools.javac.api.ClientCodeWrapper;
import com.sun.tools.javac.util.List;
import combo.ComboInstance;
import combo.ComboParameter;
import combo.ComboTask.Result;
import combo.ComboTestHelper;

public class TestLambdaToMethodStats extends ComboInstance<TestLambdaToMethodStats> {
    enum ExprKind implements ComboParameter {
        LAMBDA("()->null"), MREF1("this::g"), MREF2("this::h");
        String exprStr;
        ExprKind(String exprStr) {
            this.exprStr = exprStr;
        }
        @Override
        public String expand(String optParameter) {
            return exprStr;
        }
    }
    enum TargetKind implements ComboParameter {
        IMPLICIT(""), SERIALIZABLE("(A & java.io.Serializable)");
        String targetStr;
        TargetKind(String targetStr) {
            this.targetStr = targetStr;
        }
        @Override
        public String expand(String optParameter) {
            return targetStr;
        }
    }
    enum DiagnosticKind {
        LAMBDA_STAT("compiler.note.lambda.stat", true, false), MREF_STAT("compiler.note.mref.stat", false, false), MREF_STAT1("compiler.note.mref.stat.1", false, true);
        String code;
        boolean lambda;
        boolean bridge;
        DiagnosticKind(String code, boolean lambda, boolean bridge) {
            this.code = code;
            this.lambda = lambda;
            this.bridge = bridge;
        }
    }
    public static void main(String... args) throws Exception {
        new ComboTestHelper<TestLambdaToMethodStats>().withDimension("EXPR", (x, expr) -> x.ek = expr, ExprKind.values()).withDimension("CAST", (x, target) -> x.tk = target, TargetKind.values()).run(TestLambdaToMethodStats::new);
    }
    ExprKind ek;
    TargetKind tk;
    String template = "interface A {\n   Object o();\n}\nclass Test {\n   A a = #{CAST}#{EXPR};\n   Object g() { return null; }\n   Object h(Object... o) { return null; }\n" + "}";
    @Override
    public void doWork() throws IOException {
        newCompilationTask().withOption("--debug=dumpLambdaToMethodStats").withSourceFromTemplate(template).generate(this::check);
    }
    void check(Result<?> res) {
        DiagnosticKind diag = null;
        boolean altMetafactory = false;
        for (DiagnosticKind dk : DiagnosticKind.values()) {
            List<Diagnostic<? extends JavaFileObject>> jcDiag = res.diagnosticsForKey(dk.code);
            if (jcDiag.nonEmpty()) {
                diag = dk;
                altMetafactory = (Boolean) ((ClientCodeWrapper.DiagnosticSourceUnwrapper) jcDiag.head).d.getArgs()[0];
                break;
            }
        }
        if (diag == null) {
            fail("No diagnostic found; " + res.compilationInfo());
        }
        boolean error = diag.lambda != (ek == ExprKind.LAMBDA);
        error |= diag.bridge != (ek == ExprKind.MREF2);
        error |= altMetafactory != (tk == TargetKind.SERIALIZABLE);
        if (error) {
            fail("Bad stat diagnostic found for source\nlambda = " + diag.lambda + "\nbridge = " + diag.bridge + "\naltMF = " + altMetafactory + "\n" + res.compilationInfo());
        }
    }
}
