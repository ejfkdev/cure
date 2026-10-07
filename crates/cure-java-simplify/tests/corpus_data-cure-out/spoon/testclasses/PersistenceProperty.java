package spoon.test.annotation.testclasses;

import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

@Target(value={})
@Retention(RetentionPolicy.RUNTIME)
public @interface PersistenceProperty {
    String name();
    String value();
}
