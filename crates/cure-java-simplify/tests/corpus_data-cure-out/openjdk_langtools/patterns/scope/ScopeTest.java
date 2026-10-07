import java.io.IOException;
import java.util.Arrays;
import java.util.stream.Collectors;
import org.junit.jupiter.api.Test;
import tools.javac.combo.JavacTemplateTestBase;

class ScopeTest extends JavacTemplateTestBase {
    private static String st_block(String... statements) {
        return Arrays.stream(statements).collect(Collectors.joining("", "{", "}"));
    }
    private static String st_if(String condition, String then, String els) {
        return "if (" + condition + ") " + then + " else " + els;
    }
    private static String st_while(String condition, String body) {
        return "while (" + condition + ") " + body;
    }
    private static String st_do_while(String body, String condition) {
        return "do " + body + " while (" + condition + ");";
    }
    private static String st_for(String init, String condition, String update, String body) {
        return "for (" + init + "; " + condition + "; " + update + ") " + body;
    }
    private static String st_s_use() {
        return "s.length();";
    }
    private static String st_break() {
        return "break;";
    }
    private static String st_return() {
        return "return;";
    }
    private static String st_noop() {
        return ";";
    }
    private static String expr_empty() {
        return "";
    }
    private static String expr_o_match_str() {
        return "o instanceof String s";
    }
    private static String expr_not(String expr) {
        return "!(" + expr + ")";
    }
    private void program(String block) {
        addSourceFile("C.java", "class C { void m(Object o) " + block + "}");
    }
    private void assertOK(String block) {
        reset();
        program(block);
        try {
            compile();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        assertCompileSucceeded();
    }
    private void assertFail(String expectedDiag, String block) {
        reset();
        program(block);
        try {
            compile();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        assertCompileFailed(expectedDiag);
    }
    @Test void testIf() {
        assertOK(st_block(st_if(expr_o_match_str(), st_s_use(), st_return()), st_s_use()));
        assertOK(st_block(st_if(expr_not(expr_o_match_str()), st_return(), st_s_use()), st_s_use()));
        assertFail("compiler.err.cant.resolve.location", st_block(st_if(expr_o_match_str(), st_s_use(), st_noop()), st_s_use()));
        assertFail("compiler.err.cant.resolve.location", st_block(st_if(expr_not(expr_o_match_str()), st_noop(), st_s_use()), st_s_use()));
    }
    @Test void testWhile() {
        assertOK(st_block(st_while(expr_not(expr_o_match_str()), st_noop()), st_s_use()));
        assertFail("compiler.err.cant.resolve.location", st_block(st_while(expr_not(expr_o_match_str()), st_break()), st_s_use()));
    }
    @Test void testDoWhile() {
        assertOK(st_block(st_do_while(st_noop(), expr_not(expr_o_match_str())), st_s_use()));
        assertFail("compiler.err.cant.resolve.location", st_block(st_do_while(st_break(), expr_not(expr_o_match_str())), st_s_use()));
    }
    @Test void testFor() {
        assertOK(st_block(st_for(expr_empty(), expr_not(expr_o_match_str()), expr_empty(), st_noop()), st_s_use()));
        assertFail("compiler.err.cant.resolve.location", st_block(st_for(expr_empty(), expr_not(expr_o_match_str()), expr_empty(), st_break()), st_s_use()));
    }
}
