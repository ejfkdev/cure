import java.util.function.Supplier;

int field = 1;
int constField = 2;
Supplier<Integer> supplier = () -> {
    int lambdaLocal = 5; // violation 'Variable 'lambdaLocal' should be declared final'
    return lambdaLocal;
};
int categorized = switch (field) {
    case 1 -> {
        int branchLocal = 10; // violation 'Variable 'branchLocal' should be declared final'
        yield branchLocal;
    }
    default -> 0;
};
void main() {
    int local = 2; // violation 'Variable 'local' should be declared final'
    int reassigned = 0;
    reassigned = 9;
    System.out.println(field + constField + local + categorized + reassigned);
    supplier.get();
}
