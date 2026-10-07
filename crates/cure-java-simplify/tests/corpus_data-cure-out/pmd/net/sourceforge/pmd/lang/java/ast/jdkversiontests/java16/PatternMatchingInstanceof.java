import static java.lang.annotation.ElementType.CONSTRUCTOR;
import static java.lang.annotation.ElementType.FIELD;
import static java.lang.annotation.ElementType.LOCAL_VARIABLE;
import static java.lang.annotation.ElementType.METHOD;
import static java.lang.annotation.ElementType.MODULE;
import static java.lang.annotation.ElementType.PACKAGE;
import static java.lang.annotation.ElementType.PARAMETER;
import static java.lang.annotation.ElementType.TYPE;
import java.lang.annotation.ElementType;
import java.lang.annotation.Target;

public class PatternMatchingInstanceof {
    private String s = "other string";
    public void test() {
        Object obj = "abc";
        if (obj instanceof String s) {
            System.out.println("a) obj == s: " + (obj == s));
            s = "other value";
            System.out.println("changed s to " + s + ": obj == s: " + (obj == s));
        } else {
            System.out.println("b) obj == s: " + (obj == s));
        }
        if (!(obj instanceof String s)) {
            System.out.println("c) obj == s: " + (obj == s));
        } else {
            System.out.println("d) obj == s: " + (obj == s));
        }
        if (obj instanceof String s && s.length() > 2) {
            System.out.println("e) obj == s: " + (obj == s));
        }
        if (obj instanceof String s || s.length() > 5) {
            System.out.println("f) obj == s: " + (obj == s));
        }
        if (obj instanceof String s) {
            System.out.println("g) obj == s: " + (obj == s));
        } else {
            System.out.println("h) obj == s: " + (obj == s));
        }
        if (obj instanceof String s) {
            System.out.println("i) obj == s: " + (obj == s));
        } else {
            System.out.println("j) obj == s: " + (obj == s));
        }
        if (obj instanceof String s) {
            System.out.println("k) obj == s: " + (obj == s));
        } else {
            System.out.println("l) obj == s: " + (obj == s));
        }
    }
    public static void main(String[] args) {
        new PatternMatchingInstanceof().test();
    }
    class Foo {
        {}
    }
    @Target(value=ElementType.TYPE_USE) @interface Nullable {
    }
}
