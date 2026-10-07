package com.puppycrawl.tools.checkstyle.checks.javadoc.missingjavadoctype;

public class InputMissingJavadocTypeQualifiedAnnotationWithParameters {
    public @interface SomeAnnotation {
        String value() default "";
        int quantity() default 1;
        boolean isOk() default false;
    }
    @SomeAnnotation("value")
    public interface A {
    }
    @SomeAnnotation(value = "value", quantity = 5, isOk = true)
    public interface B {
    }
    @SomeAnnotation(
        value = "value",
        isOk = true
    )
    public interface C {
    }
    @InputMissingJavadocTypeQualifiedAnnotationWithParameters.SomeAnnotation("value")
    public interface D {
    }
    @InputMissingJavadocTypeQualifiedAnnotationWithParameters
    .SomeAnnotation(value = "value", isOk = false)
    public interface E {
    }
    @InputMissingJavadocTypeQualifiedAnnotationWithParameters.SomeAnnotation(
        value = "value",
        quantity = 2,
        isOk = false
    )
    public interface F {
    }
    @InputMissingJavadocTypeQualifiedAnnotationWithParameters
    .SomeAnnotation(
        value = "value",
        isOk = false
    )
    public interface G {
    }
}
