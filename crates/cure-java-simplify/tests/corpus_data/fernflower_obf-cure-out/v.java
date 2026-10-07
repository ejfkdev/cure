import java.lang.reflect.Field;
import java.util.List;
import java.util.logging.Level;

public class v {
    private static final String[] a;
    public static Object a(Object var0) {
        if (var0 != null) {
            a(var0, var0.getClass());
        }
        return var0;
    }
    private static void a(Object param0, Class<?> param1) {}
    private static void a(Field var0, Object var1) {
        int var3 = y.d;
        Throwable var10000;
        label43:
            {
                if (var3 != 0) {
                    return;
                }
                if (!List.class.isAssignableFrom(var0.getType())) {
                    break label43;
                }
                Throwable var2;
                try {
                    var0.set(var1, t.b(((x) var0.getAnnotation(x.class)).a()));
                    return;
                } catch (Throwable var5) {
                    var2 = var5;
                }
                try {
                    t.a.log(Level.WARNING, var1.getClass() + "." + var0.getName() + a[1] + var2.getMessage(), var2);
                    if (var3 == 0) {
                        return;
                    }
                } catch (Throwable var6) {
                    var10000 = var6;
                    throw var10000;
                }
            }
        try {
            t.a.warning(var1.getClass() + "." + var0.getName() + a[2]);
        } catch (Throwable var4) {
            var10000 = var4;
            throw var10000;
        }
    }
    private static void b(Field var0, Object var1) {
        try {
            var0.set(var1, t.a(var0.getType()));
        } catch (Throwable var3) {
            t.a.log(Level.WARNING, var1.getClass() + "." + var0.getName() + a[0] + var3.getMessage(), var3);
        }
    }
    public <I> I a(Class<I> var1) {
        try {
            return (I) a(var1.newInstance());
        } catch (Throwable var3) {
            throw new IllegalArgumentException(var3);
        }
    }
    static {
        a = new String[] {": ", ": ", ": @InjectList required a java.util.List<E> as field type"};
    }
}
