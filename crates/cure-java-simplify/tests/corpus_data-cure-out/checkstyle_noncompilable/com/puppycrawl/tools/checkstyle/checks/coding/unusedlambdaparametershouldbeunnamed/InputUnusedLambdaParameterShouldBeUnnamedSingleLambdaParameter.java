package com.puppycrawl.tools.checkstyle.checks.coding.unusedlambdaparametershouldbeunnamed;

import java.util.List;
import java.util.function.Function;

public class InputUnusedLambdaParameterShouldBeUnnamedSingleLambdaParameter {
    String character;
    void testUnused() {
        List<String> list = List.of("a", "b", "c");
        List<String> Xs = list.stream().map((character) -> {
            return "x";
        }).toList();
        Xs = list.stream().map((character) -> "x").toList();
        Xs = list.stream().map((character) -> {
            return this.character;
        }).toList();
        Xs = list.stream().map((character) -> this.character).toList();
        Xs = list.stream().map((Character) -> {
            return 'x'.toString();
        }).toList();
        Xs = list.stream().map((character) -> {
            return "x";
        }).toList();
        Xs = list.stream().map((C) -> {
            Object a = new C();
            return "x";
        }).toList();
        Function<String, String> function = (s) -> "x";
        Xs = list.stream().map((C) -> {
            C = "a";
            return "x";
        }).toList();
    }
    void testUsed() {
        List<String> list = List.of("a", "b", "c");
        List<String> Xs = list.stream().map((character) -> {
            return "x" + character;
        }).toList();
        Xs = list.stream().map((character) -> "x" + character).toList();
        Xs = list.stream().map((character) -> {
            return this.character + character;
        }).toList();
        Xs = list.stream().map((character) -> this.character + character).toList();
        Xs = list.stream().map((Character) -> {
            return Character.charAt(0).toString();
        }).toList();
        Xs = list.stream().map((character) -> {
            return character.toString();
        }).toList();
        Xs = list.stream().map((C) -> {
            Object a = new C(C);
            return "x";
        }).toList();
        Function<String, String> function = (s) -> s.trim().toLowerCase();
    }
    void testUnnamed() {
        List<String> list = List.of("a", "b", "c");
        List<String> Xs = list.stream().map((_) -> {
            return "x";
        }).toList();
        Xs = list.stream().map((_) -> {
            return "x";
        }).toList();
    }
    void testSwitchRule(int x) {
        switch (x) {
            case 1 -> {
                System.out.println("1");
            }
            case 2 -> System.out.println("2");
            default -> System.out.println("default");
        }
    }
    String character(String c) {
        return c;
    }
    class C {
        C() {}
        C(String c) {
            character = c;
        }
    }
}
