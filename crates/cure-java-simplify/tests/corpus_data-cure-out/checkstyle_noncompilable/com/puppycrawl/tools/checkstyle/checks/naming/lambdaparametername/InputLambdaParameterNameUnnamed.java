package com.puppycrawl.tools.checkstyle.checks.naming.lambdaparametername;

import java.util.Arrays;
import java.util.List;
import java.util.stream.Collectors;

public class InputLambdaParameterNameUnnamed {
    void method(Object o) {
        List<Integer> numbers = Arrays.asList(1, 2, 3);
        numbers = numbers.stream().map((_) -> 0).toList();
        List<String> strings = Arrays.asList("a", "b", "c");
        System.out.println(strings.stream().collect(Collectors.toMap(String::toUpperCase, (_) -> "NODATA")));
        System.out.println(strings.stream().collect(Collectors.toMap(String::toUpperCase, (__) -> "NODATA")));
        System.out.println(strings.stream().collect(Collectors.toMap(String::toUpperCase, (_BAD) -> "NODATA")));
        System.out.println(strings.stream().collect(Collectors.toMap(String::toUpperCase, (BAD_) -> "NODATA")));
        switch (o) {
            case Integer __ -> {}
            case String _ -> {}
            default -> {}
        }
    }
}
