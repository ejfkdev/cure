int[] javaStyleField = new int[1]; // violation 'Array brackets at illegal position.'
void main() {
    System.out.println(javaStyleField.length + new Nested().cStyleField.length);
}
class Nested {
    int cStyleField[] = new int[1];
}
