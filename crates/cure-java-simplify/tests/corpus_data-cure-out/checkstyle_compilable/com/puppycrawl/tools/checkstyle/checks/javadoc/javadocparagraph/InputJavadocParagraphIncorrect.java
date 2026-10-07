package com.puppycrawl.tools.checkstyle.checks.javadoc.javadocparagraph;

class InputJavadocParagraphIncorrect {
    public static final byte NUL = 0;
    boolean emulated() {
        return false;
    }
    class InnerInputJavadocParagraphIncorrect {
        public static final byte NUL = 0;
        boolean emulated() {
            return false;
        }
    }
    InnerInputJavadocParagraphIncorrect anon = new InnerInputJavadocParagraphIncorrect() {
        // violation 2 lines below 'Redundant <p> tag.'
            /**
         * <p>Some summary.
         *
         * Some paragraph.
         *
         * @since 8.0
         */
        // violation 5 lines above 'Empty line should be followed by <p> tag on the next line.'
        public static final byte NUL = 0;
        // violation 3 lines below 'tag should be preceded with an empty line.'
        // violation 4 lines below 'tag should be placed immediately before the first word'
        /**
         *   Some summary.<p>
         *
         *  <p>  Some paragraph.
         *
         * @see <a href="example.com">
         *     Documentation about <p> GWT emulated source</a>
         */
        boolean emulated() {return false;}

        // violation 3 lines below 'Empty line should be followed by <p> tag on the next line.'
        /**
         * Some Summary.
         *
         *
         * Some paragraph.
         */
        // violation 3 lines above 'Empty line should be followed by <p> tag on the next line.'
         void doubleNewline() {}
    };
}
