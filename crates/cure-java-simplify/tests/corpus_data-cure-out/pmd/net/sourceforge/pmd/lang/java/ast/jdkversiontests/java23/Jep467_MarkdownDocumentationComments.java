import java.util.Locale;

public class Jep467_MarkdownDocumentationComments {
    public String name(String prefix) {
        return (prefix + "name").toUpperCase(Locale.ROOT);
    }
    @Override
    public String toString() {
        return super.toString();
    }
}
