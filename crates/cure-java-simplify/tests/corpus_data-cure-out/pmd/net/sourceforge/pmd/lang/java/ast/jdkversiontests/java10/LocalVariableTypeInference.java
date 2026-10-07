import java.util.ArrayList;

public class LocalVariableTypeInference {
    public void aMethod() {
        var stream = new ArrayList<String>().stream();
    }
    public void asMethodParameter() {
        print("Java 10");
    }
    private void print(String text) {
        System.out.println(text);
    }
}
