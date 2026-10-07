package com.puppycrawl.tools.checkstyle.checks.javadoc.javadocmethod;

public class InputJavadocMethodMissingJavadocNoMissingTags {
    int missingReturn(int number) throws ThreadDeath {
        return number;
    }
    int missingParam(int number) throws ThreadDeath {
        return number;
    }
    int missingThrows(int number) throws ThreadDeath {
        return number;
    }
    int missingReturnButInheritDocPresent(int number) throws java.util.NoSuchElementException {
        return number;
    }
    private int missingReturnInTheMiddle(int number) {
        return number;
    }
    private int missingReturnAtTheEnd(int number) {
        return number;
    }
    private int missingReturnAtTheEndFollowedByEmptyLine(int number) {
        return number;
    }
}
