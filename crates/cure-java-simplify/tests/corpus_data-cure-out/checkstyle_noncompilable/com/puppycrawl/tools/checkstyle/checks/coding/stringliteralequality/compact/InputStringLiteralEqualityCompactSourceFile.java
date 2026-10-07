void main() {
    String name = "test";
    // violation below 'Literal Strings should be compared using equals(), not '=='.'
    if (name == "Lars") {
        System.out.println("matched");
    }
}
