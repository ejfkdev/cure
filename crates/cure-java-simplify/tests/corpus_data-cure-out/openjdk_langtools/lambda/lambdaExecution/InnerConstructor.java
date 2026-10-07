import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

public class InnerConstructor {
    @Test
    public void testLambdaWithInnerConstructor() {
        assertEquals("Cbl:nada", seq1().m().toString());
        assertEquals("Cbl:rats", seq2().m("rats").toString());
    }
    Ib1 seq1() {
        return () -> {
            return new Cbl();
        };
    }
    Ib2 seq2() {
        return (x) -> {
            return new Cbl(x);
        };
    }
    class Cbl {
        String val;
        Cbl() {
            this.val = "nada";
        }
        Cbl(String z) {
            this.val = z;
        }
        public String toString() {
            return "Cbl:" + val;
        }
    }
    interface Ib1 {
        Object m();
    }
    interface Ib2 {
        Object m(String x);
    }
}
