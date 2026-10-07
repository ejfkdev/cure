import module java.base;

int java = 1;
void increment() {
    java = java + 1;
    // 2 violations above:
    //  'Reference to instance variable 'java' needs "this.".'
    //  'Reference to instance variable 'java' needs "this.".'
}

void main() {
}
