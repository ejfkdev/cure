class at extends ap {
    final an d;
    private static final String e;
    at(an var1, String var2, String var3) {
        super(var2, var3);
        this.d = var1;
    }
    public double d() {
        return (double) an.a(this.d).getHeapMemoryUsage().getUsed() / 1024.0 / 1024.0;
    }
    public String c() {
        return e;
    }
    public Double e() {
        return (double) an.a(this.d).getHeapMemoryUsage().getMax() / 1024.0 / 1024.0;
    }
    static {
        e = "MB";
    }
}
