public class BindingsTest1 {
    public static boolean Ktrue() {
        return true;
    }
    public static void main(String[] args) {
        Object o1 = "hello";
        Integer i = 42;
        Object o2 = i;
        Object o3 = "there";
        if (o1 instanceof String s) {
            s.length();
        }
        if (o1 instanceof String s && o2 instanceof Integer in) {
            s.length();
            in.intValue();
        }
        if (o1 instanceof String s && s.length() > 0) {
            System.out.print("done");
        }
        if (!(!(o1 instanceof String s) || !(o3 instanceof Integer in))) {
            s.length();
            i.intValue();
        }
        if (!(o1 instanceof String s) || s.length() > 0) {
            System.out.println("done");
        }
        if (o1 instanceof String s && s.length() > 0) {
            System.out.println("done");
        }
        if (!(o1 instanceof String s) ? false : s.length() > 0) {
            System.out.println("done");
        }
        if (!(!(o1 instanceof String s) || !(o3 instanceof Integer in))) {
            s.length();
            i.intValue();
        }
        if (o1 instanceof String s) {
            s.length();
        }
        L1:
            {
                if (o1 instanceof String s) {
                    s.length();
                } else {
                    break L1;
                }
                s.length();
            }
        L2:
            {
                if (!(o1 instanceof String s)) {
                    break L2;
                } else {
                    s.length();
                }
                s.length();
            }
        L4:
            {
                if (!(o1 instanceof String s)) {
                    break L4;
                }
                s.length();
            }
        while (!(o1 instanceof String s)) {}
        s.length();
        L5:
            {
                while (!(o1 instanceof String s)) {}
                s.length();
            }
        L6:
            for (; !(o1 instanceof String s); ) {}
        s.length();
        L7:
            do {} while (!(o1 instanceof String s));
        s.length();
        while (!(o1 instanceof String s)) {
            L8:
                break L8;
        }
        s.length();
        for (; !(o1 instanceof String s); ) {
            L9:
                break L9;
        }
        s.length();
        do {
            L10:
                break L10;
        } while (!(o1 instanceof String s));
        s.length();
        if (o1 instanceof String s) {
            new Runnable() {
                @Override
                public void run() {
                    s.length();
                }
            }.run();
            Runnable r2 = () -> {
                s.length();
            };
            r2.run();
            String s2 = s;
        }
        if (o1 instanceof String s) {
            new Runnable() {
                @Override
                public void run() {
                    s.length();
                }
            }.run();
            Runnable r2 = () -> {
                s.length();
            };
            r2.run();
            String s2 = s;
        }
        boolean result = o1 instanceof String a1 ? o1 instanceof String a2 : !(o1 instanceof String a3);
        boolean result2 = o1 instanceof String a1 ? o1 instanceof String a2 : !switch (0) {
            default -> false;
        };
        if (!((VoidPredicate) (() -> o1 instanceof String str && !str.isEmpty())).get()) {
            throw new AssertionError();
        }
        if (!((VoidPredicate) (() -> o1 instanceof String str && !str.isEmpty())).get()) {
            throw new AssertionError();
        }
        if (!new VoidPredicate() { public boolean get() { return o1 instanceof String str && !str.isEmpty();} }.get()) {
            throw new AssertionError();
        }
        if (!switch (i) {
            default:
                if (!(o1 instanceof String str)) {
                    yield false;
                }
                if (str.isEmpty()) {
                    yield true;
                }
                yield true;
        }) {
            throw new AssertionError();
        }
        L:
            {
                while (!(o1 instanceof String s)) {
                    break L;
                }
                s.length();
            }
        L:
            {
                for (; !(o1 instanceof String s); ) {
                    break L;
                }
                s.length();
            }
        {
            int j = 0;
            L:
                while (j++ < 2) 
                    if (!(o1 instanceof String s)) {
                        break L;
                    }
        }
        {
            int j = 0;
            L:
                for (; j++ < 2; ) 
                    if (!(o1 instanceof String s)) {
                        break L;
                    }
        }
        L:
            if (!(o1 instanceof String s)) {
                Runnable r = () -> {
                    NESTED:
                        {
                            if (!(o1 instanceof String n)) {
                                break NESTED;
                            }
                            n.length();
                        }
                };
                break L;
            }
        switch (0) {
            case 0:
                if (!(o1 instanceof String s)) {
                    break;
                }
                s.length();
        }
        if (!(invokeOnce("") instanceof String s)) {
            throw new AssertionError();
        }
        System.out.println("BindingsTest1 complete");
    }
    interface VoidPredicate {
        public boolean get();
    }
    static boolean id(boolean b) {
        return b;
    }
    private static boolean invoked;
    static Object invokeOnce(Object val) {
        if (invoked) {
            throw new IllegalStateException();
        } else {
            invoked = true;
            return val;
        }
    }
}
