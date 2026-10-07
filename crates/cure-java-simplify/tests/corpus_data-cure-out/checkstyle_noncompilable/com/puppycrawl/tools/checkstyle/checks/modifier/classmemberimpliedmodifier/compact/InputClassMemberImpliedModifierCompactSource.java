void main() {
    enum LocalColor { RED }
    interface LocalPrintable { }
}
enum Color {
    RED, GREEN, BLUE
}

interface Printable {
    void print();
}

record Point(int x, int y) {
}

@Deprecated enum Status {
    ACTIVE
}

static enum StaticColor {
    RED
}

static interface StaticPrintable {
}

static record StaticPoint(int x, int y) {
}

class OrdinaryClass {
}
