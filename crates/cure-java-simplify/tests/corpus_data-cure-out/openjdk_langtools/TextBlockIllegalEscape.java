public class TextBlockIllegalEscape {
    static void test() {
        EQ("""
           \!
           """, "");
    }
}
