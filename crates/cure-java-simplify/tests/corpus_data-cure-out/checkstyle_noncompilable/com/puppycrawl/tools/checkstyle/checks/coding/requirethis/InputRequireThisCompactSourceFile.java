import java.util.List;

int field = 1;
int derived = field; // violation 'Reference to instance variable 'field' needs "this.".'
int counter = 0;
void increment() {
    field = field + 1;
    // 2 violations above:
    //  'Reference to instance variable 'field' needs "this.".'
    //  'Reference to instance variable 'field' needs "this.".'
    counter = counter + 1;
}

void main() {
    increment(); // violation 'Method call to 'increment' needs "this.".'
}

class Inner {
    int innerField = 2;

    void touch() {
        innerField = innerField + 1;
        // 2 violations above:
        //  'Reference to instance variable 'innerField' needs "this.".'
        //  'Reference to instance variable 'innerField' needs "this.".'
    }
}
