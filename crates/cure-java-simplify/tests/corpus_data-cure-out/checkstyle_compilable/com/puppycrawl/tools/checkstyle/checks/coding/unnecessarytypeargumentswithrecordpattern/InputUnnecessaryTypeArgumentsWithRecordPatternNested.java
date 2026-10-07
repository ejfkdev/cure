package com.puppycrawl.tools.checkstyle.checks.coding.unnecessarytypeargumentswithrecordpattern;

public class InputUnnecessaryTypeArgumentsWithRecordPatternNested {
    record Box<T>(T t) {
    }
    void testInstanceof(Box<Box<String>> box) {
        if (box instanceof Box<Box<String>>(Box<String>(var s))) {
            System.out.println(s);
        }
        if (box instanceof Box<?>(Box<?> (var s))) {
            System.out.println(s);
        }
        if (box instanceof Box(Box(var s))) {
            System.out.println(s);
        }
    }
    void testSwitchViolation(Box<Box<String>> box) {
        switch (box) {
            case Box<Box<String>>(Box<String>(var s)) -> System.out.println(s);
            default -> {}
        }
    }
    void testSwitchOk(Box<Box<String>> box) {
        switch (box) {
            case Box(Box(var s)) -> System.out.println(s);
            default -> {}
        }
    }
}
