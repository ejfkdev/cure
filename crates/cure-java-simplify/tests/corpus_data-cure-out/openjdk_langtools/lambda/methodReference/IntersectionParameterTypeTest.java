import java.util.function.BiFunction;

public class IntersectionParameterTypeTest {
    sealed interface Term {
        record Lit() implements Term {
        }
        record Lam(String x, Term a) implements Term {
        }
    }
    public static <U, T> void call(BiFunction<U, T, T> op, U x, T t) {
        op.apply(x, t);
    }
    public static void main(String[] args) {
        call(Term.Lam::new, "x", (Term) new Term.Lit());
        call(Term.Lam::new, "x", new Term.Lit());
    }
}
