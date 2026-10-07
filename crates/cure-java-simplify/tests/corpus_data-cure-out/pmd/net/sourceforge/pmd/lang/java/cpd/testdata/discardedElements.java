package a.b.c;

import static org.junit.Assert.assertEquals;
import java.io.IOException;
import java.util.List;
import java.util.Properties;

@Foo // ignored
public class Foo {
    int x;
    {
        x++;
        foo();
    }
    @AnnotationWithParams("ugh")
    @AnnotationWithParams({@Nested(1) ,
                           @Nested(2) ,
                           @Nested
        })
    public void foo() {}
}
