package com.puppycrawl.tools.checkstyle.checks.coding.fallthrough;

import java.io.IOException;

public class InputFallThroughTryCatchInSwitch {
    public int foo(int x) {
        switch (x) {
            case 1:
                try {
                    throw new IOException("Exception occurred.");
                } catch (IOException e) {
                    if (e.getMessage().contains("Exception")) {
                        break;
                    } else {
                        return 0;
                    }
                } catch (Exception e) {
                    for (int i = 0; i < 3; i++) {
                        if (i == 1) {
                            break;
                        }
                    }
                }
            default:
        }
        return 0;
    }
}
