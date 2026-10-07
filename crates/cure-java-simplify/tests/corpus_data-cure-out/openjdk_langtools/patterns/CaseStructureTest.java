import combo.ComboInstance;
import combo.ComboParameter;
import combo.ComboTask;
import combo.ComboTestHelper;
import java.util.Arrays;
import java.util.stream.Collectors;
import toolbox.ToolBox;

public class CaseStructureTest extends ComboInstance<CaseStructureTest> {
    private static final String JAVA_VERSION = System.getProperty("java.specification.version");
    protected ToolBox tb;
    CaseStructureTest() {
        super();
        tb = new ToolBox();
    }
    public static void main(String... args) throws Exception {
        new ComboTestHelper<CaseStructureTest>().withDimension("AS_CASE_LABEL_ELEMENTS", (x, asCaseLabelElements) -> x.asCaseLabelElements = asCaseLabelElements, true, false).withArrayDimension("CASE_LABELS", (x, caseLabels, idx) -> x.caseLabels[idx] = caseLabels, DIMENSIONS, CaseLabel.values()).withFilter((t) -> Arrays.stream(t.caseLabels).anyMatch((l) -> l != CaseLabel.NONE)).withFailMode(ComboTestHelper.FailMode.FAIL_FAST).run(CaseStructureTest::new);
    }
    private static final int DIMENSIONS = 4;
    private boolean asCaseLabelElements;
    private CaseLabel[] caseLabels = new CaseLabel[DIMENSIONS];
    private static final String MAIN_TEMPLATE = """
            public class Test {
                public static void doTest(Integer in) {
                    switch (in) {
                        case -1: break;
                        #{CASES}
                        #{DEFAULT}
                    }
                }
            }
            """;
    @Override
    protected void doWork() throws Throwable {
        String labels = Arrays.stream(caseLabels).filter((l) -> l != CaseLabel.NONE).map((l) -> l.code).collect(Collectors.joining(asCaseLabelElements ? ", " : ": case ", "case ", ": break;"));
        boolean hasDefault = Arrays.stream(caseLabels).anyMatch((l) -> l == CaseLabel.DEFAULT || l == CaseLabel.TYPE_PATTERN);
        newCompilationTask().withSourceFromTemplate(MAIN_TEMPLATE.replace("#{CASES}", labels).replace("#{DEFAULT}", hasDefault ? "" : "default: break;")).generate((result) -> {
            boolean shouldPass = true;
            long patternCases = Arrays.stream(caseLabels).filter((l) -> l == CaseLabel.TYPE_PATTERN).count();
            long constantCases = Arrays.stream(caseLabels).filter((l) -> l == CaseLabel.CONSTANT).count();
            long nullCases = Arrays.stream(caseLabels).filter((l) -> l == CaseLabel.NULL).count();
            long defaultCases = Arrays.stream(caseLabels).filter((l) -> l == CaseLabel.DEFAULT).count();
            if (constantCases > 1) {
                shouldPass &= false;
            }
            if (constantCases > 0) {
                shouldPass &= patternCases == 0;
            }
            if (defaultCases > 0) {
                shouldPass &= asCaseLabelElements && nullCases > 0;
            }
            if (defaultCases > 1) {
                shouldPass &= false;
            }
            if (nullCases > 1) {
                shouldPass &= false;
            }
            if (nullCases > 0) {
                shouldPass &= patternCases == 0 && (constantCases == 0 || !asCaseLabelElements);
                if (defaultCases > 0 && asCaseLabelElements) {
                    shouldPass &= Arrays.asList(caseLabels).indexOf(CaseLabel.NULL) < Arrays.asList(caseLabels).indexOf(CaseLabel.DEFAULT);
                }
            }
            if (patternCases > 1) {
                shouldPass &= false;
            }
            if (patternCases > 0 && defaultCases > 0) {
                shouldPass &= false;
            }
            if (!(shouldPass ^ result.hasErrors())) {
                throw new AssertionError("Unexpected result: shouldPass=" + shouldPass + ", actual: " + !result.hasErrors() + ", info: " + result.compilationInfo());
            }
        });
    }
    public enum CaseLabel implements ComboParameter {
        NONE(""), TYPE_PATTERN("Integer i"), CONSTANT("1"), NULL("null"), DEFAULT("default");
        private final String code;
        private CaseLabel(String code) {
            this.code = code;
        }
        @Override
        public String expand(String optParameter) {
            throw new UnsupportedOperationException("Not supported.");
        }
    }
}
