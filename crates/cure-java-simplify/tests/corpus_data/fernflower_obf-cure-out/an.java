import java.lang.management.GarbageCollectorMXBean;
import java.lang.management.ManagementFactory;
import java.lang.management.MemoryMXBean;
import java.lang.management.MemoryPoolMXBean;
import java.lang.management.OperatingSystemMXBean;
import java.lang.management.ThreadMXBean;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

@aa(
   a = {am.class}
)
public class an implements am {
    private OperatingSystemMXBean a = ManagementFactory.getOperatingSystemMXBean();
    private MemoryMXBean b = ManagementFactory.getMemoryMXBean();
    private List<MemoryPoolMXBean> c = ManagementFactory.getMemoryPoolMXBeans();
    private ThreadMXBean d = ManagementFactory.getThreadMXBean();
    private List<GarbageCollectorMXBean> e = ManagementFactory.getGarbageCollectorMXBeans();
    private Map<Long, ao> f = Collections.synchronizedMap(new TreeMap());
    private ak g;
    private List<aw> h;
    private ak i;
    private ak j;
    public static boolean k;
    private static final String[] l;
    public an() {
        this.g = new at(this, l[2], l[1]);
        this.i = new au(this, l[0], l[5]);
        this.j = new av(this, l[4], l[3]);
    }
    private List<aw> a() {
        boolean var4 = k;
        List var10000;
        label45:
            {
                var10000 = this.h;
                if (var4) {
                    return var10000;
                }
                if (var10000 != null) {
                    break label45;
                }
                ArrayList var1 = new ArrayList();
                for (MemoryPoolMXBean var3 : this.c) {
                    var1.add(new aw(this, var3));
                    if (var4) {
                        break label45;
                    }
                    if (var4) {
                        break;
                    }
                }
                this.h = var1;
            }
        return this.h;
    }
    public void a(k<ak> var1) {
        var1.a(this.g);
        var1.a(this.a());
        var1.a(this.i);
        var1.a(this.j);
    }
    public List<ao> b() {
        ArrayList var1 = new ArrayList(this.f.values());
        Collections.sort(var1);
        return var1;
    }
    static MemoryMXBean a(an var0) {
        return var0.b;
    }
    static Map b(an var0) {
        return var0.f;
    }
    static ThreadMXBean c(an var0) {
        return var0.d;
    }
    static OperatingSystemMXBean d(an var0) {
        return var0.a;
    }
    static List e(an var0) {
        return var0.e;
    }
    static {
        l = new String[] {"JVM", "Heap", "JVM", "GC", "JVM", "CPU"};
    }
}
