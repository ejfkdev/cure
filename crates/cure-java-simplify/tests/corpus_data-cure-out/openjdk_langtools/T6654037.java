import com.sun.source.tree.BinaryTree;
import com.sun.source.tree.ClassTree;
import com.sun.source.tree.CompilationUnitTree;
import com.sun.source.tree.MethodTree;
import com.sun.source.tree.VariableTree;
import com.sun.tools.javac.api.JavacTaskImpl;
import com.sun.tools.javac.tree.JCTree;
import java.net.URI;
import java.util.Arrays;
import javax.tools.JavaCompiler;
import javax.tools.JavaFileObject;
import javax.tools.SimpleJavaFileObject;
import javax.tools.ToolProvider;

public class T6654037 {
    public static void main(String[] args) throws Exception {
        JavaCompiler tool = ToolProvider.getSystemJavaCompiler();
        assert tool != null;
        JCTree condJC = (JCTree) (BinaryTree) ((VariableTree) ((MethodTree) ((ClassTree) ((JavacTaskImpl) tool.getTask(null, null, null, Arrays.asList("-Xjcov"), null, Arrays.asList(new MyFileObject("package test; public class Test {private void test() {Object o = null; boolean b = o != null && o instanceof String;} private Test() {}}")))).parse().iterator().next().getTypeDecls().get(0)).getMembers().get(0)).getBody().getStatements().get(1)).getInitializer();
        if (condJC.pos != 93) 
            throw new IllegalStateException("Unexpected position=" + condJC.pos);
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
