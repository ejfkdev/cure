public class NullQualifiedNew {
    class Nested {
        int j;
        Nested(int i) {
            j = i;
        }
    }
    public static void main(String[] args) {
        null.new Nested(6);
    }
}
