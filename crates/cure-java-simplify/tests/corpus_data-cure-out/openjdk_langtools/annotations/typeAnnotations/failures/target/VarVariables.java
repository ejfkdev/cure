import java.lang.annotation.Target;
import java.lang.annotation.ElementType;
import java.util.List;
import java.util.function.Consumer;

class VarVariables {
    private void test(Object o) {
        @DA var v1 = "";
        @DTA var v2 = "";
        @TA var v3 = "";
        for (var v = ""; !v.isEmpty(); ) {}
        for (var v = ""; !v.isEmpty(); ) {}
        for (var v = ""; !v.isEmpty(); ) {}
        for (var v : List.of("")) {}
        for (var v : List.of("")) {}
        for (var v : List.of("")) {}
        try (var v = open()) {} catch (Exception ex) {}
        try (var v = open()) {} catch (Exception ex) {}
        try (var v = open()) {} catch (Exception ex) {}
        boolean b3 = o instanceof R(@TA var v);
    }
    private AutoCloseable open() {
        return null;
    }
    record R(String str) {
    }
    @Target(ElementType.TYPE_USE) @interface TA {
    }
    @Target({ElementType.TYPE_USE, ElementType.LOCAL_VARIABLE, ElementType.PARAMETER}) @interface DTA {
    }
    @Target({ElementType.LOCAL_VARIABLE, ElementType.PARAMETER}) @interface DA {
    }
}
