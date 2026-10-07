class EffectivelyFinal01 {
    interface SAM {
        Integer m(Integer i);
    }
    void test(Integer nefPar) {
        SAM s = (Integer h) -> {
            return 0 + h + nefPar;
        };
        nefPar++;
    }
}
