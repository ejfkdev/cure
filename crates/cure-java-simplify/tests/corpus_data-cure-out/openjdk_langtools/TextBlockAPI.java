import toolbox.JavacTask;
import toolbox.JavaTask;
import toolbox.Task;
import toolbox.ToolBox;

public class TextBlockAPI {
    private static ToolBox TOOLBOX = new ToolBox();
    private final static String JDK_VERSION = Integer.toString(Runtime.version().feature());
    public static void main(String... args) {
        test1();
        test2();
        test3();
        test4();
        test5();
        test6();
        test7();
        test8();
    }
    static void test1() {
        for (String lineterminators : new String[] {"\n", "\r", "\r\n"}) 
            for (String whitespace : new String[] {"", "   ", "\t", "\u000C"}) 
                for (String content : new String[] {"a", "ab", "abc", "•", "*".repeat(1000), "*".repeat(10000)}) {
                    compPass("public class CorrectTest {\n    public static void main(String... args) {\n        String xxx = \"\"\"" + whitespace + lineterminators + content + "\"\"\";\n    }\n}\n");
                }
    }
    static void test2() {
        compPass("public class UnicodeDelimiterTest {", "    public static void main(String... args) {", "        String xxx = \\u0022\\u0022\\u0022\nabc\n\\u0022\\u0022\\u0022;", "    }", "}");
    }
    static void test3() {
        compFail("public class EndTest {", "    public static void main(String... args) {", "        String xxx = \"\"\"\nabc\"\"\"");
        compFail("public class TwoQuoteClose {", "    public static void main(String... args) {", "        String xxx = \"\"\"\nabc\"\"");
        compFail("public class OneQuoteClose {", "    public static void main(String... args) {", "        String xxx = \"\"\"\nabc\"");
        compFail("public class NoClose {", "    public static void main(String... args) {", "        String xxx = \"\"\"\nabc");
        compFail("public class ZeroTerminator {", "    public static void main(String... args) {", "        String xxx = \"\"\"\nabc\\u0000");
        compFail("public class NonBreakingSpace {", "    public static void main(String... args) {", "        String xxx = \"\"\"\nabc\\u001A");
    }
    static void test4() {
        String[] terminators = {"\n", "\r\n", "\r"};
        for (String terminator : terminators) {
            new JavacTask(TOOLBOX).sources("public class LineTerminatorTest {" + terminator + "    public static void main(String... args) {" + terminator + "        String s =" + terminator + "\"\"\"" + terminator + "abc" + terminator + "\"\"\";" + terminator + "        System.out.println(s.equals(\"abc\\n\"));" + terminator + "    }" + terminator + "}" + terminator).classpath(".").options("-encoding", "utf8").run();
            String output = new JavaTask(TOOLBOX).classpath(".").classArgs("LineTerminatorTest").run().writeAll().getOutput(Task.OutputKind.STDOUT);
            if (!output.contains("true")) {
                throw new RuntimeException("Error detected");
            }
        }
    }
    static void test5() {
        compPass("public class EscapeSChar {", "    public static void main(String... args) {", "        char xxx = '\\s';", "    }", "}");
        compPass("public class EscapeSString {", "    public static void main(String... args) {", "        String xxx = \"\\s\";", "    }", "}");
        compPass("public class EscapeSTextBlock {", "    public static void main(String... args) {", "        String xxx = \"\"\"", "                     \\s", "                     \"\"\";", "    }", "}");
    }
    static void test6() {
        String[] terminators = {"\n", "\r\n", "\r"};
        for (String terminator : terminators) {
            compPass("public class EscapeLineTerminator {", "    public static void main(String... args) {", "        String xxx = \"\"\"", "                     \\" + terminator + "                     \"\"\";", "    }", "}");
        }
    }
    static void test7() {
        compFail("public class EscapeLineTerminatorChar {", "    public static void main(String... args) {", "        char xxx = '\\\n';", "    }", "}");
        compFail("public class EscapeLineTerminatorString {", "    public static void main(String... args) {", "        String xxx = \"\\\n\";", "    }", "}");
    }
    static void test8() {
        new JavacTask(TOOLBOX).sources("class C {\n\n    void x() {\n        String s = \"\"\"\n\n\"\"\";\n    }\n}\n").classpath(".").options("-encoding", "utf8", "-Xlint").run();
    }
    static void compPass(String source) {
        String output = new JavacTask(TOOLBOX).sources(source).classpath(".").options("-encoding", "utf8").run().writeAll().getOutput(Task.OutputKind.DIRECT);
        if (output.contains("compiler.err")) {
            throw new RuntimeException("Error detected");
        }
    }
    static void compPass(String... lines) {
        compPass(String.join("\n", lines) + "\n");
    }
    static void compFail(String source) {
        String errors = new JavacTask(TOOLBOX).sources(source).classpath(".").options("-XDrawDiagnostics", "-encoding", "utf8").run(Task.Expect.FAIL).writeAll().getOutput(Task.OutputKind.DIRECT);
        if (!errors.contains("compiler.err")) {
            throw new RuntimeException("No error detected");
        }
    }
    static void compFail(String... lines) {
        compFail(String.join("\n", lines) + "\n");
    }
}
