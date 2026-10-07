public class EnhancedTypeCheckingSwitch {
    record Point(int i, int j) {
    }
    enum Color {
        RED, GREEN, BLUE
    }
    static void typeTester(Object obj) {
        switch (obj) {
            case null -> System.out.println("null");
            case String s -> System.out.println("String");
            case Color c -> System.out.println("Color: " + c.toString());
            case Point p -> System.out.println("Record class: " + p.toString());
            case int[] ia -> System.out.println("Array of ints of length" + ia.length);
            default -> System.out.println("Something else");
        }
    }
    public static void main(String[] args) {
        typeTester("test");
        typeTester(Color.BLUE);
        typeTester(new int[] {1, 2, 3, 4});
        typeTester(new Point(7, 8));
        typeTester(new Object());
    }
}
