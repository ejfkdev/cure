import java.lang.annotation.*;
import static java.lang.annotation.RetentionPolicy.*;
import static java.lang.annotation.ElementType.*;
import java.util.List;

class TestAnonInnerInstance1<T> {
    Object mtest(TestAnonInnerInstance1<T> t) {
        return null;
    }
    Object mmtest(TestAnonInnerInstance1<T> t) {
        return null;
    }
    public void test() {
        mtest(new TestAnonInnerInstance1<T>() {
                  class InnerAnon<U> { // Test1$1$InnerAnon.class
                      @A @B @C @D String ia_m1(){ return null; };
                  }
    //If this is commented out, annotations are attributed correctly
                  InnerAnon<String> IA = new InnerAnon< String>();
              });
    }
}

@Retention(RUNTIME) @Target({TYPE_USE,FIELD}) @interface A {
}

@Retention(RUNTIME) @Target({TYPE_USE,METHOD}) @interface B {
}

@Retention(CLASS) @Target({TYPE_USE,FIELD}) @interface C {
}

@Retention(CLASS) @Target({TYPE_USE,METHOD}) @interface D {
}
