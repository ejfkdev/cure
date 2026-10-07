package com.puppycrawl.tools.checkstyle.grammar.java8;

import java.util.Arrays;
import java.util.List;
import java.util.logging.Logger;

public class InputLambda7 {
    private static final Logger LOG = Logger.getLogger(InputLambda7.class.getName());
    public void doSomething() {
        Arrays.asList(1, 2, 3, 4, 5, 6).forEach((value) -> {
            LOG.info(value.toString());
        });
    }
}
