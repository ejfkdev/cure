package com.puppycrawl.tools.checkstyle.checks.coding.textblockgooglestyleformatting;

public class InputTextBlockGoogleStyleFormatting7 {
    public void testMethod1() {
        char[] channelNames = getVi(new ObjectString("""
                                </doc>
                                """)).toCharArray();
        String ctx = getTestAppContext("""
                        <bean id='docBuilderFactory'
                        """ + getVi(new ObjectString("")) + """
                            <si-xml:xpath-splitter id='splitter'
                        """);
    }
    public void testMethod2(Object config) {
        try {
            Object o1 = new Object();
        } catch (IllegalArgumentException e) {
            throw new IllegalArgumentException("""
                  you may run into rate limiting issues with your IdP""", e);
        }
        String v0 = "345";
        String v1 = switch (v0) {
            case "2":
                yield """
                                       dsfdsf
                                       """;
            case "3":
                yield
                                      """
                                      jkdf
                                      """;
            default:
                yield "12";
        };
    }
    public String testMethod3(String s1) {
        String s2 = s1.isBlank() ? """
                Mode 1
                """ : s1.equals("s1") ? """
                Mode 2
                """ : """
                Default Mode
                """;
        return switch (s1) {
            case "1" -> """
                        jk
                        """;
            case "2" -> """
                method
                """;
            default -> "?";
        };
    }
    public String getVi(ObjectString s1) {
        return s1 + "";
    }
    public String getTestAppContext(String s1) {
        return s1 + "";
    }
    class ObjectString {
        public ObjectString(String s1) {}
    }
    class ComponentModification extends ObjectString {
        public ComponentModification(String n1) {
            super("""
                    Component %s was modified during phase with priority %s by %s.
                    """);
        }
    }
}
