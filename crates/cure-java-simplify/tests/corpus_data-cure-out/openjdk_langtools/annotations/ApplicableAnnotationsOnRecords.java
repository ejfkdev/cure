import java.lang.classfile.*;
import com.sun.tools.javac.util.Assert;
import java.lang.annotation.*;
import java.io.InputStream;
import java.util.Objects;

@Retention(RetentionPolicy.RUNTIME)
@Target({ElementType.FIELD}) @interface FieldAnnotation {
}

@Retention(RetentionPolicy.RUNTIME)
@Target({ElementType.METHOD}) @interface MethodAnnotation {
}

@Retention(RetentionPolicy.RUNTIME)
@Target({ElementType.PARAMETER}) @interface ParameterAnnotation {
}

public record ApplicableAnnotationsOnRecords(@FieldAnnotation @MethodAnnotation @ParameterAnnotation String s, @FieldAnnotation @MethodAnnotation @ParameterAnnotation int i) {
    public static void main(String... args) throws Exception {
        try (InputStream in = ApplicableAnnotationsOnRecords.class.getResourceAsStream("ApplicableAnnotationsOnRecords.class")) {
            ClassModel cm = ClassFile.of().parse(Objects.requireNonNull(in).readAllBytes());
            Assert.check(cm.methods().size() > 5);
            for (MethodModel mm : cm.methods()) {
                String methodName = mm.methodName().stringValue();
                if (!(methodName.equals("toString") || methodName.equals("hashCode") || methodName.equals("equals") || methodName.equals("main"))) 
                    if (methodName.equals("<init>")) {
                        var paAnnos = mm.findAttribute(Attributes.runtimeVisibleParameterAnnotations()).orElseThrow().parameterAnnotations();
                        Assert.check(paAnnos.size() > 0);
                        for (var pa : paAnnos) {
                            Assert.check(pa.size() == 1);
                            Assert.check(Objects.equals(pa.get(0).classSymbol().descriptorString(), "LParameterAnnotation;"));
                        }
                    } else {
                        var annos = mm.findAttribute(Attributes.runtimeVisibleAnnotations()).orElseThrow().annotations();
                        Assert.check(annos.size() == 1);
                        Assert.check(Objects.equals(annos.get(0).classSymbol().descriptorString(), "LMethodAnnotation;"));
                    }
            }
            Assert.check(cm.fields().size() > 0);
            for (FieldModel fm : cm.fields()) {
                var annos = fm.findAttribute(Attributes.runtimeVisibleAnnotations()).orElseThrow().annotations();
                Assert.check(annos.size() == 1);
                Assert.check(Objects.equals(annos.getFirst().classSymbol().descriptorString(), "LFieldAnnotation;"));
            }
        }
    }
}
