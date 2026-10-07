package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

class InputIndentationYieldForceStrict {
    public static void main(final String[] args) {
        int today = 0;
        boolean isWeekDay = switch (today) {
            case 2 -> {
                System.out.println("Monday");
                yield true;
            }
            case 3 -> {
                System.out.println("Tuesday");
                yield true;
            }
            default -> {
                yield true;
            }
        };
        int tomorrow = 1;
        boolean isTomorrowWeekDay = switch (tomorrow) {
            case 3 -> {
                System.out.println("Monday");
                yield true;
            }
            case 4 -> {
                System.out.println("Tuesday");
                yield true;
            }
            default -> {
                yield true;
            }
        };
        boolean isWeekend = switch (today) {
            case 0:
                System.out.println("Saturday");
                yield true;
            case 1:
                System.out.println("Sunday");
                yield true;
            default:
                yield true;
        };
        boolean isWeekendTom = switch (tomorrow) {
            case 1:
                System.out.println("Saturday");
                yield true;
            case 2:
                System.out.println("Sunday");
                yield true;
            default:
                yield true;
        };
    }
    public boolean returnKeywordWrong(int k) {
        return switch (k) {
            case 1 -> {
                yield false;
            }
            case 2 -> {
                yield true;
            }
            case 3 -> {
                yield true;
            }
            default -> {
                yield false;
            }
        };
    }
    public boolean returnKeywordCorrect(int k) {
        return switch (k) {
            case 1 -> {
                yield false;
            }
            case 2 -> {
                yield true;
            }
            case 3 -> {
                yield true;
            }
            default -> {
                yield false;
            }
        };
    }
}
