class BadAccess03 {
    void test() {
        int k;
        Runnable r = () -> {
            k = 2;
        };
    }
}
