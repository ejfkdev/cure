import java.lang.classfile.*;
import java.lang.classfile.attribute.*;
import java.io.*;
import java.lang.annotation.*;
import java.util.*;

public class LocalVariableTable {
    public static void main(String... args) throws Exception {
        new LocalVariableTable().run();
    }
    void run() throws Exception {
        Class<?>[] classes = getClass().getDeclaredClasses();
        Arrays.sort(classes, (c1, c2) -> c1.getName().compareTo(c2.getName()));
        for (Class<?> c : classes) {
            if (c.getSimpleName().startsWith("Pattern")) 
                check(c);
        }
        if (errors > 0) 
            throw new Exception(errors + " errors found");
    }
    void check(Class<?> c) throws Exception {
        System.err.println("Checking " + c.getSimpleName());
        Expect expect = c.getAnnotation(Expect.class);
        if (expect == null) {
            error("@Expect not found for class " + c.getSimpleName());
            return;
        }
        MethodModel m = getMethodByName(ClassFile.of().parse(Objects.requireNonNull(getClass().getResource(c.getName() + ".class")).openStream().readAllBytes()), c.getSimpleName().contains("Lambda") ? "lambda$" : "test");
        if (m == null) {
            error("test method not found");
            return;
        }
        CodeAttribute code = m.findAttribute(Attributes.code()).orElse(null);
        if (code == null) {
            error("Code attribute not found");
            return;
        }
        LocalVariableTableAttribute lvt = code.findAttribute(Attributes.localVariableTable()).orElse(null);
        if (lvt == null) {
            error("LocalVariableTable attribute not found");
            return;
        }
        Set<String> foundNames = new LinkedHashSet<>();
        for (LocalVariableInfo e : lvt.localVariables()) {
            foundNames.add(e.name().stringValue());
        }
        Set<String> expectNames = new LinkedHashSet<>(Arrays.asList(expect.value()));
        if (!foundNames.equals(expectNames)) {
            Set<String> foundOnly = new LinkedHashSet<>(foundNames);
            foundOnly.removeAll(expectNames);
            for (String s : foundOnly) 
                error("Unexpected name found: " + s);
            Set<String> expectOnly = new LinkedHashSet<>(expectNames);
            expectOnly.removeAll(foundNames);
            for (String s : expectOnly) 
                error("Expected name not found: " + s);
        }
    }
    MethodModel getMethodByName(ClassModel cf, String name) {
        for (MethodModel m : cf.methods()) {
            if (m.methodName().stringValue().startsWith(name)) 
                return m;
        }
        return null;
    }
    void error(String msg) {
        System.err.println("Error: " + msg);
        errors++;
    }
    int errors;
    @Retention(RetentionPolicy.RUNTIME) @interface Expect {
        String[] value();
    }
    @Expect({ "o", "s" })
    static class Pattern_Simple {
        public static void test(Object o) {
            if (o instanceof String s) {
                s.length();
            }
        }
    }
    @Expect({ "s" })
    static class Pattern_Lambda {
        public static void test(Object o) {
            if (o instanceof String s) {
                Runnable r = () -> {
                    s.length();
                };
            }
        }
    }
}
