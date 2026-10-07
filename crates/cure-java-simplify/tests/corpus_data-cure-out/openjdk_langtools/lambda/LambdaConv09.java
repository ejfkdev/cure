class LambdaConv09 {
    interface Foo1 {
    }
    interface Foo2 {
        boolean equals(Object object);
    }
    interface Foo3 extends Foo2 {
        public abstract String toString();
    }
    interface Foo4<T> extends Foo2 {
        int compare(T o1, T o2);
    }
    interface Foo5 {
        boolean equals(Object object);
        String toString();
    }
    interface Foo6<T> {
        boolean equals(Object obj);
        int compare(T o1, T o2);
    }
    interface Foo7<T> extends Foo2, Foo6<T> {
    }
    void test() {}
}
