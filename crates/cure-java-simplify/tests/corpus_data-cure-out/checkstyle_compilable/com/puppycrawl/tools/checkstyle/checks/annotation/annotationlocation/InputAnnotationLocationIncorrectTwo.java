package com.puppycrawl.tools.checkstyle.checks.annotation.annotationlocation;

public class InputAnnotationLocationIncorrectTwo {
    @MyAnnotation1
            (value = "")
@MyAnn_21 class Foo {
        public void method1(@MyAnnotation3 @MyAnn_21 Object param1) {
            try {} catch (@MyAnnotation3 @MyAnn_21 Exception e) {}
        }
    }
}
