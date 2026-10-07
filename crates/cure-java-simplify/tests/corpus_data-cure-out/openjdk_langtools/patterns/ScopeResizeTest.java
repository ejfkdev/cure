import combo.ComboInstance;
import combo.ComboParameter;
import combo.ComboTask;
import combo.ComboTestHelper;
import java.util.stream.Stream;
import toolbox.ToolBox;

public class ScopeResizeTest extends ComboInstance<ScopeResizeTest> {
    protected ToolBox tb;
    ScopeResizeTest() {
        super();
        tb = new ToolBox();
    }
    public static void main(String... args) throws Exception {
        PredefinedVariables[] variants = Stream.iterate(0, (i) -> i + 1).limit(17).map((s) -> new PredefinedVariables(s)).toArray((s) -> new PredefinedVariables[s]);
        new ComboTestHelper<ScopeResizeTest>().withDimension("PREDEFINED_VARIABLES", (x, predefinedVariables) -> x.predefinedVariables = predefinedVariables, variants).run(ScopeResizeTest::new);
    }
    private PredefinedVariables predefinedVariables;
    private static final String MAIN_TEMPLATE = """
            public class Test {
                public static void test(Object o) {
                    #{PREDEFINED_VARIABLES}
                    switch (o) {
                        case String s when s.isEmpty() -> {}
                        default -> {}
                    }
                }
            }
            """;
    @Override
    protected void doWork() throws Throwable {
        newCompilationTask().withSourceFromTemplate(MAIN_TEMPLATE, (pname) -> switch (pname) {
            case "PREDEFINED_VARIABLES" -> predefinedVariables;
            default -> throw new UnsupportedOperationException(pname);
        }).analyze((result) -> {});
    }
    public record PredefinedVariables(int size) implements ComboParameter {
        @Override
        public String expand(String optParameter) {
            StringBuilder variables = new StringBuilder();
            for (int i = 0; i < size(); i++) {
                variables.append("int i" + i + ";\n");
            }
            return variables.toString();
        }
    }
}
