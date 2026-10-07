int outerField = 1;
void main() {
}
class Inner {
    int innerField = 2;
    void useOuter() {
        outerField = 2;
    }
    void useInner() {
        innerField = 3;
    }
}
