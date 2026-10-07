import java.io.*;
import java.lang.annotation.*;
import java.util.*;
import java.lang.classfile.*;
import java.lang.classfile.attribute.*;

public class LocalVariableTable {
    public static void main(String... args) throws Exception {
        new LocalVariableTable().run();
    }
    void run() throws Exception {
        Class<?>[] classes = getClass().getDeclaredClasses();
        Arrays.sort(classes, (c1, c2) -> c1.getName().compareTo(c2.getName()));
        for (Class<?> c : classes) {
            if (c.getSimpleName().startsWith("Lambda")) 
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
        MethodModel m = getLambdaMethod(ClassFile.of().parse(Objects.requireNonNull(getClass().getResource(c.getName() + ".class")).openStream().readAllBytes()));
        if (m == null) {
            error("lambda method not found");
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
    MethodModel getLambdaMethod(ClassModel cf) {
        for (MethodModel m : cf.methods()) {
            if (m.methodName().stringValue().startsWith("lambda$")) 
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
    interface Run0 {
        public void run();
    }
    interface Run1 {
        public void run(int a0);
    }
    interface Run2 {
        public void run(int a0, int a1);
    }
    @Expect({ "x" })
    static class Lambda_Args0_Local1 {
        Run0 r = () -> {};
    }
    @Expect({ "x", "this" })
    static class Lambda_Args0_Local1_this {
        int v;
        Run0 r = () -> {};
    }
    @Expect({ "a" })
    static class Lambda_Args1_Local0 {
        Run1 r = (a) -> {};
    }
    @Expect({ "a", "x" })
    static class Lambda_Args1_Local1 {
        Run1 r = (a) -> {};
    }
    @Expect({ "a", "x", "v" })
    static class Lambda_Args1_Local1_Captured1 {
        void m() {}
    }
    @Expect({ "a1", "a2", "x1", "x2", "this", "v1", "v2" })
    static class Lambda_Args2_Local2_Captured2_this {
        int v;
        void m() {}
    }
    @Expect({ "e", "c" })
    static class Lambda_Try_Catch {
        private static Runnable asUncheckedRunnable(Closeable c) {
            return () -> {
                try {
                    c.close();
                } catch (IOException e) {
                    throw new UncheckedIOException(e);
                }
            };
        }
    }
}
