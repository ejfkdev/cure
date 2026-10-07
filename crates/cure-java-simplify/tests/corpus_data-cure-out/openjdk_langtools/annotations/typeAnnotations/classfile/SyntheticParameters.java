import annotations.classfile.ClassfileInspector;
import java.lang.classfile.ClassModel;
import java.lang.classfile.TypeAnnotation;
import java.io.*;
import java.lang.annotation.*;

public class SyntheticParameters extends ClassfileInspector {
    private static final String Inner_class = "SyntheticParameters$Inner.class";
    private static final String Foo_class = "SyntheticParameters$Foo.class";
    private static final Expected Inner_expected = new Expected("SyntheticParameters$Inner", null, new ExpectedMethodTypeAnnotation[] {(ExpectedMethodTypeAnnotation) new ExpectedMethodTypeAnnotation.Builder("<init>", "A", TypeAnnotation.TargetType.METHOD_FORMAL_PARAMETER, false, 1).setParameterIndex(0).build()}, null);
    private static final Expected Foo_expected = new Expected("SyntheticParameters$Foo", null, new ExpectedMethodTypeAnnotation[] {(ExpectedMethodTypeAnnotation) new ExpectedMethodTypeAnnotation.Builder("<init>", "A", TypeAnnotation.TargetType.METHOD_FORMAL_PARAMETER, false, 1).setParameterIndex(0).build()}, null);
    public static void main(String... args) throws Exception {
        new SyntheticParameters().run(new ClassModel[] {getClassFile(Inner_class, Inner.class), getClassFile(Foo_class, Foo.class)}, new Expected[] {Inner_expected, Foo_expected});
    }
    public class Inner {
        public Inner(@A int a) {}
        public void foo(@A int a, int b) {}
    }
    public static enum Foo {
        ONE(null);
        Foo(@A Object a) {}
    }
}

@Target({ElementType.TYPE_USE}) @interface A {
}
