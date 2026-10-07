import java.io.IOException;
import java.lang.annotation.Target;
import java.lang.annotation.ElementType;
import java.util.Objects;

public class Records {
    @Target(ElementType.TYPE_USE) @interface Nullable {
    }
    @Target({ElementType.CONSTRUCTOR, ElementType.PARAMETER}) @interface MyAnnotation {
    }
    public record MyComplex(int real, @Deprecated int imaginary) {
        @MyAnnotation
        public MyComplex(@MyAnnotation int real, int imaginary) {
            if (real > 100) 
                throw new IllegalArgumentException("too big");
            this.real = real;
            this.imaginary = imaginary;
        }
        public record Nested(int a) {
        }
        public static class NestedClass {
        }
    }
    public record Range(int lo, int hi) {
        @MyAnnotation
        public Range {
            if (lo > hi) 
                throw new IllegalArgumentException(String.format("(%d,%d)", lo, hi));
        }
        public void foo() {}
    }
    public record RecordWithLambdaInCompactConstructor(String foo) {
        public RecordWithLambdaInCompactConstructor {
            Objects.requireNonNull(foo, () -> "foo");
        }
    }
    public record VarRec(@Nullable @Deprecated String @Nullable... x) {
    }
    public record ArrayRec(int[] x) {
    }
    public record EmptyRec<Type>() {
        public void foo() {}
        public Type bar() {
            return null;
        }
        public static void baz() {
            EmptyRec<String> r = new EmptyRec<>();
            System.out.println(r);
        }
    }
    public interface Person {
        String firstName();
        String lastName();
    }
    public record PersonRecord(String firstName, String lastName) implements Person, java.io.Serializable {
    }
}
