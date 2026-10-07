import java.util.List;

public class LambdaBug2783 {
    public Spec<String> test() {
        Spec<String> result = (Spec<String>) ((a, b) -> {
            return a.toArray(String[]::new);
        });
        result = (a, b) -> {
            return a.toArray(String[]::new);
        };
        result = (Spec<String>) ((a, b) -> a.toArray(String[]::new));
        return (Spec<String>) ((a, b) -> {
            return a.toArray(String[]::new);
        });
    }
    interface Spec<T> {
        String[] process(List<T> var1, List<?> var2);
    }
}
