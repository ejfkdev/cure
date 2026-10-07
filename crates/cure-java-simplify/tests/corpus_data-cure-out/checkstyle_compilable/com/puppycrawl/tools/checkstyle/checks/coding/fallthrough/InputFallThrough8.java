package com.puppycrawl.tools.checkstyle.checks.coding.fallthrough;

public class InputFallThrough8 {
    void methodLastLine(int i) {
        while (true) {
            switch (i) {
                case 0:
                    i++;
                case 1:
                    i++;
                    break;
                case 2:
                    i++;
                case 3:
                    i--;
                    break;
            }
        }
    }
    void testLastCase(int i) {
        switch (i) {
            case 0:
                i++;
        }
    }
    void testLastCase2(int i) {
        switch (i) {
            case 0:
                i++;
        }
    }
}
