import java.util.function.*;
import java.util.*;

class T8175317 {
    void m(Supplier<List<String>> s) {}
    void testMethodLambda(List l) {
        m(() -> l);
    }
    void testAssignLambda(List l) {}
    void testMethodMref() {
        m(this::g);
    }
    void testAssignMref() {}
    List g() {
        return null;
    }
}
