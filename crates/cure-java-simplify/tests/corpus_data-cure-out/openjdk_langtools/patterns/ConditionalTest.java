import combo.ComboInstance;
import combo.ComboParameter;
import combo.ComboTask;
import combo.ComboTestHelper;
import java.nio.file.Path;
import java.nio.file.Paths;
import toolbox.ToolBox;

public class ConditionalTest extends ComboInstance<ConditionalTest> {
    protected ToolBox tb;
    ConditionalTest() {
        super();
        tb = new ToolBox();
    }
    public static void main(String... args) throws Exception {
        new ComboTestHelper<ConditionalTest>().withDimension("COND", (x, cond) -> x.cond = cond, Pattern.values()).withDimension("TRUE", (x, trueSec) -> x.trueSec = trueSec, Pattern.values()).withDimension("FALSE", (x, falseSec) -> x.falseSec = falseSec, Pattern.values()).run(ConditionalTest::new);
    }
    private Pattern cond;
    private Pattern trueSec;
    private Pattern falseSec;
    private static final String MAIN_TEMPLATE = """
            public class Test {
                public static boolean doTest(Object o, boolean b) {
                    return #{COND} ? #{TRUE} : #{FALSE}
                }
            }
            """;
    @Override
    protected void doWork() throws Throwable {
        Path base = Paths.get(".");
        newCompilationTask().withSourceFromTemplate(MAIN_TEMPLATE, (pname) -> switch (pname) {
            case "COND" -> cond;
            case "TRUE" -> trueSec;
            case "FALSE" -> falseSec;
            default -> throw new UnsupportedOperationException(pname);
        }).analyze((result) -> {
            boolean shouldPass = cond == Pattern.TRUE && (trueSec == Pattern.TRUE || trueSec == Pattern.FALSE) ? false : cond == Pattern.FALSE && (falseSec == Pattern.TRUE || falseSec == Pattern.FALSE) ? false : cond == Pattern.TRUE && falseSec == Pattern.TRUE ? false : cond == Pattern.FALSE && trueSec == Pattern.TRUE ? false : trueSec == Pattern.TRUE && falseSec == Pattern.TRUE ? false : trueSec == Pattern.TRUE && falseSec == Pattern.TRUE ? false : cond == Pattern.TRUE && falseSec == Pattern.FALSE ? false : cond == Pattern.FALSE && trueSec == Pattern.FALSE ? false : !(trueSec == Pattern.FALSE && falseSec == Pattern.FALSE);
            if (!shouldPass) {
                result.containsKey("Blabla");
            }
        });
    }
    public enum Pattern implements ComboParameter {
        NONE("b"), TRUE("o instanceof String s"), FALSE("!(o instanceof String s)");
        private final String code;
        private Pattern(String code) {
            this.code = code;
        }
        @Override
        public String expand(String optParameter) {
            return code;
        }
    }
}
