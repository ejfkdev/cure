import combo.ComboInstance;
import combo.ComboParameter;
import combo.ComboTask;
import combo.ComboTestHelper;
import java.io.StringWriter;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;
import javax.tools.Diagnostic;
import toolbox.ToolBox;

public class T8235564 extends ComboInstance<T8235564> {
    protected ToolBox tb;
    T8235564() {
        super();
        tb = new ToolBox();
    }
    public static void main(String... args) throws Exception {
        new ComboTestHelper<T8235564>().withDimension("INVOCATION", (x, invocation) -> x.invocation = invocation, Invocation.values()).withDimension("PARAM", (x, param) -> x.param = param, Parameter.values()).run(T8235564::new);
    }
    private Invocation invocation;
    private Parameter param;
    private static final String MAIN_TEMPLATE = """
            public class Test {
                static void test() {
                    Runnable r = () -> {};
                    #{INVOCATION};
                }
                private static void existingWithFunctional(Runnable r) {}
                private static void existingWithoutFunctional(String parameter) {}
            }
            """;
    @Override
    protected void doWork() throws Throwable {
        StringWriter out = new StringWriter();
        newCompilationTask().withSourceFromTemplate(MAIN_TEMPLATE, (pname) -> switch (pname) {
            case "INVOCATION" -> invocation;
            case "PARAM" -> param;
            default -> throw new UnsupportedOperationException(pname);
        }).withOption("-XDshould-stop.at=FLOW").withOption("-XDrawDiagnostics").withOption("-XDdev").withWriter(out).analyze((result) -> {
            List<String> diags = result.diagnosticsForKind(Diagnostic.Kind.ERROR).stream().map((d) -> d.getLineNumber() + ":" + d.getCode()).collect(Collectors.toList());
            List<String> expected = new ArrayList<>();
            switch (param) {
                case VALID_VARIABLE, VALID_LAMBDA, VALID_MEMBER_REF -> {}
                case UNDEFINED_VARIABLE -> expected.add("4:compiler.err.cant.resolve.location");
                case UNDEFINED_METHOD, UNDEFINED_LAMBDA -> expected.add("4:compiler.err.cant.resolve.location.args");
                case UNDEFINED_MEMBER_REF -> expected.add("4:compiler.err.invalid.mref");
                case UNDEFINED_CONDEXPR -> {
                    if (invocation != Invocation.EXISTING_WITHOUT_FUNCTIONAL) {
                        expected.add("4:compiler.err.invalid.mref");
                        expected.add("4:compiler.err.invalid.mref");
                    }
                }
            }
            switch (invocation) {
                case EXISTING_WITH_FUNCTIONAL -> {
                    if (param == Parameter.UNDEFINED_CONDEXPR) {
                        expected.add("4:compiler.err.cant.apply.symbol");
                    }
                }
                case EXISTING_WITHOUT_FUNCTIONAL -> {
                    if (param != Parameter.UNDEFINED_VARIABLE && param != Parameter.UNDEFINED_MEMBER_REF && param != Parameter.UNDEFINED_METHOD) {
                        expected.add("4:compiler.err.cant.apply.symbol");
                    }
                }
                case UNDEFINED -> {
                    if (param != Parameter.UNDEFINED_VARIABLE && param != Parameter.UNDEFINED_MEMBER_REF && param != Parameter.UNDEFINED_METHOD) {
                        expected.add("4:compiler.err.cant.resolve.location.args");
                    }
                }
            }
            if (!expected.equals(diags)) {
                throw new AssertionError("Expected errors not found, expected: " + expected + ", actual: " + diags);
            }
            if (out.toString().length() > 0) {
                throw new AssertionError("No output expected, but got:\n" + out + "\n\n" + result.compilationInfo());
            }
        });
    }
    public enum Invocation implements ComboParameter {
        EXISTING_WITH_FUNCTIONAL("existingWithFunctional(#{PARAM})"), EXISTING_WITHOUT_FUNCTIONAL("existingWithoutFunctional(#{PARAM})"), UNDEFINED("undefined(#{PARAM})");
        private final String invocation;
        private Invocation(String invocation) {
            this.invocation = invocation;
        }
        @Override
        public String expand(String optParameter) {
            return invocation;
        }
    }
    public enum Parameter implements ComboParameter {
        VALID_VARIABLE("r"), VALID_LAMBDA("() -> {}"), VALID_MEMBER_REF("Test::test"), UNDEFINED_VARIABLE("undefined"), UNDEFINED_LAMBDA("() -> {undefined();}"), UNDEFINED_MEMBER_REF("Test::undefined"), UNDEFINED_METHOD("undefined()"), UNDEFINED_CONDEXPR("1 == 2 ? Test::undefined : Test::undefined");
        private final String code;
        private Parameter(String code) {
            this.code = code;
        }
        @Override
        public String expand(String optParameter) {
            return code;
        }
    }
}
