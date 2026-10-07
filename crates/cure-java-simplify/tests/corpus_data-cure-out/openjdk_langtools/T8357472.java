class T8357472 {
    class A<T> {
        protected class B<V> {
        }
        public static <T, M extends A<T>> void f(Object g) {
            @SuppressWarnings("unchecked")
                        M.B<?> mapping = (M.B<?>) g;
            M.B<?>[] mapping2 = new M.B[1];
            mapping2[0] = mapping;
        }
    }
}
