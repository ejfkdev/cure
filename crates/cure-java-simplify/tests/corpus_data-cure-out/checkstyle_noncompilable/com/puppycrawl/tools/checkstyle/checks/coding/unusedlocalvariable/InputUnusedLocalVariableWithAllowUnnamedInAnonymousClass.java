package com.puppycrawl.tools.checkstyle.checks.coding.unusedlocalvariable;

import java.util.List;

public class InputUnusedLocalVariableWithAllowUnnamedInAnonymousClass {
    Runnable anonymous() {
        return new Runnable() {
            @Override
            public void run() {
                var _ = "ok";
                String _ = "ok";
                for (String _ : List.of("ok")) {
                }
                var __ = "violation"; // violation 'Unused named local variable '__''
            }
        };
    }
    void besideAnonymous() {
        var _ = "ok";
        new Runnable() {
            @Override
            public void run() {
            }
        }.run();
    }
}
