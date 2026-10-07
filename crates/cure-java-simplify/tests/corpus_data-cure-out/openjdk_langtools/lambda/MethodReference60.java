public class MethodReference60 {
    interface ArrayFactory<X> {
        X make(int size);
    }
    interface BadArrayFactory1<X> {
        X make();
    }
    interface BadArrayFactory2<X> {
        X make(int i1, int i2);
    }
    interface BadArrayFactory3<X> {
        X make(String s);
    }
    public static void meth() {
        ArrayFactory<Integer[]> factory5 = int[]::new;
    }
}
