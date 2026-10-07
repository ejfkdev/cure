import java.util.HashMap;
import java.util.Map;

class T6227617 {
    void m() {
        String s = (String) "";
        Object o = (Object) "";
        Integer I1 = (Integer) new HashMap<String, Integer>().get("");
    }
    static final int i1 = Foo.i1;
    static final String s = Foo.s;
}

class Foo {
    static final int i1 = (int) 1;
    static final int i2 = (int) 1L;
    static final String s = (String) "abc";
}
