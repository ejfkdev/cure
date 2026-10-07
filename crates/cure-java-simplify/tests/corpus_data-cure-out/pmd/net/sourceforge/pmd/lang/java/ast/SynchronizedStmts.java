class Sync {
    public static void getInstance() {
        synchronized (0) {
            return;
        }
    }
}
