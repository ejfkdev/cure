import pack.I;
import pack.J;
import java.lang.reflect.Method;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.function.Function;
import java.util.Arrays;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.function.BiFunction;

public final class ProtectedInaccessibleMethodRefTest2 extends I {
    public static void main(String... args) {
        new ProtectedInaccessibleMethodRefTest2().test(Paths.get("test"));
        Set<String> methodNames = new HashSet<>();
        for (Method meth : ProtectedInaccessibleMethodRefTest2.class.getDeclaredMethods()) {
            methodNames.add(meth.getName());
        }
        List<String> expectedMethods = Arrays.asList("lambda$test$0", "lambda$test$1", "lambda$test$2");
        if (!methodNames.containsAll(expectedMethods)) {
            throw new AssertionError("Did not find evidence of new code generation");
        }
    }
    void test(Path outputDir) {
        new Sub(this::readFile).check(outputDir);
        new Sub(ProtectedInaccessibleMethodRefTest2::readFile, this).check(outputDir);
        new Sub(ProtectedInaccessibleMethodRefTest2::readFile2).check(outputDir);
    }
    public class Sub extends J {
        Sub(Function<Path,String> fileReader) {
            super(fileReader);
        }
        Sub(BiFunction<ProtectedInaccessibleMethodRefTest2, Path,String> fileReader, ProtectedInaccessibleMethodRefTest2 instance) {
            super((p) -> fileReader.apply(instance, p));
        }
    }
}
