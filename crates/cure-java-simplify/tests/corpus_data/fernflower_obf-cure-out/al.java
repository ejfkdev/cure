import java.text.DecimalFormat;
import java.util.ArrayList;
import java.util.Date;
import java.util.List;
import java.util.concurrent.TimeUnit;

public class al implements Comparable<al> {
    private final ak a;
    private final List<n<Date, Double>> b = new ArrayList();
    private final List<n<Date, Double>> c = new ArrayList();
    private double d = 0.0;
    private double e = 0.0;
    private double f = 0.0;
    private static final String[] g;
    public al(ak var1) {
        this.a = var1;
    }
    protected static long a(Date param0, Date param1, TimeUnit param2) {}
    protected static <T> T a(List<T> param0) {}
    protected static <T> T b(List<T> param0) {}
    public void a(double param1) {}
    public ak a() {
        return this.a;
    }
    public List<n<Date, Double>> b() {
        return this.b;
    }
    public List<n<Date, Double>> c() {
        return this.c;
    }
    public double d() {
        return this.d;
    }
    public double e() {
        return this.b.isEmpty() ? 0.0 : this.e / (double) this.b.size();
    }
    public double f() {
        return this.c.isEmpty() ? 0.0 : this.f / (double) this.c.size();
    }
    public String toString() {
        StringBuilder var1 = new StringBuilder(this.a.a());
        var1.append(g[2]);
        var1.append(this.a.b());
        if (this.a.c() != null) {
            var1.append(g[1]);
            var1.append(this.a.c());
            var1.append("]");
        }
        var1.append(g[0]);
        var1.append(DecimalFormat.getNumberInstance().format(this.d()));
        var1.append(" ");
        var1.append(DecimalFormat.getNumberInstance().format(this.e()));
        var1.append(" ");
        var1.append(DecimalFormat.getNumberInstance().format(this.f()));
        return var1.toString();
    }
    protected static boolean a(Object param0, Object param1) {}
    public int a(al var1) {
        return var1 == null ? 1 : a(this.a().a(), var1.a().a()) ? this.a().b().compareTo(var1.a().b()) : this.a().a().compareTo(var1.a().a());
    }
    static {
        g = new String[] {": ", " [", " - "};
    }
}
