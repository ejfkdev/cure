import java.util.List;
import java.util.ArrayList;
import java.io.File;
import java.nio.file.Paths;
import java.lang.annotation.*;
import java.util.Arrays;
import java.lang.classfile.*;
import java.lang.classfile.attribute.*;
import com.sun.tools.javac.util.Assert;
import toolbox.JavacTask;
import toolbox.ToolBox;

public class VariablesDeclaredWithVarTest {
    ToolBox tb = new ToolBox();
    final String src = """
            import java.util.function.*;
            import java.lang.annotation.ElementType;
            import java.lang.annotation.Target;

            @Target({ElementType.TYPE_USE, ElementType.PARAMETER, ElementType.LOCAL_VARIABLE})
            @interface A {}

            class Test {
                void kaa() {
                    @A var c = g(1, 1L);
                }

                <X> X g(X a, X b) {
                    return a;
                }

                void foo() {
                    bar((@A var s) -> s);
                }

                void bar(Function<String, String> f) {}
            }
            """;
    public static void main(String... args) throws Exception {
        new VariablesDeclaredWithVarTest().run();
    }
    void run() throws Exception {
        compileTestClass();
        checkClassFile(new File(Paths.get(System.getProperty("user.dir"), "Test.class").toUri()), 0);
    }
    void compileTestClass() throws Exception {
        new JavacTask(tb).sources(src).run();
    }
    void checkClassFile(final File cfile, int... taPositions) throws Exception {
        ClassModel classFile = ClassFile.of().parse(cfile.toPath());
        List<TypeAnnotation> annos = new ArrayList<>();
        for (MethodModel method : classFile.methods()) {
            findAnnotations(classFile, method, annos);
            String methodName = method.methodName().stringValue();
            Assert.check(annos.size() == 0, "there shouldn't be any type annotations in any method, found " + annos.size() + " type annotations at method " + methodName);
        }
    }
    void findAnnotations(ClassModel cf, MethodModel m, List<TypeAnnotation> annos) {
        findAnnotations(cf, m, Attributes.runtimeVisibleTypeAnnotations(), annos);
        findAnnotations(cf, m, Attributes.runtimeInvisibleTypeAnnotations(), annos);
    }
    <T extends Attribute<T>> void findAnnotations(ClassModel cf, AttributedElement m, AttributeMapper<T> attrName, List<TypeAnnotation> annos) {
        Attribute<T> attr = m.findAttribute(attrName).orElse(null);
        addAnnos(annos, attr);
        if (m instanceof MethodModel) {
            CodeAttribute cattr = m.findAttribute(Attributes.code()).orElse(null);
            if (cattr != null) {
                attr = cattr.findAttribute(attrName).orElse(null);
                addAnnos(annos, attr);
            }
        }
    }
    private <T extends Attribute<T>> void addAnnos(List<TypeAnnotation> annos, Attribute<T> attr) {
        if (attr != null) {
            switch (attr) {
                case RuntimeVisibleTypeAnnotationsAttribute vanno -> {
                    annos.addAll(vanno.annotations());
                }
                case RuntimeInvisibleTypeAnnotationsAttribute ivanno -> {
                    annos.addAll(ivanno.annotations());
                }
                default -> {
                    throw new AssertionError();
                }
            }
        }
    }
}
