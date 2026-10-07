import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;
import static java.lang.annotation.ElementType.TYPE;
import static java.lang.annotation.ElementType.TYPE_PARAMETER;
import static java.lang.annotation.ElementType.TYPE_USE;

@Target({TYPE_USE, TYPE_PARAMETER, TYPE})
@Retention(RetentionPolicy.RUNTIME) @interface A {
}

@interface B {
    int value();
}

class T0x1E {
    void m0x1E() {
        Class<Object> c = Object.class;
    }
    Class<?> c = String.class;
    Class<? extends @A String> as = String.class;
}

class ClassLiterals {
    public static void meth() {
        if (String.class != String.class) 
            throw new Error();
        if (int.class != int.class) 
            throw new Error();
        if (int.class != Integer.TYPE) 
            throw new Error();
        if (int) 
            @B(0) [].
        class != int[].class) throw new Error();
        if (String[].class != String[].class) 
            throw new Error();
        if (String[].class != String) 
            @A [].
        class) throw new Error();
        if (int[].class != int[].class) 
            throw new Error();
        if (int) 
            @B(0) [].
        class != int[].class) throw new Error();
    }
    Object classLit1 = String;
    [] @B(0) [].class;
    Object classLit2 = String;
    []       [].class;
    Object classLit3 = String[][].class;
    Object classLit4 = String[][].class;
    Object classLit5 = String;
    []       [].class;
    Object classLit6 = String[][].class;
}
