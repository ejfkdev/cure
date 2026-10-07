import java.io.IOException;
import java.io.InputStream;
import java.lang.classfile.Attributes;
import java.lang.classfile.ClassFile;
import java.lang.classfile.ClassModel;
import java.lang.classfile.MethodModel;
import java.lang.classfile.attribute.CodeAttribute;
import java.lang.classfile.attribute.LocalVariableTableAttribute;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.stream.Collectors;

public class SyntheticVariables {
    public static void main(String[] args) throws IOException {
        try (InputStream in = Test.class.getClassLoader().getResource(Test.class.getName().replace('.', '/') + ".class").openStream()) {
            Map<String, MethodModel> name2Method = ClassFile.of().parse(in.readAllBytes()).methods().stream().collect(Collectors.toMap((m) -> m.methodName().stringValue(), (m) -> m));
            assertEquals(Set.of("str", "b", "b2", "this", "l", "o"), localVars(name2Method.get("testInMethod")));
            assertEquals(Set.of("str", "b", "b2", "l", "o"), localVars(name2Method.get("lambda$testInLambda$0")));
        }
    }
    private static Set<String> localVars(MethodModel method) {
        return method.findAttribute(Attributes.code()).orElseThrow().findAttribute(Attributes.localVariableTable()).orElseThrow().localVariables().stream().map((info) -> info.name().stringValue()).collect(Collectors.toSet());
    }
    private static void assertEquals(Set<String> expected, Set<String> actual) {
        if (!Objects.equals(expected, actual)) {
            throw new AssertionError("Unexpected value, expected: " + expected + ", got: " + actual);
        }
    }
    public record Test(Object o) {
        private void testInMethod() {
            Object o = create();
            boolean b = o.toString() instanceof String str && str.isEmpty();
            System.err.println(b);
            switch (create()) {
                case Test(Test(String str)) when str.isEmpty() -> System.err.println(1);
                case Test(Test(String str)) -> System.err.println(2);
                default -> System.err.println(2);
            }
            boolean b2 = List.of(0).get(0) instanceof int;
            System.err.println(b2);
        }
        private void testInLambda() {
            Object o = create();
            Runnable r = () -> {
                boolean b = o.toString() instanceof String str && str.isEmpty();
                System.err.println(b);
                switch (create()) {
                    case Test(Test(String str)) when str.isEmpty() -> System.err.println(1);
                    case Test(Test(String str)) -> System.err.println(2);
                    default -> System.err.println(2);
                }
                boolean b2 = List.of(0).get(0) instanceof int;
                System.err.println(b2);
            };
        }
        private static String val = "v";
        public static Test create() {
            try {
                return new Test(new Test(val));
            } finally {
                val += val;
            }
        }
    }
}
