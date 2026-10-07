class VarargsAndWildcardParameterizedTypeTest {
    interface I<T> {
        String m(T... t);
    }
    void m() {
        null.m(Integer.valueOf(1), Integer.valueOf(1));
    }
}
