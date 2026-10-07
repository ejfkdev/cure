import java.io.PrintWriter;
import java.io.StringWriter;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Iterator;

public class p {
    protected String a;
    protected String b;
    protected String c;
    private String d;
    private String e;
    private Throwable f;
    private static final String[] g;
    public p(String var1, String var2, Throwable var3, String var4) {
        int var10 = s.d;
        super();
        this.e = "";
        this.a = var1;
        this.c = var2;
        this.f = var3;
        ArrayList var5 = new ArrayList(Arrays.asList(Thread.currentThread().getStackTrace()));
        int var6 = 3;
        while (!var5.isEmpty()) {
            if (var6 <= 0) {
                break;
            }
            var5.remove(0);
            --var6;
            if (var10 == 0) {
                continue;
            }
            int var11 = ap.c;
            ++var11;
            ap.c = var11;
            break;
        }
        StringBuilder var7 = new StringBuilder();
        Iterator var8 = var5.iterator();
        while (true) {
            if (var8.hasNext()) {
                StackTraceElement var9 = (StackTraceElement) var8.next();
                var7.append(var9.getClassName());
                var7.append(".");
                var7.append(var9.getMethodName());
                var7.append(g[0]);
                var7.append(var9.getFileName());
                var7.append(":");
                var7.append(var9.getLineNumber());
                var7.append(")");
                var7.append("\n");
                if (var10 != 0) {
                    break;
                }
                if (var10 == 0) {
                    continue;
                }
            }
            this.d = var7.toString();
            break;
        }
        p var10000;
        String var10001;
        label56:
            {
                label55:
                    {
                        label54:
                            {
                                if (var10 != 0) {
                                    break label55;
                                }
                                if (var3 == null) {
                                    break label54;
                                }
                                StringWriter var17 = new StringWriter();
                                PrintWriter var18 = new PrintWriter(var17);
                                var3.printStackTrace(var18);
                                this.e = var17.toString();
                                var18.close();
                            }
                        var10000 = this;
                        var10001 = var4;
                        if (var10 != 0) {
                            break label56;
                        }
                        this.b = var4;
                    }
                if (var4 != null) {
                    return;
                }
                var10000 = this;
                var10001 = var1;
            }
        var10000.b = var10001;
    }
    public String a() {
        return this.c;
    }
    public String b() {
        return this.d;
    }
    public String c() {
        return this.b;
    }
    public String d() {
        return this.e;
    }
    public String toString() {
        StringBuilder var1 = new StringBuilder();
        var1.append(this.e());
        var1.append(g[4]);
        var1.append(this.a());
        var1.append(g[1]);
        var1.append(g[3]);
        var1.append(this.c());
        var1.append(g[2]);
        return var1.toString();
    }
    public String e() {
        return this.a;
    }
    public Throwable f() {
        return this.f;
    }
    static {
        g = new String[] {" (", "\n---------------------------------------------\n", "\n---------------------------------------------\n", "Location:\n", " \n---------------------------------------------\n"};
    }
}
