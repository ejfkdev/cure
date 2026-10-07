package com.puppycrawl.tools.checkstyle.checks.naming.abbreviationaswordinname;

public class InputAbbreviationAsWordInNameIgnoreStaticPart2 {
    class StateX {
        int userID;
        int scaleX, scaleY, scaleZ;
        int getScaleX() {
            return this.scaleX;
        }
    }
    @interface Annotation1 {
        String VALUE = "value"; // in @interface this is final/static
    }
    @interface Annotation2 {
        static String VALUE = "value"; // in @interface this is final/static
    }
    @interface Annotation3 {
        final String VALUE = "value"; // in @interface this is final/static
    }
    @interface Annotation4 {
        final static String VALUE = "value"; // in @interface this is final/static
    }
}
