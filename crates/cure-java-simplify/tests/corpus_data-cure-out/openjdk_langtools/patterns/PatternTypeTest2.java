public class PatternTypeTest2 {
    public static void main(String[] args) {
        if (42 instanceof Integer j) {
            System.out.println("It's an Integer");
        } else {
            throw new AssertionError("Broken");
        }
    }
}
