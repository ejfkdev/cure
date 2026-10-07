public class ArrayVsVarargs {
    void f(String[] arg) {}
    void g(String... arg) {}
    void h(String[] arg) {}
    void i(String... arg) {}
    void j(String[][] arg) {}
    void k(String[]... arg) {}
    Class<?> c = byte[].class;
}
