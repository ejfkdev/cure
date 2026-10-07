import java.util.*;

public class LambdaNoFuncIntfFlow {
    private void t(Object i) {
        int j = i instanceof ArrayList ? (ArrayList<String>) i : (() -> {
            return null;
        });
        Runnable r = () -> t(0);
    }
}
