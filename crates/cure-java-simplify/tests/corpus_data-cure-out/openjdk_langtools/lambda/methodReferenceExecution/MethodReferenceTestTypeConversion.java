import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

class MethodReferenceTestTypeConversion_E<T> {
    T xI(T t) {
        return t;
    }
}

public class MethodReferenceTestTypeConversion {
    interface ISi {
        int m(Short a);
    }
    interface ICc {
        char m(Character a);
    }
    @Test
    public void testUnboxObjectToNumberWiden() {
        ISi q = new MethodReferenceTestTypeConversion_E<Short>()::xI;
        assertEquals((short) 77, q.m((short) 77));
    }
    @Test
    public void testUnboxObjectToChar() {
        ICc q = new MethodReferenceTestTypeConversion_E<Character>()::xI;
        assertEquals('@', q.m('@'));
    }
}
