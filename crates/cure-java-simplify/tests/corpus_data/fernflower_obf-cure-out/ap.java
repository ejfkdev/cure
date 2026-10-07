public abstract class ap implements ak {
    private final String a;
    private final String b;
    public static int c;
    private static final String d;
    public ap(String var1, String var2) {
        this.b = var1;
        this.a = var2;
    }
    public String b() {
        return this.a;
    }
    public String a() {
        return this.b;
    }
    public int hashCode() {
        return (this.b + this.a).hashCode();
    }
    public boolean equals(Object param1) {}
    public String toString() {
        return this.b + d + this.a;
    }
    static {
        d = " - ";
    }
}
