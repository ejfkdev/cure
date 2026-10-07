import java.lang.classfile.Attributes;
import java.lang.classfile.ClassFile;
import java.lang.constant.ClassDesc;
import java.lang.reflect.AccessFlag;
import java.util.Map;
import jdk.test.lib.compiler.InMemoryJavaCompiler;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertEquals;

class RepeatableInLambdaTest {
    static final String src = """
            import java.lang.annotation.Repeatable;
            import java.lang.annotation.Target;
            import static java.lang.annotation.ElementType.TYPE_USE;
            import java.util.function.Supplier;

            @Target(TYPE_USE)
            @Repeatable(AC.class)
            @interface A {
            }

            @Target(TYPE_USE)
            @interface AC {
                A[] value();
            }

            @Target(TYPE_USE)
            @interface B {}

            class Test {
                void test() {
                    Supplier<Integer> s = () -> (@A @A @B Integer) 1;
                }
            }
            """;
    @Test void test() {
        var bytes = InMemoryJavaCompiler.compile(Map.of("Test", src)).get("Test");
        var lambdaMethod = ClassFile.of().parse(bytes).methods().stream().filter((mm) -> mm.flags().has(AccessFlag.SYNTHETIC)).findFirst().orElseThrow();
        System.err.println(lambdaMethod);
        var annoList = lambdaMethod.code().orElseThrow().findAttribute(Attributes.runtimeInvisibleTypeAnnotations()).orElseThrow().annotations();
        assertEquals(2, annoList.size());
        assertEquals(ClassDesc.of("AC"), annoList.getFirst().annotation().classSymbol());
        assertEquals(ClassDesc.of("B"), annoList.get(1).annotation().classSymbol());
    }
}
