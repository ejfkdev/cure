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

public class TypeAnnotationsPositionsOnRecords {
    final String src = """
            import java.lang.annotation.*;

            @Retention(RetentionPolicy.RUNTIME)
            @Target({ ElementType.TYPE_USE })
            @interface Nullable {}

            record Record1(@Nullable String t) {}

            record Record2(@Nullable String t) {
                public Record2 {}
            }

            record Record3(@Nullable String t1, @Nullable String t2) {}

            record Record4(@Nullable String t1, @Nullable String t2) {
                public Record4 {}
            }

            record Record5(String t1, @Nullable String t2) {}

            record Record6(String t1, @Nullable String t2) {
                public Record6 {}
            }

            class Test2 {
                @Target(ElementType.TYPE_USE)
                @Retention(RetentionPolicy.RUNTIME)
                public @interface Anno {}

                class Foo {}
                record Record7(Test2.@Anno Foo foo) {
                    public Record7 {} // compact constructor
                }
            }
            """;
    public static void main(String... args) throws Exception {
        new TypeAnnotationsPositionsOnRecords().run();
    }
    ToolBox tb = new ToolBox();
    void run() throws Exception {
        compileTestClass();
        checkClassFile(new File(Paths.get(System.getProperty("user.dir"), "Record1.class").toUri()), 0);
        checkClassFile(new File(Paths.get(System.getProperty("user.dir"), "Record2.class").toUri()), 0);
        checkClassFile(new File(Paths.get(System.getProperty("user.dir"), "Record3.class").toUri()), 0, 1);
        checkClassFile(new File(Paths.get(System.getProperty("user.dir"), "Record4.class").toUri()), 0, 1);
        checkClassFile(new File(Paths.get(System.getProperty("user.dir"), "Record5.class").toUri()), 1);
        checkClassFile(new File(Paths.get(System.getProperty("user.dir"), "Record6.class").toUri()), 1);
        checkClassFile(new File(Paths.get(System.getProperty("user.dir"), "Test2$Record7.class").toUri()), 0);
    }
    void compileTestClass() throws Exception {
        new JavacTask(tb).sources(src).run();
    }
    void checkClassFile(final File cfile, int... taPositions) throws Exception {
        ClassModel classFile = ClassFile.of().parse(cfile.toPath());
        System.err.println("-----------loading " + cfile.getPath());
        int accessorPos = 0;
        int checkedAccessors = 0;
        for (MethodModel method : classFile.methods()) {
            String methodName = method.methodName().stringValue();
            if (methodName.equals("toString") || methodName.equals("hashCode") || methodName.equals("equals")) {
                continue;
            }
            if (methodName.equals("<init>")) {
                checkConstructor(classFile, method, taPositions);
            } else {
                for (int taPos : taPositions) {
                    if (taPos == accessorPos) {
                        checkAccessor(classFile, method);
                        checkedAccessors++;
                    }
                }
                accessorPos++;
            }
        }
        checkFields(classFile, taPositions);
        Assert.check(checkedAccessors == taPositions.length);
    }
    void checkConstructor(ClassModel classFile, MethodModel method, int... positions) throws Exception {
        List<TypeAnnotation> annos = new ArrayList<>();
        findAnnotations(classFile, method, annos);
        Assert.check(annos.size() == positions.length);
        int i = 0;
        for (int pos : positions) {
            TypeAnnotation ta = annos.get(i);
            Assert.check(ta.targetInfo().targetType().name().equals("METHOD_FORMAL_PARAMETER"));
            assert ta.targetInfo() instanceof TypeAnnotation.FormalParameterTarget;
            Assert.check(((TypeAnnotation.FormalParameterTarget) ta.targetInfo()).formalParameterIndex() == pos);
            i++;
        }
    }
    void checkAccessor(ClassModel classFile, MethodModel method) {
        List<TypeAnnotation> annos = new ArrayList<>();
        findAnnotations(classFile, method, annos);
        Assert.check(annos.size() == 1);
        TypeAnnotation ta = annos.get(0);
        Assert.check(ta.targetInfo().targetType().name().equals("METHOD_RETURN"));
    }
    void checkFields(ClassModel classFile, int... positions) {
        if (positions != null && positions.length > 0) {
            int fieldPos = 0;
            int annotationPos = 0;
            int currentAnnoPosition = positions[annotationPos];
            int annotatedFields = 0;
            for (FieldModel field : classFile.fields()) {
                List<TypeAnnotation> annos = new ArrayList<>();
                findAnnotations(classFile, field, annos);
                if (fieldPos != currentAnnoPosition) {
                    Assert.check(annos.size() == 0);
                } else {
                    Assert.check(annos.size() == 1);
                    TypeAnnotation ta = annos.get(0);
                    Assert.check(ta.targetInfo().targetType().name().equals("FIELD"));
                    annotationPos++;
                    currentAnnoPosition = annotationPos < positions.length ? positions[annotationPos] : -1;
                    annotatedFields++;
                }
                fieldPos++;
            }
            Assert.check(annotatedFields == positions.length);
        }
    }
    void findAnnotations(ClassModel cm, AttributedElement m, List<TypeAnnotation> annos) {
        findAnnotations(cm, m, Attributes.runtimeVisibleTypeAnnotations(), annos);
        findAnnotations(cm, m, Attributes.runtimeInvisibleTypeAnnotations(), annos);
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
