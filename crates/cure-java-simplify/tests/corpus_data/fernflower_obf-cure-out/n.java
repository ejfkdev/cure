import java.util.ArrayList;
import java.util.Collection;
import java.util.HashMap;
import java.util.Iterator;
import java.util.List;
import java.util.Map;

public class n<F, S> {
    private F a;
    private S b;
    public static boolean c;
    private static final String d;
    public n() {}
    public n(F var1) {
        this.a = var1;
    }
    public n(F var1, S var2) {
        this.a = var1;
        this.b = var2;
    }
    public F a() {
        return this.a;
    }
    public void a(F var1) {
        this.a = var1;
    }
    public S b() {
        return this.b;
    }
    public void b(S var1) {
        this.b = var1;
    }
    public boolean equals(Object param1) {}
    private boolean a(Object param1, Object param2) {}
    public String toString() {
        return this.a + d + this.b;
    }
    public int hashCode() {
        Object var10000;
        label28:
            {
                if (this.a == null) {
                    var10000 = "";
                    break label28;
                }
                var10000 = this.a;
            }
        Object var10001;
        var3 = var10000.hashCode() / 2;
        if (this.b == null) {
            var10001 = "";
            return var3 + var10001.hashCode() / 2;
        }
        return var3 + this.b.hashCode() / 2;
    }
    public static <T extends n<K, V>, K, V> List<K> a(Collection<T> var0) {
        ArrayList var1 = new ArrayList(var0.size());
        for (n var3 : var0) {
            var1.add(var3.a());
        }
        return var1;
    }
    public static <T extends n<K, V>, K, V> List<V> b(Collection<T> param0) {}
    public static <K, V> List<n<K, V>> a(Map<K, V> var0) {
        boolean var4 = c;
        ArrayList var1 = new ArrayList(var0.size());
        Iterator var2 = var0.entrySet().iterator();
        ArrayList var10000;
        while (true) {
            if (var2.hasNext()) {
                Map.Entry var3 = (Map.Entry) var2.next();
                var10000 = var1;
                if (var4) {
                    break;
                }
                var1.add(new n(var3.getKey(), var3.getValue()));
                if (!var4) {
                    continue;
                }
                int var5 = ap.c;
                ++var5;
                ap.c = var5;
            }
            var10000 = var1;
            break;
        }
        return var10000;
    }
    public static <K, V> Map<K, V> c(Collection<n<K, V>> var0) {
        HashMap var1 = new HashMap();
        for (n var3 : var0) {
            var1.put(var3.a(), var3.b());
        }
        return var1;
    }
    static {
        d = ": ";
    }
}
