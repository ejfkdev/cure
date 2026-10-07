package spoon.test.position.testclasses;

import java.io.Serializable;
import java.util.List;
import java.util.Map;

public class Expressions {
    void method() {
        System.out.print("x");
        System.out.print("x");
        System.out.print((String) null);
        System.out.print((String) (Serializable) null);
        System.out.print((String) null);
        System.out.print((String) null);
        System.out.print((List<?>) null);
        System.out.print((List<List<Map<String,Integer>>>) null);
    }
}
