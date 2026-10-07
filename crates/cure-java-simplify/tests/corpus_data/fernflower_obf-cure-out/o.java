import java.math.BigDecimal;
import java.util.regex.Pattern;

public class o {
    private Object a;
    private static final Pattern b;
    private static final String c;
    public boolean a() {
        boolean var10000;
        if (this.a == null) {
            var10000 = true;
            return var10000;
        }
        return false;
    }
    public boolean b() {}
    public boolean c() {
        boolean var10000;
        if (!this.b()) {
            var10000 = true;
            return var10000;
        }
        return false;
    }
    public o a(String... var1) {
        if (this.b()) {
            return this;
        }
        for (String var5 : var1) {
            if (this.a.equals(var5)) {
                return b((Object) null);
            }
        }
        return this;
    }
    public boolean d() {}
    public Object e() {
        return this.a;
    }
    public Object a(Object var1) {
        Object var10000;
        if (this.a == null) {
            var10000 = var1;
            return var10000;
        }
        return this.a;
    }
    public <T> T a(Class<?> param1, T param2) {}
    public <V> V b(Class<V> param1, V param2) {}
    public String f() {
        String var10000;
        if (this.a()) {
            var10000 = null;
            return var10000;
        }
        return this.g();
    }
    public String g() {
        String var10000;
        if (this.a == null) {
            var10000 = "";
            return var10000;
        }
        return this.a.toString();
    }
    public String a(String var1) {
        String var10000;
        if (this.a()) {
            var10000 = var1;
            return var10000;
        }
        return this.g();
    }
    public boolean a(boolean var1) {
        return this.a() ? var1 : this.a instanceof Boolean ? (Boolean) this.a : Boolean.parseBoolean(String.valueOf(this.a));
    }
    public boolean h() {
        return this.a(false);
    }
    public int a(int param1) {}
    public Integer i() {}
    public long a(long param1) {}
    public Long j() {}
    public double a(double param1) {}
    public BigDecimal a(BigDecimal param1) {}
    public static o b(Object var0) {
        o var1 = new o();
        var1.a = var0;
        return var1;
    }
    public String toString() {
        return this.g();
    }
    public <E extends Enum<E>> E a(Class<E> var1) {
        if (this.a == null) {
            return null;
        }
        if (var1.isAssignableFrom(this.a.getClass())) {
            return (E) this.a;
        }
        try {
            return (E) Enum.valueOf(var1, String.valueOf(this.a));
        } catch (Exception var3) {
            return null;
        }
    }
    public String b(int var1) {
        String var2 = this.g();
        if (var2 == null) {
            return null;
        }
        if (var1 < 0) {
            var1 *= -1;
            return var2.length() < var1 ? "" : var2.substring(var1);
        } else {
            return var2.length() < var1 ? var2 : var2.substring(0, var1);
        }
    }
    public String c(int var1) {
        String var2 = this.g();
        if (var2 == null) {
            return null;
        }
        if (var1 < 0) {
            var1 *= -1;
            return var2.length() < var1 ? var2 : var2.substring(0, var2.length() - var1);
        } else {
            return var2.length() < var1 ? var2 : var2.substring(var2.length() - var1);
        }
    }
    public String a(int var1, int var2) {
        String var3 = this.g();
        return var3 == null ? null : var1 > var3.length() ? "" : var3.substring(var1, Math.min(var3.length(), var2));
    }
    public int k() {
        String var1 = this.g();
        return var1 == null ? 0 : var1.length();
    }
    public boolean b(Class<?> param1) {}
    static {
        c = "Cannot convert to: ";
        var var17 = new char[] {'\\', 'd', '+', '(', '\\', '.', '\\', 'd', '+', ')', '?'};
        var var2 = 11;
        var var12 = 11;
        var var39 = new char[] {'\\', 'd', '+', '(', '\\', '.', '\\', 'd', '+', ')', '?'};
        var var46 = 10;
        while (true) {
            char var47 = var39[var46];
            byte var48;
            switch (var2 % 5) {
                case 0:
                    var48 = 74;
                    break;
                case 1:
                    var48 = 64;
                    break;
                case 2:
                    var48 = 76;
                    break;
                case 3:
                    var48 = 110;
                    break;
                default:
                    var48 = 29;
            }
            var39[var46] = (char) (var47 ^ var48);
            ++var2;
            if (var12 == 0) {
                var46 = var12;
                var39 = var17;
            } else {
                if (var12 <= var2) {
                    b = Pattern.compile(new String(var17).intern());
                    return;
                }
                var39 = var17;
                var46 = var2;
            }
        }
    }
}
