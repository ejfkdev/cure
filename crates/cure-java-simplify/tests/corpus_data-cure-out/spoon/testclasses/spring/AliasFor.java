package spoon.test.annotation.testclasses.spring;

import java.lang.annotation.Annotation;
import java.lang.annotation.Documented;
import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.METHOD)
@Documented
public @interface AliasFor {
    @AliasFor("attribute")
        String value() default "";

        /**
         * The name of the attribute that <em>this</em> attribute is an alias for.
         * @see #value
         */
    @AliasFor("value")
        String attribute() default "";

        /**
         * The type of annotation in which the aliased {@link #attribute} is declared.
         * <p>Defaults to {@link Annotation}, implying that the aliased attribute is
         * declared in the same annotation as <em>this</em> attribute.
         */
    Class<? extends Annotation> annotation() default Annotation.class;
}
