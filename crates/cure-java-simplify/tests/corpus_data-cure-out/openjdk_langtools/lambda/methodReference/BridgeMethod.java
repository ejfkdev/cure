import java.lang.reflect.Method;
import java.util.HashSet;
import java.util.Set;

public class BridgeMethod {
    interface H {
        Object m();
    }
    interface K<T> {
        void m(T t);
    }
    interface L extends K<String> {
    }
    interface M {
        void m(String s);
    }
    interface KM extends K<String>, M {
    }
    interface N extends H {
        String m();
    }
    private static void assertTrue(boolean cond) {
        if (!cond) 
            throw new AssertionError();
    }
    static void bar(String s) {
        System.out.println("BridgeMethod.bar(String) " + s);
    }
    String moo() {
        return "moo";
    }
    private static Set<String> setOfStringObject() {
        Set<String> s = new HashSet<>();
        s.add("java.lang.String");
        s.add("java.lang.Object");
        return s;
    }
    public static void main(String[] args) {
        L la = BridgeMethod::bar;
        la.m("hi");
        Method[] methods = la.getClass().getDeclaredMethods();
        Set<String> types = setOfStringObject();
        System.out.println("methods in SAM conversion of L:");
        for (Method m : methods) {
            assertTrue(m.getName().equals("m"));
            System.out.println(m.toGenericString());
            Class[] parameterTypes = m.getParameterTypes();
            assertTrue(parameterTypes.length == 1);
            assertTrue(types.remove(parameterTypes[0].getName()));
        }
        assertTrue(types.isEmpty() || types.size() == 1 && types.contains("java.lang.String"));
        KM km = BridgeMethod::bar;
        methods = km.getClass().getDeclaredMethods();
        types = setOfStringObject();
        System.out.println("methods in SAM conversion of KM:");
        for (Method m : methods) {
            assertTrue(m.getName().equals("m"));
            System.out.println(m.toGenericString());
            Class<?>[] parameterTypes = m.getParameterTypes();
            assertTrue(parameterTypes.length == 1);
            assertTrue(types.remove(parameterTypes[0].getName()));
        }
        assertTrue(types.isEmpty());
        N n = new BridgeMethod()::moo;
        assertTrue(n.m().equals("moo"));
        assertTrue(((H) n).m().equals("moo"));
        methods = n.getClass().getDeclaredMethods();
        types = setOfStringObject();
        System.out.println("methods in SAM conversion of N:");
        for (Method m : methods) {
            System.out.println(m.toGenericString());
            if (m.getName().equals("m")) {
                Class<?> returnType = m.getReturnType();
                assertTrue(types.remove(returnType.getName()));
            }
        }
        assertTrue(types.size() == 1);
    }
}
