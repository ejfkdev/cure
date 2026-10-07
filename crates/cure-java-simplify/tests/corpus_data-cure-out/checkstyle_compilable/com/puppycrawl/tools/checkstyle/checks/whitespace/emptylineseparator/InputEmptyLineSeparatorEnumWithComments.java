package com.puppycrawl.tools.checkstyle.checks.whitespace.emptylineseparator;

public class InputEmptyLineSeparatorEnumWithComments {
    enum State {
        NEW, SUCCESS, FAILURE
    }
    enum State1 {
        NEW, SUCCESS, FAILURE
    }
    enum State2 {
        NEW, SUCCESS, FAILURE
    }
    enum State3 {
        NEW, SUCCESS, FAILURE
    }
    enum State4 {
        NEW, SUCCESS, FAILURE
    }
    enum State5 {
        SUCCESS { // violation 2 lines above ''//' has more than 1 empty lines before.'
            @Override
            public String getMessage() {
                return "Success";
            }
        }, FAILURE;
        public String getMessage() {
            return "Failure";
        }
    }
}
