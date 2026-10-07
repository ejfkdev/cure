import java.util.Iterator;
import java.util.TimerTask;

class ba extends TimerTask {
    final ae a;
    private static final String[] b;
    private ba(ae var1) {
        this.a = var1;
    }
    public void run() {
        boolean var4 = ae.e;
        Iterator var1 = ae.a(this.a).iterator();
        while (true) {
            if (var1.hasNext()) {
                ac var2 = (ac) var1.next();
                label23:
                    {
                        try {
                            var2.a();
                        } catch (Exception var6) {
                            r.a(b[0]).a(b[1], var2.getClass().getName()).a();
                            break label23;
                        }
                        if (var4) {
                            break;
                        }
                    }
                if (!var4) {
                    continue;
                }
                int var5 = ap.c;
                ++var5;
                ap.c = var5;
            }
            ae.a(this.a, System.currentTimeMillis());
            break;
        }
    }
    ba(ae var1, af var2) {
        this(var1);
    }
    static {
        b = new String[] {"EveryMinuteTaskFailed", "class"};
    }
}
