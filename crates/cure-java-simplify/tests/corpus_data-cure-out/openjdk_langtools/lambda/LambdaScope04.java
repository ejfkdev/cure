class LambdaScope04 {
    interface SAM {
        void m(Object o);
    }
    static SAM field1 = (field1) -> {};
    static SAM field2 = (param) -> {};
    SAM field3 = (field3) -> {};
    SAM field4 = (param) -> {};
    {}
    static {}
    void testLocalInstance() {}
    static void testLocalStatic() {}
    void testParamInstance(Object local) {}
    static void testParamStatic(Object local) {}
    void testForInstance() {
        for (int local = 0; local != 0; local++) {}
    }
    static void testForStatic(Iterable<Object> elems) {
        for (int local = 0; local != 0; local++) {}
    }
    void testForEachInstance(Iterable<Object> elems) {
        for (Object local : elems) {}
    }
    static void testForEachStatic(Iterable<Object> elems) {
        for (Object local : elems) {}
    }
    void testCatchInstance() {
        try {} catch (Throwable local) {}
    }
    static void testCatchStatic(Iterable<Object> elems) {
        try {} catch (Throwable local) {}
    }
    void testTWRInstance(AutoCloseable res) {
        try (AutoCloseable local = res) {}
    }
    static void testTWRStatic(AutoCloseable res) {
        try (AutoCloseable local = res) {}
    }
    void testBlockLocalInstance() {}
    static void testBlockLocalStatic() {}
    void testSwitchLocalInstance(int i) {
        switch (i) {
            case 0:
                Object local = null;
            default:
                {}
        }
    }
    static void testSwitchLocalStatic(int i) {
        switch (i) {
            case 0:
                Object local = null;
            default:
                {}
        }
    }
}
