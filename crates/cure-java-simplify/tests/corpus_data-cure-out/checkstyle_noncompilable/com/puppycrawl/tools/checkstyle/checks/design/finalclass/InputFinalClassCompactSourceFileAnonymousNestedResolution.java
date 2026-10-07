Inner obj = new Inner() { };
void main() { }
class Outer {
    static class Inner {
        private Inner() {}
    }
}

class Inner {
    private Inner() {}
}
