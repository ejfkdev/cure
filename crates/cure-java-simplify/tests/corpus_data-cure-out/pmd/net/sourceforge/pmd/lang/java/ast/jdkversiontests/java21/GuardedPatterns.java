public class GuardedPatterns {
    static void test(Object o) {
        switch (o) {
            case String s when s.length() == 1 -> System.out.println("single char string");
            case String s -> System.out.println("string");
            case Integer i when i.intValue() == 1 -> System.out.println("integer 1");
            default -> System.out.println("default case");
        }
    }
    void testIdentifierWhen(String when) {
        System.out.println(when);
    }
    void testIdentifierWhen() {
        System.out.println(1);
    }
    private static class when {
    }
    static void testWithNull(Object o) {
        switch (o) {
            case String s when (s.length() == 1) -> System.out.println("single char string");
            case String s -> System.out.println("string");
            case Integer i when i.intValue() == 1 -> System.out.println("integer 1");
            case null -> System.out.println("null!");
            default -> System.out.println("default case");
        }
    }
    static void instanceOfPattern(Object o) {
        if (o instanceof String s && s.length() > 2) {
            System.out.println("A string containing at least two characters");
        }
        if (o != null && (o instanceof String s && s.length() > 3)) {
            System.out.println("A string containing at least three characters");
        }
        if (o instanceof String s && s.length() > 4) {
            System.out.println("A string containing at least four characters");
        }
    }
    static void testScopeOfPatternVariableDeclarations(Object obj) {
        if (obj instanceof String s && s.length() > 3) {
            System.out.println(s);
        } else {
            System.out.println("Not a string");
        }
    }
    public static void main(String[] args) {
        test("a");
        test("fooo");
        test(1);
        test(1L);
        instanceOfPattern("abcde");
        try {
            test(null);
        } catch (NullPointerException e) {
            e.printStackTrace();
        }
        testWithNull(null);
        testScopeOfPatternVariableDeclarations("a");
        testScopeOfPatternVariableDeclarations("long enough");
    }
}
