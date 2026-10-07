package com.puppycrawl.tools.checkstyle.checks.javadoc.summaryjavadoc;

class InputSummaryJavadocIncorrect5 {
    InputSummaryJavadocIncorrect5 anon = new InputSummaryJavadocIncorrect5() {

        /**
         * Some Javadoc.
         */
        public static final byte NUL = 0;

        /**
         * Some Javadoc.
         */
        void emulated(String s) {}

        /**
         * As of JDK 1.1, replaced by {@link #setBounds(int,int,int,int)}.
         */
        void foo3() {}
    // violation below 'Summary javadoc is missing.'
        /**
         * @throws Exception if a problem occurs
         */
        void foo4() throws Exception {}

        /** An especially short bit of Javadoc. */
        void foo5() {}

        /**
         * An especially short bit of Javadoc.
         */
        void foo6() {}
    // violation below 'First sentence .* missing an ending period.'
         /**
          * mm{@inheritDoc}
          */
         void foo7() {}
    // violation below 'Summary javadoc is missing.'
         /**
          * {@link #setBounds(int,int,int,int)}
          */
         void foo8() {}
    // violation below 'Summary javadoc is missing.'
         /**
          *
          */
         void foo10() {}
    };
    void foo7() {}
    <T> T foo8(T t) {
        return null;
    }
    String[] foo9() {
        return null;
    }
}
