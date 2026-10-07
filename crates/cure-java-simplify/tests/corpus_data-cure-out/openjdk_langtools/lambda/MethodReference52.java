import java.util.*;

class MethodReference52 {
    interface Clone1 {
        int[] m();
    }
    interface Clone2 {
        Object m();
    }
    interface WrongClone {
        long[] m();
    }
    interface GetClass {
        Class<? extends List> m();
    }
    interface WrongGetClass {
        Class<List<String>> m();
    }
    void test(int[] iarr, List<String> ls) {}
}
