class T8357653b {
    class A<T> {
        class B<W> {
            public T rett() {
                return null;
            }
        }
    }
    class C extends A<String> {
        static class D {
            {
                String s = null.rett();
                String s2 = new B[1][0].rett();
            }
        }
    }
}
