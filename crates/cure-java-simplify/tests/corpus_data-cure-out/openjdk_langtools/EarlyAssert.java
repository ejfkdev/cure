class EASuper {
    static {
        EarlyAssert.foo();
    }
}

public class EarlyAssert extends EASuper {
    static public void foo() {}
    public static void main(String[] args) {
        throw new Error("Assertions are not disabled after initialization as they should be.");
    }
}
