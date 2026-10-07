package com.puppycrawl.tools.checkstyle.checks.imports.unnecessaryfullyqualifiedtype;

import java.util.Optional;

public class InputUnnecessaryFullyQualifiedType {
    private java.util.Map<Boolean, Optional<String>> choiceMap;
    private java.lang.String name;
    private Optional<String> present;
    private Optional<String> alsoPresent;
    void create() {
        Object map = new java.util.HashMap<String, String>();
    }
    void method() throws java.io.IOException {
        boolean isCollection = (java.util.Set<String>) null instanceof java.util.Collection;
    }
    void notAType() {
        java.lang.System.out.println("ok");
        java.util.Collections.emptyList();
    }
    java.util.Map.Entry<String, String> nestedTypeReferenceIsNotReported() {
        return null;
    }
}
