int staticVar = 1;
int pubInstance = 2;
int privInstance = 3;
int pubInstance2 = 4; // violation 'Variable access definition in wrong order'
void method1() {
}

int afterMethod = 5; // violation 'Instance variable definition in wrong order'
int afterMethodStatic = 6; // violation 'Static variable definition in wrong order'
void main() {
}
class Inner {
    void innerMethod() {}
    int innerField = 7;
}
