import java.lang.management.MemoryPoolMXBean;

class aw extends ap {
    private MemoryPoolMXBean d;
    final an e;
    private static final String[] f;
    public aw(an var1, MemoryPoolMXBean var2) {
        super(f[1], f[0] + var2.getName());
        this.e = var1;
        this.d = var2;
    }
    public double d() {}
    public String c() {
        return f[2];
    }
    public Double e() {}
    static {
        f = new String[] {"MEM-", "JVM", "MB"};
    }
}
