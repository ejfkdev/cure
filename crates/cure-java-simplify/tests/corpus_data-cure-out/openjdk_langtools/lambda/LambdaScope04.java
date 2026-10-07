class LambdaScope04 {
    interface SAM {
        void m(Object o);
    }
    static SAM field1 = (field1) -> {};
    static SAM field2 = (param) -> {
    Object field2 = null;
};
    SAM field3 = (field3) -> {};
    SAM field4 = (param) -> {
    Object field4 = null;
};
    {
        Object local = null;
        SAM s2 = (param) -> {
            Object local = null;
        };
    }
    static {
        SAM s3 = (field1) -> {
            Object field_2 = null;
        };
    }
    void testLocalInstance() {
        SAM s3 = (field1) -> {
            Object field_2 = null;
        };
    }
    static void testLocalStatic() {
        SAM s3 = (field1) -> {
            Object field_2 = null;
        };
    }
    void testParamInstance(Object local) {
        SAM s3 = (field1) -> {
            Object field_2 = null;
        };
    }
    static void testParamStatic(Object local) {
        SAM s3 = (field1) -> {
            Object field_2 = null;
        };
    }
    void testForInstance() {
        for (int local = 0; local != 0; local++) {
            SAM s3 = (field1) -> {
                Object field_2 = null;
            };
        }
    }
    static void testForStatic(Iterable<Object> elems) {
        for (int local = 0; local != 0; local++) {
            SAM s3 = (field1) -> {
                Object field_2 = null;
            };
        }
    }
    void testForEachInstance(Iterable<Object> elems) {
        for (Object local : elems) {
            SAM s3 = (field1) -> {
                Object field_2 = null;
            };
        }
    }
    static void testForEachStatic(Iterable<Object> elems) {
        for (Object local : elems) {
            SAM s3 = (field1) -> {
                Object field_2 = null;
            };
        }
    }
    void testCatchInstance() {
        try {} catch (Throwable local) {
            SAM s3 = (field1) -> {
                Object field_2 = null;
            };
        }
    }
    static void testCatchStatic(Iterable<Object> elems) {
        try {} catch (Throwable local) {
            SAM s3 = (field1) -> {
                Object field_2 = null;
            };
        }
    }
    void testTWRInstance(AutoCloseable res) {
        try (AutoCloseable local = res) {
            SAM s3 = (field1) -> {
                Object field_2 = null;
            };
        }
    }
    static void testTWRStatic(AutoCloseable res) {
        try (AutoCloseable local = res) {
            SAM s3 = (field1) -> {
                Object field_2 = null;
            };
        }
    }
    void testBlockLocalInstance() {
        {
            SAM s3 = (field1) -> {
                Object field_2 = null;
            };
        }
    }
    static void testBlockLocalStatic() {
        {
            SAM s3 = (field1) -> {
                Object field_2 = null;
            };
        }
    }
    void testSwitchLocalInstance(int i) {
        switch (i) {
            case 0:
                Object local = null;
            default:
                {
                    SAM s3 = (field1) -> {
                        Object field_2 = null;
                    };
                }
        }
    }
    static void testSwitchLocalStatic(int i) {
        switch (i) {
            case 0:
                Object local = null;
            default:
                {
                    SAM s3 = (field1) -> {
                        Object field_2 = null;
                    };
                }
        }
    }
}
