class InputEmptyLineSeparatorCompactNoPackage {
    void top() {}
    void nemcp2(int eights) {
        top();
    }
    void nemcp1() {
        nemcp2(888);
    }
    void emcp2() {
        nemcp1();
    }
    void emcp1(int myArg) {
        emcp2();
    }
    void bottom() {
        emcp1(56);
    }
    static void stnemcp() {
        new InputEmptyLineSeparatorCompactNoPackage().bottom();
    }
    static void stemcp() {
        stnemcp();
    }
}
