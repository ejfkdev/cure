public class InputJava7Multicatch {
    public static void main() {
        try {} catch (@SuppressWarnings("all") FileNotFoundException | CustomException e) {}
    }
}
