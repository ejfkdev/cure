void main() { }

class Outer {
    class Nested { }

    // violation below 'Init blocks, constructors, fields and methods should be before inner types'
    void process() { }
}
