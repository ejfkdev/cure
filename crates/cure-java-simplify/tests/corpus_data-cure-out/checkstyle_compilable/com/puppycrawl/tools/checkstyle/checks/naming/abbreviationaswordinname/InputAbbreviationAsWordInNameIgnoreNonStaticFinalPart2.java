package com.puppycrawl.tools.checkstyle.checks.naming.abbreviationaswordinname;

public class InputAbbreviationAsWordInNameIgnoreNonStaticFinalPart2 {
    class StateX {
        int userID;
        int scaleX, scaleY, scaleZ;
        int getScaleX() {
            return this.scaleX;
        }
    }
    @interface Annotation1 {
        String VALUELONG = "value";
    }
    @interface Annotation2 {
        static String VALUELONG = "value";
    }
    @interface Annotation3 {
        final String VALUELONG = "value";
    }
    @interface Annotation4 {
        final static String VALUELONG = "value";
    }
    final class InnerClassOneVIOLATION {
    }
    static class InnerClassTwoVIOLATION {
    }
    static final class InnerClassThreeVIOLATION {
    }
}
