package com.puppycrawl.tools.checkstyle.checks.naming.abbreviationaswordinname;

abstract class InputAbbreviationAsWordInNameType5Part2 {
}

class StateX5 {
    int userID;
    int scaleX, scaleY, scaleZ;
    int getScaleX() {
        return this.scaleX;
    }
}

@interface Annotation15 {
    String VALUE = "value"; // in @interface this is final/static
}

@interface Annotation25 {
    static String VALUE = "value"; // in @interface this is final/static
}

@interface Annotation35 {
    final String VALUE = "value"; // in @interface this is final/static
}

@interface Annotation45 {
    final static String VALUE = "value"; // in @interface this is final/static
}
