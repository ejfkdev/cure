import static org.junit.jupiter.api.Assertions.assertEquals;
import org.junit.jupiter.api.Test;

public class MethodReferenceTestNewInnerImplicitArgs {
    static class S {
        String b;
        S(String s, String s2) {
            b = s + s2;
        }
    }
    interface I {
        S m();
    }
    interface I2 {
        S m(int i, int j);
    }
    @Test
    public void testConstructorReferenceImplicitParameters() {
        String title = "Hey";
        String a2 = "!!!";
        class MS extends S {
                    MS() {
                        super(title, a2);
                    }
                }

                I result = MS::new;
        assertEquals("Hey!!!", result.m().b);
        class MS2 extends S {
                    MS2(int x, int y) {
                        super(title+x, a2+y);
                    }
                }

                I2 result2 = MS2::new;
        assertEquals("Hey8!!!4", result2.m(8, 4).b);
    }
}
