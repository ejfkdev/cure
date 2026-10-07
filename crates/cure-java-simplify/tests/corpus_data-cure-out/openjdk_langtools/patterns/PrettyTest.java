import java.io.IOException;
import java.io.StringWriter;
import java.net.URI;
import java.util.Arrays;
import java.util.List;
import javax.tools.*;
import com.sun.source.tree.CompilationUnitTree;
import com.sun.source.util.JavacTask;

public class PrettyTest {
    public static void main(String[] args) throws Exception {
        new PrettyTest().run();
    }
    void run() throws Exception {
        String pretty = parse("class Test {\n    boolean t(Object o) {\n         boolean b;\n         boolean _ = true;\n         b = o instanceof String s;\n         b = o instanceof R(String s);\n         b = o instanceof R(var s);\n         b = o instanceof R2(R(var s), String t);\n         b = o instanceof R2(R(var s), var t);\n         b = o instanceof R(String _);\n         b = o instanceof R2(R(var _), var _);\n         b = o instanceof R2(R(_), var t);\n    }\n    record R(String s) {}\n    record R2(R r, String s) {}\n}\n").toString().replaceAll("\\R", "\n");
        if (!"""
                \n\
                class Test {
                    \n\
                    boolean t(Object o) {
                        boolean b;
                        boolean _ = true;
                        b = o instanceof String s;
                        b = o instanceof R(String s);
                        b = o instanceof R(var s);
                        b = o instanceof R2(R(var s), String t);
                        b = o instanceof R2(R(var s), var t);
                        b = o instanceof R(String _);
                        b = o instanceof R2(R(var _), var _);
                        b = o instanceof R2(R(_), var t);
                    }
                    \n\
                    record R(String s) {
                    }
                    \n\
                    record R2(R r, String s) {
                    }
                }""".equals(pretty)) {
            throw new AssertionError("Actual prettified source: " + pretty);
        }
    }
    private CompilationUnitTree parse(String code) throws IOException {
        JavaCompiler tool = ToolProvider.getSystemJavaCompiler();
        assert tool != null;
        DiagnosticListener<JavaFileObject> noErrors = (d) -> {};
        StringWriter out = new StringWriter();
        return ((JavacTask) tool.getTask(out, null, noErrors, List.of(), null, Arrays.asList(new MyFileObject(code)))).parse().iterator().next();
    }
    static class MyFileObject extends SimpleJavaFileObject {
        private String text;
        public MyFileObject(String text) {
            super(URI.create("myfo:/Test.java"), JavaFileObject.Kind.SOURCE);
            this.text = text;
        }
        @Override
        public CharSequence getCharContent(boolean ignoreEncodingErrors) {
            return text;
        }
    }
}
