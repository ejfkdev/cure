package com.puppycrawl.tools.checkstyle.checks.annotation.openjdkannotationlocation;

import java.lang.annotation.ElementType;
import java.lang.annotation.Repeatable;
import java.lang.annotation.Target;

public class InputOpenjdkAnnotationLocation4 {
    @Annotation void exampleGood() {}
    @Annotation public String method() {
        return "";
    }
    @Annotation @Annotation
    @Annotation @Annotation
    public void badMethod() {}
    @Annotation @Annotation @Annotation @Annotation
    public void goodMethod() {}
    @Annotation @Annotation class Temp {
    }
    void methodNoAnnotation() {}
    void parameterlessSamelineInForEach() {
        @Annotation
                @Annotation int temp1;
                // violation above 'Annotations must be on a separate line from 'temp1'.'
        for (Object o : new Object[0]) 
            break;
        for (Object o : new Object[0]) 
            break;
        for (Object o; ; ) 
            break;
        for (Object o; ; ) 
            break;
    }
    @Repeatable(Annotations.class)
    @Target({ElementType.METHOD, ElementType.LOCAL_VARIABLE, ElementType.TYPE}) @interface Annotation {
        String value() default "";
    }
    @Target({ElementType.METHOD, ElementType.LOCAL_VARIABLE, ElementType.TYPE}) @interface Annotations {
        Annotation[] value();
    }
}
