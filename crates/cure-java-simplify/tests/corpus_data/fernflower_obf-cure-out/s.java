import java.util.LinkedHashMap;
import java.util.Map;

public class s {
    private final String a;
    private Map<String, String> b = new LinkedHashMap();
    private Throwable c;
    public static int d;
    private static final String[] e;
    public s(String var1) {
        this.a = var1;
    }
    public s a(Throwable var1) {
        this.c = var1;
        return this;
    }
    public s a(String var1, Object var2) {
        if (var2 != null) {
            this.b.put(var1, var2.toString());
        }
        return this;
    }
    public a9 a() {
        return r.b(this.b());
    }
    protected p b() {}
    protected static String b(Throwable var0) {
        String var1 = "-";
        if (var0 != null) {
            StackTraceElement[] var2 = var0.getStackTrace();
            if (var2.length > 0) {
                var1 = var2[0].getFileName() + ":" + var2[0].getLineNumber() + "[" + var2[0].getClassName() + "." + var2[0].getMethodName() + "]";
            }
        } else {
            StackTraceElement[] var3 = Thread.currentThread().getStackTrace();
            if (var3.length > 5) {
                var1 = var3[5].getFileName() + ":" + var3[5].getLineNumber() + "[" + var3[5].getClassName() + "." + var3[5].getMethodName() + "]";
            }
        }
        return var1;
    }
    static {
        e = new String[] {", Exception: ", "origin", ": ", " = ", ", "};
    }
}
