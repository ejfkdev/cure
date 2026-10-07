import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;
import java.util.function.BiFunction;
import java.util.function.Function;

public class LocalVariableSyntaxForLambdaParameters {
    @Target({ElementType.PARAMETER})
    @Retention(RetentionPolicy.RUNTIME)
    public @interface Nonnull {
    }
    public void createLambdas() {
        Function<Integer, String> lambda1 = (var x) -> String.valueOf(x);
    }
    public void createAnnotatedLambdaParameters() {
        Function<Integer, String> lambda1 = (@Nonnull var x) -> String.valueOf(x);
    }
}
