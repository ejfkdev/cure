package com.puppycrawl.tools.checkstyle.checks.javadoc.javadoctype;

import java.util.HashMap;
import java.util.List;

public record InputJavadocTypeRecordComponents2(String value) {
}

record MyRecord1(String myString, Integer myInt) {
}

record MyRecord2(HashMap<String, String> myHashMap) {
}

record MyRecord3<X>() {
}

record MyRecord4() {
}

record MyRecord5<X>() {
}

record MyRecord6<X>(String myString, int myInt) {
}

record MyRecord7(List<String>myList) {
}

record MyRecord8<X, T>(String X) {
}

record MyRecord9<X, T>(String myString, int myInt) {
}
