import java.util.concurrent.TimeUnit;

public class ax {
    private long a = 0L;
    private static final String[] b;
    public static ax a() {
        return new ax();
    }
    private ax() {
        this.b();
    }
    public void b() {
        this.a = System.nanoTime();
    }
    public long c() {
        return TimeUnit.NANOSECONDS.toMillis(System.nanoTime() - this.a);
    }
    public String d() {
        return this.c() + b[0];
    }
    public String a(boolean var1) {
        String var2 = String.valueOf(((double) System.nanoTime() - (double) this.a) / 1000.0);
        if (var1) {
            this.b();
        }
        return var2;
    }
    public String e() {
        String var1 = this.c() + b[1];
        this.b();
        return var1;
    }
    static {
        b = new String[] {"ms", "ms"};
    }
}
