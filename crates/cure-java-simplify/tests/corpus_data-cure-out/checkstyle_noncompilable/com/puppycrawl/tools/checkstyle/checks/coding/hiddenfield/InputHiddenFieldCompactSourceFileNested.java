String name = "default";
void main() { }

static class StaticNested {
    void process(String name) { }
}

class Inner {
    void process(String name) { } // violation ''name' hides a field'
}
