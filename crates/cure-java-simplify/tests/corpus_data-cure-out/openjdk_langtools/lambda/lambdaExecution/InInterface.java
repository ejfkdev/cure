import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

interface LTII {
    interface ILsp1 {
        String m();
    }
    interface ILsp2 {
        String m(String x);
    }
    default ILsp1 t1() {
        return () -> {
            return "yo";
        };
    }
    default ILsp2 t2() {
        return (x) -> {
            return "snur" + x;
        };
    }
}

public class InInterface implements LTII {
    @Test
    public void testLambdaInDefaultMethod() {
        assertEquals("yo", t1().m());
        assertEquals("snurp", t2().m("p"));
    }
}
