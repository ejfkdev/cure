package japa.bdd.samples;

@Deprecated
public class JavaConceptsUgly {
    static int x = 0;
    public static void main(String[] args) {
        x = x;
        x = ~x;
        --x;
        boolean b;
        x &= 2;
        x |= 2;
        x ^= 2;
        x -= 2;
        x %= 2;
        x /= 2;
        x *= 2;
        x <<= 2;
        x >>= 2;
        x >>>= 2;
        b = x <= 1;
        x = x << 1;
        x = x >> 1;
        x = x >>> 1;
        x = x - 1;
        x = x * 1;
        x = x % 1;
        x = x / 1;
    }
}
