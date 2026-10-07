package com.puppycrawl.tools.checkstyle.grammar.antlr4;

import java.lang.annotation.ElementType;
import java.lang.annotation.Repeatable;
import java.lang.annotation.Target;

@Target(ElementType.PACKAGE) @interface PackageAnnotations {
    PackageAnnotation[] value();
}

@Repeatable(PackageAnnotations.class)
@Target(ElementType.PACKAGE) @interface PackageAnnotation {
    String value() default  "";
}
