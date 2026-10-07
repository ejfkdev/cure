class BadReturn {
    interface SAM {
        Comparable<?> m();
    }
    static void testNeg1() {
        SAM s = () -> {
            return "";
        };
    }
    static void testNeg2() {
        SAM s = () -> {
            return System.out.println("");
        };
    }
    static void testPos() {
        SAM s = () -> {
            return true;
        };
    }
}
