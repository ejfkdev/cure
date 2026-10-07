import combo.ComboInstance;
import combo.ComboParameter;
import combo.ComboTask;
import combo.ComboTestHelper;
import java.io.InputStream;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Iterator;
import java.util.Objects;
import java.util.function.BiPredicate;
import toolbox.ToolBox;
import javax.tools.JavaFileObject;

public class ConditionalExpressionResolvePending extends ComboInstance<ConditionalExpressionResolvePending> {
    protected ToolBox tb;
    ConditionalExpressionResolvePending() {
        super();
        tb = new ToolBox();
    }
    public static void main(String... args) throws Exception {
        new ComboTestHelper<ConditionalExpressionResolvePending>().withDimension("METHOD", (x, method) -> x.method = method, Method.values()).withDimension("EXPRESSION", (x, expression) -> x.expression = expression, Expression.values()).withDimension("TRUE", (x, True) -> x.True = True, TestOrDummy.values()).withDimension("FALSE", (x, False) -> x.False = False, TestOrDummy.values()).withDimension("SNIPPET", (x, snippet) -> x.snippet = snippet, Snippet.values()).run(ConditionalExpressionResolvePending::new);
    }
    private Method method;
    private Expression expression;
    private TestOrDummy True;
    private TestOrDummy False;
    private Snippet snippet;
    private static final String MAIN_TEMPLATE = """
            public class Test {
                public static boolean doTest(boolean c, Object input) {
                    String clazzName = input.getClass().getName();
                    int len = clazzName.length();
                    #{METHOD}
                }
            }
            """;
    @Override
    protected void doWork() throws Throwable {
        Path base = Paths.get(".");
        ComboTask task = newCompilationTask().withSourceFromTemplate(MAIN_TEMPLATE, (pname) -> switch (pname) {
            case "METHOD" -> method;
            case "EXPRESSION" -> expression;
            case "TRUE" -> True;
            case "FALSE" -> False;
            case "SNIPPET" -> snippet;
            default -> throw new UnsupportedOperationException(pname);
        });
        task.generate((result) -> {
            try {
                Iterator<? extends JavaFileObject> filesIt = result.get().iterator();
                JavaFileObject file = filesIt.next();
                if (filesIt.hasNext()) {
                    throw new IllegalStateException("More than one classfile returned!");
                }
                byte[] data;
                try (InputStream input = file.openInputStream()) {
                    data = input.readAllBytes();
                }
                ClassLoader inMemoryLoader = new ClassLoader() {
                    protected Class<?> findClass(String name) throws ClassNotFoundException {
                        if ("Test".equals(name)) {
                            return defineClass(name, data, 0, data.length);
                        }
                        return super.findClass(name);
                    }
                };
                runTest((c, input) -> {
                    try {
                        return (boolean) Class.forName("Test", false, inMemoryLoader).getDeclaredMethod("doTest", boolean.class, Object.class).invoke(null, c, input);
                    } catch (Exception ex) {
                        throw new IllegalStateException(ex);
                    }
                });
            } catch (Throwable ex) {
                throw new IllegalStateException(ex);
            }
        });
    }
    private void runTest(BiPredicate<Boolean, Object> test) {
        assertEquals(false, test.test(true, ""));
        assertEquals(true, test.test(true, 1));
        assertEquals(false, test.test(false, ""));
        assertEquals(true, test.test(false, 1));
    }
    private void assertEquals(Object o1, Object o2) {
        if (!Objects.equals(o1, o2)) {
            throw new AssertionError();
        }
    }
    public enum Method implements ComboParameter {
        VARIABLE("""
                 boolean b = #{EXPRESSION};
                 return b;
                 """), IF("""
           boolean b;
           if (#{EXPRESSION}) b = true;
           else b = false;
           return b;
           """), RETURN("""
               return #{EXPRESSION};
               """);
        private final String body;
        private Method(String body) {
            this.body = body;
        }
        @Override
        public String expand(String optParameter) {
            return body;
        }
    }
    public enum Expression implements ComboParameter {
        CONDITIONAL("c ? #{TRUE} : #{FALSE}"), AND("(c && #{TRUE}) || (!c && #{FALSE})");
        private final String expression;
        private Expression(String expression) {
            this.expression = expression;
        }
        @Override
        public String expand(String optParameter) {
            return expression;
        }
    }
    public enum TestOrDummy implements ComboParameter {
        TEST("!(#{SNIPPET})"), DUMMY("input.getClass() == Integer.class");
        private final String code;
        private TestOrDummy(String code) {
            this.code = code;
        }
        @Override
        public String expand(String optParameter) {
            return code;
        }
    }
    public enum Snippet implements ComboParameter {
        PATTERN("input instanceof String sX"), SWITCH_EXPRESSION("switch (len) { case 16 -> {boolean r = true; yield r; } default -> {boolean r = false; yield r; } }"), SWITCH_EXPRESSION_STRING("switch (clazzName) { case \"java.lang.String\"-> {boolean r = true; yield r; } default -> {boolean r = false; yield r; } }");
        private static int idx;
        private final String snippet;
        private Snippet(String snippet) {
            this.snippet = snippet;
        }
        @Override
        public String expand(String optParameter) {
            return snippet.replace("sX", "s" + idx++);
        }
    }
}
