import java.lang.invoke.*;
import java.util.*;

public class MethodReferenceTestVarHandle_neg {
    interface Setter {
        int apply(int[] arr, int idx, int val);
    }
    public static void meth() {
        Setter f = MethodHandles.arrayElementVarHandle(int[].class)::set;
    }
}
