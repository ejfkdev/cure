package com.puppycrawl.tools.checkstyle.utils.javadocutil;

public class InputJavadocUtilCompactCtorDefComments {
    public record ModifierPath(String value) {
        public ModifierPath {}
    }
    public record BodyCommentOnly(String value) {
        public BodyCommentOnly {
            class Local {
                        }
        }
    }
    public record DanglingReal(String value) {
        public DanglingReal {}
    }
}
