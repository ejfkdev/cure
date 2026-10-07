package com.puppycrawl.tools.checkstyle.grammar.java14;

public class InputJava14TextBlocks {
    private static final CharSequence type = "type";
    static {
        String code1 = """
              public void print($type o) {
                  System.out.println(Objects.toString(o));
              }
              """.replace("$type", type);
        String code2 = String.format("""
              public void print(%s o) {
                  System.out.println(Objects.toString(o));
              }
              """, type);
    }
    public String getFormattedText(String parameter) {
        return """
            Some parameter: %s
            """.formatted(parameter);
    }
    public String getIgnoredNewLines() {
        return """
            This is a long test which looks to \
            have a newline but actually does not""";
    }
    public String getEscapedSpaces() {
        return """
            line 1·······
            line 2·······\s
            """;
    }
    void lineTerminators() {}
}
