import java.lang.classfile.Attributes;
import java.lang.classfile.ClassFile;
import java.lang.classfile.instruction.LoadInstruction;
import java.lang.classfile.instruction.StoreInstruction;
import java.util.BitSet;
import java.util.Map;
import jdk.test.lib.compiler.InMemoryJavaCompiler;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

public class PrimitiveInstanceOfBytecodeTest {
    private static final String SOURCE = """
            public class Test {
                public record A(int i) {}
                public Integer get(A a) {
                    if (a instanceof A(int i)) {
                        return i;
                    }
                    return null;
                }
            }
            """;
    @Test
    public void testNoUnusedVarInRecordPattern() {
        var testBytes = InMemoryJavaCompiler.compile(Map.of("Test", SOURCE)).get("Test");
        var code = ClassFile.of().parse(testBytes).methods().stream().filter((m) -> m.methodName().equalsString("get")).findFirst().orElseThrow().findAttribute(Attributes.code()).orElseThrow();
        BitSet stores = new BitSet(code.maxLocals());
        BitSet loads = new BitSet(code.maxLocals());
        code.forEach((ce) -> {
            switch (ce) {
                case StoreInstruction store -> stores.set(store.slot());
                case LoadInstruction load -> loads.set(load.slot());
                default -> {}
            }
        });
        loads.clear(0, 2);
        stores.clear(0, 2);
        if (!loads.equals(stores)) {
            System.err.println("Loads: " + loads);
            System.err.println("Stores: " + stores);
            System.err.println(code.toDebugString());
            fail("Store and load mismatch, see stderr");
        }
    }
}
