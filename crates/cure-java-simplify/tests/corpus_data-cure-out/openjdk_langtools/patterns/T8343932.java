public class T8343932 {
    abstract sealed class J<T1, T2> permits X.S, A {
    }
    final class A extends J<Integer, Integer> {
    }
    public class X<T> {
        final class S<U> extends J<T, U> {
            abstract sealed class J<T1, T2> permits XX.SS, AA {
            }
            final class AA extends J<Integer, Integer> {
            }
            public class XX<T> {
                final class SS<U> extends J<T, U> {
                }
            }
        }
        static int test(J<Integer, Integer> ji) {
            return switch (ji) {
                case A a -> 42;
                case X<Integer>.S<Integer> e -> 4200;
            };
        }
        static int test(X<Integer>.S<Integer>.J<Integer, Integer> ji) {
            return switch (ji) {
                case X<Integer>.S<Integer>.AA a -> 42;
                case X<Integer>.S<Integer>.XX<Integer>.SS<Integer> e -> 4200;
            };
        }
    }
}
