public class GuardsErrors {
    void typeTestPatternSwitchTest(Object o, int check) {
        switch (o) {
            case Integer i when i == check -> System.err.println();
            default -> {}
        }
        check = 0;
    }
    void variablesInGuards(Object o) {
        int i1;
        int i2 = 0;
        switch (o) {
            case Integer v when (i1 = 0) == 0 -> {}
            case Integer v when i2++ == 0 -> {}
            case Integer v when ++i2 == 0 -> {}
            case Integer v when new Predicate() {
                public boolean test() {
                    final int i;
                    i = 2;
                    return i == 2;
                }
            }.test() -> {}
            case Integer v when v != null -> {
                v = null;
            }
            case Number v1 when v1 instanceof Integer v2 && (v2 = 0) == 0 -> {}
            default -> {}
        }
    }
    GuardsErrors(Object o) {
        switch (o) {
            case Integer v when (f = 0) == 0 -> {}
            default -> throw new RuntimeException();
        }
    }
    final int f;
    interface Predicate {
        public boolean test();
    }
}
