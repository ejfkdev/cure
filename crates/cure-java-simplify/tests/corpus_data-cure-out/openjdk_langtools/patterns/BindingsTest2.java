public class BindingsTest2 {
    public static boolean Ktrue() {
        return true;
    }
    public static void meth() {
        Object o1 = "hello";
        Object o2 = 42;
        if (Ktrue() ? o2 instanceof Integer x : o2 instanceof String x) {
            x.intValue();
        }
        if (Ktrue() ? o2 instanceof Integer x : true) {
            x.intValue();
        }
        s.length();
        if (o1 instanceof String s || o2 instanceof Integer s) {
            s.length();
        }
        if (o1 instanceof String s || o2 instanceof Integer i) {
            s.length();
        }
        s.length();
        if (!(o1 instanceof String s) ? o2 instanceof Integer s : true) {
            s.length();
        }
        if (!(o1 instanceof String s) ? o2 instanceof Integer i : true) {
            s.length();
            i.intValue();
        }
        if (!(o1 instanceof String s) ? o1 instanceof String s2 : true) {
            s.length();
            s2.length();
        }
        if (!(Ktrue() ? !(o2 instanceof Integer x) : !(o1 instanceof String x))) {
            x.intValue();
        }
        if (!(Ktrue() ? !(o2 instanceof Integer x) : !(o1 instanceof String s))) {
            x.intValue();
        }
        if (!(Ktrue() ? !(o2 instanceof Integer x) : !(o2 instanceof Integer x1))) {
            x.intValue();
            x1.intValue();
        }
        if (!(Ktrue() && !(o2 instanceof Integer x))) {
            x.intValue();
        }
        if (!(o1 instanceof String s || !(o2 instanceof Integer s))) {
            s.length();
        }
        if (!(o1 instanceof String s || !(o2 instanceof Integer i))) {
            s.length();
            i.intValue();
        }
        if (!(o1 instanceof String s || !(o2 instanceof String s1))) {
            s.length();
            s1.length();
        }
        if (!(!(o1 instanceof String s) ? !(o1 instanceof String s1) : true)) {
            s.length();
            s1.length();
        }
        if (!(!(o1 instanceof String s) ? !(o2 instanceof Integer s) : true)) {
            s.length();
        }
        if (!(!(o1 instanceof String s) ? !(o2 instanceof Integer i) : true)) {
            s.length();
            i.intValue();
        }
        if (o1 instanceof String s ? false : s.length() > 0) {
            System.out.println("done");
        }
        if (o1 instanceof String s ? false : s.intValue != 0) {
            System.out.println("done");
        }
        if (!(o1 instanceof String s) && s.length() > 0) {
            System.out.println("done");
        }
        if (!(o1 instanceof String s) && s.intValue > 0) {
            System.out.println("done");
        }
        while (!(o1 instanceof String s)) {
            break;
        }
        s.length();
        while (!(o1 instanceof String s)) {}
        s.length();
        while (!(o1 instanceof String s)) {
            while (true) ;
            break;
        }
        s.length();
        for (; !(o1 instanceof String s); ) {
            break;
        }
        s.length();
        for (; !(o1 instanceof String s); ) {}
        s.length();
        for (; !(o1 instanceof String s); ) {
            while (true) ;
            break;
        }
        s.length();
        do {
            break;
        } while (!(o1 instanceof String s));
        s.length();
        do {} while (!(o1 instanceof String s));
        s.length();
        do {
            while (true) ;
            break;
        } while (!(o1 instanceof String s));
        s.length();
        L:
            while (!(o1 instanceof String s)) {
                break L;
            }
        s.length();
        L:
            for (; !(o1 instanceof String s); ) {
                break L;
            }
        s.length();
        L:
            do {
                break L;
            } while (!(o1 instanceof String s));
        s.length();
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
        L:
            {
                do {
                    break L;
                } while (!(o1 instanceof String s));
                s.length();
            }
        if (o1 instanceof String s) {
            s = "";
        }
        LBL1:
            LBL2:
                if (!(o1 instanceof String s)) {
                    break LBL1;
                }
        System.err.println(s);
        LBL1:
            LBL2:
                if (!(o1 instanceof String s)) {
                    break LBL2;
                }
        System.err.println(s);
        LBL1:
            LBL2:
                if (!(o1 instanceof String s)) {
                    break LBL1;
                }
        System.err.println(s);
        LBL1:
            LBL2:
                if (!(o1 instanceof String s)) {
                    break LBL2;
                }
        System.err.println(s);
        switch (0) {
            case 0:
                if (!(o1 instanceof String s)) {
                    break;
                }
        }
        s.length();
        {
            int j = 0;
            L:
                while (j++ < 2) 
                    if (!(o1 instanceof String s)) {
                        break L;
                    }
            s.length();
        }
        {
            int j = 0;
            L:
                for (; j++ < 2; ) 
                    if (!(o1 instanceof String s)) {
                        break L;
                    }
            s.length();
        }
    }
}
