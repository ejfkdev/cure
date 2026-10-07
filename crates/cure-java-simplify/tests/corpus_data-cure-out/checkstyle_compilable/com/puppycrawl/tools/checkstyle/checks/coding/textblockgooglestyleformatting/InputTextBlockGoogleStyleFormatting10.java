package com.puppycrawl.tools.checkstyle.checks.coding.textblockgooglestyleformatting;

public class InputTextBlockGoogleStyleFormatting10 {
    public static void textFun() {
        String simpleScript = """
     Less Indentation than expected
     Violation is expected here.
            """;
        String simpleScript4 = simpleScript + (simpleScript + "                More indentation than the quotes, ok.\n                     this is simple script\n").endsWith("""
                this is a simple sentence
                    this is a simple sentence
                       this is a simple sentence
                """);
        String simpleScript10 = """
                """;
    }
}
