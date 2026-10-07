package spoon.test.lambda.testclasses;

import java.util.function.Function;

public class LambdaRxJava {
    public interface NbpOperator extends Function<String, Integer> {
    }
    public Integer bla(NbpOperator toto) {
        return toto.apply("truc");
    }
    public void toto() {
        bla((NbpOperator) ((t) -> {
            return t.length();
        }));
    }
}
