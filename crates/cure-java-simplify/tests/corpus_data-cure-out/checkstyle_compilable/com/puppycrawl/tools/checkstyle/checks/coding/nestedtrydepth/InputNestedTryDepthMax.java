package com.puppycrawl.tools.checkstyle.checks.coding.nestedtrydepth;

public class InputNestedTryDepthMax {
    void foo() {
        try {} catch (Exception e) {}
        try {
            try {} catch (Exception e) {}
        } catch (Exception e) {}
        try {
            try {
                try {} catch (Exception e) {}
            } catch (Exception e) {}
        } catch (Exception e) {}
        try {
            try {
                try {
                    try {} catch (Exception e) {}
                } catch (Exception e) {}
            } catch (Exception e) {}
        } catch (Exception e) {}
    }
}
