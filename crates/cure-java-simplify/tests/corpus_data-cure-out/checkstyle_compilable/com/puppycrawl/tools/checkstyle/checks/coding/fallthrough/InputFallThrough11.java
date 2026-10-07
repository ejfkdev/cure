package com.puppycrawl.tools.checkstyle.checks.coding.fallthrough;

public class InputFallThrough11 {
    void exhaustiveStatementSane(Object o) {
        switch (o) {
            case Object obj:

                break;
        }
        switch (o) {
            case null:

                break;
            case Object obj:

        }
        switch (o) {
            case Object obj:

                break;
            case null:

        }
        switch (o) {
            case Object obj:

                break;
            case null:

        }
    }
}
