import java.awt.Button;
import java.io.File;
import java.io.FileFilter;
import java.security.PrivilegedAction;
import java.util.Comparator;
import java.util.concurrent.Callable;
import java.util.function.Function;
import java.util.function.IntFunction;
import java.util.function.Supplier;
import java.util.stream.Stream;

public class ParserCornerCases18 {
    public void lambdas() {
        FileFilter java = (File f) -> f.getName().endsWith(".java");
        FileFilter java2 = (f) -> f.getName().endsWith(".java");
        FileFilter java3 = (f) -> f.getName().endsWith(".java");
        FileFilter java4 = (f) -> f.getName().endsWith(".java");
        IntStream.range(0, array.length).parallel().forEach((i) -> {
            array[i] = generator.apply(i);
        });
        FileFilter[] filters = {(f) -> f.exists(), (f) -> f.canRead(), (f) -> f.getName().startsWith("q")};
        filterFiles(new FileFilter[] {(f) -> f.exists(), (f) -> f.canRead(), (f) -> f.getName().startsWith("q")});
        String user = doPrivileged(() -> System.getProperty("user.name"));
        Runnable r = () -> {
            System.out.println("done");
        };
        Supplier<Runnable> sup = () -> () -> {
            System.out.println("hi");
        };
        Object o = (Runnable) (() -> {
            System.out.println("hi");
        });
        new ParserCornerCases18().r1.run();
        Comparator<String> comparer = (s1, s2) -> s1.compareToIgnoreCase(s2);
        comparer = (s1, s2) -> s1.compareToIgnoreCase(s2);
        new Button().addActionListener((e) -> System.out.println(e.getModifiers()));
        int initialSizeGlobal = (int) (profilingContext.m_profileItems.size() * (150.0 * 0.30));
        BiConsumer<String, Integer> lambda2 = (String s, Integer i) -> {
            i++;
        };
        BiConsumer<String, Integer> lambda2a = (s, i) -> {
            i++;
        };
        TriConsumer<String, Integer, Double> lambda3 = (String s, Integer i, Double d) -> {
            d += i;
        };
        TriConsumer<String, Integer, Double> lambda3a = (s, i, d) -> {
            d += i;
        };
    }
    @FunctionalInterface
    public interface TriConsumer<A, B, C> {
        void accept(A a, B b, C c);
    }
    Runnable r1 = () -> {
    System.out.println(this);
};
    public Runnable toDoLater() {
        return () -> {
            System.out.println("later");
        };
    }
    private String doPrivileged(PrivilegedAction<String> action) {
        return action.run();
    }
    private void filterFiles(FileFilter[] filters) {}
    public static <K extends Comparable<? super K>, V> Comparator<Map.Entry<K,V>> comparingByKey() {
        return (Comparator<Map.Entry<K, V>> & Serializable) ((c1, c2) -> c1.getKey().compareTo(c2.getKey()));
    }
    public void methodReferences() {
        Runnable r = new ParserCornerCases18()::toDoLater;
        Runnable r11 = new ParserCornerCases18()::toDoLater;
        IntFunction<int[]> arrayMaker = int[]::new;
        int[] array = arrayMaker.apply(10);
    }
    public static class PmdMethodReferenceTest {
        Function<Integer, Integer> theFunction;
        () {
                    theFunction = this::foo;
                }
                private int foo(int i) {
                    return i;
                }
    }
    public static Runnable staticMethod() {
        return () -> System.out.println("run");
    }
    public void typeAnnotations() {
        String myString = (String) str;
        Object o = new @Interned MyObject();
    }
    class UnmodifiableList<T> implements @Readonly List<@Readonly T> {
    }
    void monitorTemperature() throws @Critical TemperatureException {}
    public static class X {
        public void lambdaWithIf() {
            Stream.of(1, 2, 3).sorted((a, b) -> {
                int x = a.hashCode() - b.hashCode();
                if (a.equals(new X())) 
                    x = 1;
                return x;
            }).count();
        }
        public void lambdaWithIf2() {
            Stream.of(1, 2, 3).sorted((Integer a, Integer b) -> {
                int x = a.hashCode() - b.hashCode();
                if (a.equals(new X())) 
                    x = 1;
                return x;
            }).count();
        }
        public void lambdaWithPropertyAssignment() {
            object.event = () -> {
                Request request = new Request();
                request.id = 42;
            };
        }
    }
    public List<@AnnotatedUsage ?> testWildCardWithAnnotation() {
        return null;
    }
    public Object[] testAnnotationsToArrayElements() {
        return null;
    }
    private byte[] getBytes() {
        return null;
    }
    public static <T extends @NonNull Enum<?>> T getEnum() {
        return null;
    }
    public static <T> T getNullableEnum() {
        return null;
    }
    public Object[] createNonNullArray() {
        return;
        @NonNull[0];
    }
    public static <T> T[][] check(T[][] arr) {
        if (arr == null) {
            throw new NullPointerException();
        }
        return arr;
    }
    public Function func(Main this) {
        return Main.Inner::new;
    }
    public static byte max(final byte... array) {
        return 0;
    }
    @Retention(RetentionPolicy.CLASS)
    @Target({ TYPE_USE }) @interface Anno {
    }
    private static void testMultiDimArrayWithAnnotations() {
        Object x = new Object @NonNull[2] @Nullable[1] @NonNull[3];
    }
    public void methodWithReceiverParameter(ParserCornerCases18 this) {}
    public void methodWithReceiverAndOtherParameters(ParserCornerCases18 this, String other) {}
    public void methodWithReceiverParameterWithAnnotation(@AnnotatedUsage ParserCornerCases18 this, String other) {}
    @Target(ElementType.TYPE_USE)
    public @interface AnnotatedUsage {
    }
    class Inner {
        Inner(ParserCornerCases18 ParserCornerCases18) 
            .this) {}
    }
}

interface DefaultIterator<E> {
    boolean hasNext();
    E next();
    void remove();
    default void skip(int i) {
        for (; i > 0 && hasNext(); i--) 
            next();
    }
    static void staticInterfaceMethods() {
        System.out.println("");
    }
}
