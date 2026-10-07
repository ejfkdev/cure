public class ProxyMethodLookup {
    public static void main(String[] args) {
        boolean b = new R(new Component()) instanceof R(var c);
    }
    interface ComponentBase {
    }
    record Component() implements ComponentBase {
    }
    sealed interface Base {
        ComponentBase c();
    }
    record R(Component c) implements Base {
    }
}
