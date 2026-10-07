void m1() {} // violation 'Total number of methods is 3 (max allowed is 2).'

void m2() {}

class Inner { // violation 'Total number of methods is 3 (max allowed is 2).'
    void x() {}

    void y() {}

    void z() {}
}

void main() {}
