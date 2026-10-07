import annotations.classfile.ClassfileInspector;
import java.io.*;
import java.lang.annotation.*;
import java.lang.classfile.ClassModel;

public class SyntheticParameters extends ClassfileInspector {
    private static final String Inner_class = "SyntheticParameters$Inner.class";
    private static final String Foo_class = "SyntheticParameters$Foo.class";
    private static final Expected Inner_expected = new Expected("SyntheticParameters$Inner", null, null, new ExpectedParameterAnnotation[] {(ExpectedParameterAnnotation) new ExpectedParameterAnnotation("<init>", 0, "A", true, 1), (ExpectedParameterAnnotation) new ExpectedParameterAnnotation("foo", 0, "A", true, 1), (ExpectedParameterAnnotation) new ExpectedParameterAnnotation("<init>", 0, "B", false, 1), (ExpectedParameterAnnotation) new ExpectedParameterAnnotation("foo", 0, "B", false, 1), (ExpectedParameterAnnotation) new ExpectedParameterAnnotation("foo", 1, "B", false, 0)}, null);
    private static final Expected Foo_expected = new Expected("SyntheticParameters$Foo", null, null, new ExpectedParameterAnnotation[] {(ExpectedParameterAnnotation) new ExpectedParameterAnnotation("<init>", 0, "A", true, 1), (ExpectedParameterAnnotation) new ExpectedParameterAnnotation("<init>", 0, "B", false, 1)}, null);
    public static void main(String... args) throws Exception {
        new SyntheticParameters().run(new ClassModel[] {getClassFile(Inner_class, Inner.class), getClassFile(Foo_class, Foo.class)}, new Expected[] {Inner_expected, Foo_expected});
    }
    public class Inner {
        public Inner(@A @B int a) {}
        public void foo(@A @B int a, int b) {}
    }
    public static enum Foo {
        ONE(null);
        Foo(@A @B Object a) {}
    }
}

@Retention(RetentionPolicy.RUNTIME) @interface A {
}

@Retention(RetentionPolicy.CLASS) @interface B {
}
