class TargetType36 {
    interface SAM {
        int m(int i, int j);
    }
    void test() {
        SAM s1 = (SAM) ((a,b) -> a + b);
        SAM s2 = (SAM) ((a,b) -> a + b);
        SAM s3 = (SAM) ((a,b) -> a + b);
    }
}
