package com.puppycrawl.tools.checkstyle.checks.imports.unusedimports;

import java.util.Optional;
import java.util.function.Function;
import java.util.List;
import java.util.Arrays;
import java.util.function.Predicate;
import java.util.Objects;
import static java.util.Arrays.toString;
import static java.util.Arrays.asList;
import static java.lang.Integer.parseInt;
import static java.util.Collections.emptyList;

public class InputUnusedImportsFromStaticMethodRefExtended {
    private Function<int[], String> arrayToString = Arrays::toString;
    Function<String, Integer> parseIntFunc = Integer::parseInt;
    private final Predicate<List> isListEmpty = List::isEmpty;
    InputUnusedImportsFromStaticMethodRefExtended() {}
    void testMethodRefWithQualifiedName() {
        Optional.empty().map(java.util.Objects::nonNull);
    }
    void testMethodRefWithGenericType() {}
}
