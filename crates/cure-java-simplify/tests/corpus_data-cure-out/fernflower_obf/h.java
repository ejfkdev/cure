import java.util.concurrent.atomic.AtomicLong;

public class h {
    private final AtomicLong a = new AtomicLong();
    public long a() {
        boolean var5 = n.c;
        long var1 = this.a.incrementAndGet();
        long var10000;
        label48:
            {
                var10000 = var1;
                if (var5) {
                    return var10000;
                }
                if (var1 <= 9223372036854775797L) {
                    break label48;
                }
                AtomicLong var3;
                synchronized (var3 = this.a) {
                    AtomicLong var9 = this.a;
                    if (!var5) {
                        if (var9.get() > 9223372036854775797L) {
                            this.a.set(0L);
                        }
                        var9 = var3;
                    }
                }
            }
        return var1;
    }
    public String b() {
        return String.valueOf(this.a());
    }
}
