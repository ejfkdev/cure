package com.puppycrawl.tools.checkstyle.checks.naming.catchparametername;

public class InputCatchParameterNameUnnamed {
    void m() {
        try {} catch (Exception _) {}
        try {} catch (Exception __) {}
        try {} catch (Error | Exception _) {}
        try {} catch (Exception _BAD) {}
        try {} catch (Exception BAD__) {}
        try {} catch (Throwable _) {
            try {} catch (Throwable _) {}
        }
    }
}
