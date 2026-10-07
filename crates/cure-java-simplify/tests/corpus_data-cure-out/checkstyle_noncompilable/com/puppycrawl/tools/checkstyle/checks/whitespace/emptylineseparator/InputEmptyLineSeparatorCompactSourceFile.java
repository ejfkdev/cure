import java.util.List;

int field1 = 1; // violation 'should be separated from previous line'
int field2 = 2; // violation 'should be separated from previous line'
int field3 = 3;
void m1() {
}
void m2() { // violation 'should be separated from previous line'
}

void main() {
    List<Integer> list = List.of(field1, field2, field3);
}

class Nested {
}
