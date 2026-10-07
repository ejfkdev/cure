class BadBreakContinue {
    static interface SAM {
        void m();
    }
    SAM s1 = () -> {
    break;
};
    SAM s2 = () -> {
    continue;
};
    SAM s3 = () -> {};
    void testLabelled() {
        loop:
            while (true) {}
    }
    void testNonLabelled() {
        while (true) {}
    }
}
