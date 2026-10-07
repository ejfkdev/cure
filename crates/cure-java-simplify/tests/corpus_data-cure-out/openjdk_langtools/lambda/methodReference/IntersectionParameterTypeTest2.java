public class IntersectionParameterTypeTest2 {
    public static void main(String[] args) {
        f();
    }
    static <T extends Comparable<T> & G> C<T> f() {
        return new C<>(Q::g);
    }
    public interface G {
    }
    private interface E<T> {
        void g(Q g, T value);
    }
    static class C<T extends Comparable<?>> {
        C(E<T> g) {}
    }
    static class Q {
        void g(G g) {}
    }
}
