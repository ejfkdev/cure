public class LambdaConv30 {
    public static void main(String[] args) {
        Integer a = 1;
        class Inner {
                    int i;
                    Inner(int i) {
                        this.i = i;
                    }

                    public int result() {
                        return a * 1000 + i;
                    }
                }
                SAM s = v -> new Inner(v) { }.result();
        if (s.m(2) != 1002) {
            throw new AssertionError("Unexpected value!");
        }
    }
    interface SAM {
        int m(int v);
    }
}
