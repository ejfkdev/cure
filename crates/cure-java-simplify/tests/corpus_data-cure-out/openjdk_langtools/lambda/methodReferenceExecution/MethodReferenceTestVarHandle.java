import java.lang.invoke.*;
import java.util.*;
import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

public class MethodReferenceTestVarHandle {
    interface Setter {
        void apply(int[] arr, int idx, int val);
    }
    interface Getter {
        int apply(int[] arr, int idx);
    }
    @Test
  public void testSet() throws Throwable {
        Setter f = MethodHandles.arrayElementVarHandle(int[].class)::set;
        int[] data = {0};
        f.apply(data, 0, 42);
        assertEquals(42, data[0]);
    }
    @Test
  public void testGet() throws Throwable {
        Getter f = MethodHandles.arrayElementVarHandle(int[].class)::get;
        int[] data = {42};
        assertEquals(42, f.apply(data, 0));
    }
}
