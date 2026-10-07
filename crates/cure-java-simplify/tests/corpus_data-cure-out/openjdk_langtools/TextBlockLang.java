public class TextBlockLang {
    public static void main(String... args) {
        test1();
        test2();
        test3();
    }
    static void test1() {
        EQ("""
           """, "");
        EQ("""
            abc
            """, "abc\n");
        EQ("""

           """, "\n");
        EQ("""
            "
            """, "\"\n");
        EQ("""
            ""
            """, "\"\"\n");
        EQ("""
           \"""
           """, "\"\"\"\n");
        EQ("""
           "\""
           """, "\"\"\"\n");
        EQ("""
           ""\"
           """, "\"\"\"\n");
        EQ("""
            \r
            """, "\r\n");
        EQ("""
            •
            """, "•\n");
        EQ("""
            •
            """, "•\n");
        LENGTH("""
            abc
            """, 4);
    }
    static void test2() {
        EQ(" ", " ");
        EQ("""
           \s
           """, " \n");
    }
    static void test3() {
        EQ("""
           abc \
           """, "abc ");
        EQ("\\\n".translateEscapes(), "");
        EQ("\\\r\n".translateEscapes(), "");
        EQ("\\\r".translateEscapes(), "");
    }
    static void LENGTH(String string, int length) {
        if (string == null || string.length() != length) {
            System.err.println("Failed LENGTH");
            System.err.println(string + " " + length);
            throw new RuntimeException("Failed LENGTH");
        }
    }
    static void EQ(String input, String expected) {
        if (input == null || expected == null || !expected.equals(input)) {
            System.err.println("Failed EQ");
            System.err.println();
            System.err.println("Input:");
            System.err.println(input.replaceAll(" ", "."));
            System.err.println();
            System.err.println("Expected:");
            System.err.println(expected.replaceAll(" ", "."));
            throw new RuntimeException();
        }
    }
}
