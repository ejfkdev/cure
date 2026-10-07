package com.puppycrawl.tools.checkstyle.grammar.antlr4;

public class InputAntlr4AstRegressionFallThroughDefault {
    void method(int i, int j, boolean cond) {
        while (true) {
            switch (i) {
                case 0:
                case 1:
                    i++;
                    break;
                case 2:
                    i++;
                case 3:
                    i++;
                    break;
                case 4:
                    return;
                case 5:
                    throw new RuntimeException("");
                case 6:
                    continue;
                case 7:
                    {
                        break;
                    }
                case 8:
                    {
                        return;
                    }
                case 9:
                    {
                        throw new RuntimeException("");
                    }
                case 10:
                    {
                        continue;
                    }
                case 11:
                    {
                        i++;
                    }
                case 12:
                    break;
                case 13:
                    {
                        return;
                    }
                case 14:
                    {
                        return;
                    }
                case 15:
                    do {
                        System.identityHashCode("something");
                        return;
                    } while (true);
                case 16:
                    for (int j1 = 0; j1 < 10; j1++) {
                        "something";
                        return;
                    }
                case 17:
                    while (true) 
                        throw new RuntimeException("");
                case 18:
                    while (cond) {
                        break;
                    }
                case 19:
                    try {
                        i++;
                        break;
                    } catch (RuntimeException e) {
                        break;
                    } catch (Error e) {
                        return;
                    }
                case 20:
                    try {
                        i++;
                        break;
                    } catch (RuntimeException e) {} catch (Error e) {
                        return;
                    }
                case 21:
                    try {
                        i++;
                    } catch (RuntimeException e) {
                        i--;
                    } finally {
                        break;
                    }
                case 22:
                    try {
                        i++;
                        break;
                    } catch (RuntimeException e) {
                        i--;
                        break;
                    } finally {
                        i++;
                    }
                case 23:
                    switch (j) {
                        case 1:
                            continue;
                        case 2:
                            return;
                        default:
                            return;
                    }
                case 24:
                    switch (j) {
                        case 1:
                            continue;
                        case 2:
                            break;
                        default:
                            return;
                    }
                default:
                    i++;
            }
        }
    }
    void methodFallThru(int i, int j, boolean cond) {
        while (true) {
            switch (i) {
                case -1:
                case 0:
                case 1:
                    i++;
                    break;
                case 2:
                    i++;
                case 3:
                    i++;
                    break;
                case 4:
                    return;
                case 5:
                    throw new RuntimeException("");
                case 6:
                    continue;
                case 7:
                    {
                        break;
                    }
                case 8:
                    {
                        return;
                    }
                case 9:
                    {
                        throw new RuntimeException("");
                    }
                case 10:
                    {
                        continue;
                    }
                case 11:
                    {
                        i++;
                    }
                case 12:
                    break;
                case 13:
                    {
                        return;
                    }
                case 14:
                    {
                        return;
                    }
                case 15:
                    do {
                        System.identityHashCode("something");
                        return;
                    } while (true);
                case 16:
                    for (int j1 = 0; j1 < 10; j1++) {
                        "something";
                        return;
                    }
                case 17:
                    while (cond) 
                        throw new RuntimeException("");
                case 18:
                    while (cond) {
                        break;
                    }
                case 19:
                    try {
                        i++;
                        break;
                    } catch (RuntimeException e) {
                        break;
                    } catch (Error e) {
                        return;
                    }
                case 20:
                    try {
                        i++;
                        break;
                    } catch (RuntimeException e) {} catch (Error e) {
                        return;
                    }
                case 21:
                    try {
                        i++;
                    } catch (RuntimeException e) {
                        i--;
                    } finally {
                        break;
                    }
                case 22:
                    try {
                        i++;
                        break;
                    } catch (RuntimeException e) {
                        i--;
                        break;
                    } finally {
                        i++;
                    }
                case 23:
                    switch (j) {
                        case 1:
                            continue;
                        case 2:
                            return;
                        default:
                            return;
                    }
                case 24:
                    i++;
                case 25:
                    i++;
                    break;
                case 26:
                    switch (j) {
                        case 1:
                            continue;
                        case 2:
                            break;
                        default:
                            return;
                    }
                default:
                    i++;
            }
        }
    }
    void methodFallThruCC(int i, int j, boolean cond) {
        while (true) {
            switch (i) {
                case 0:
                    i++;
                case 1:
                    i++;
                case 2:
                    {
                        i++;
                    }
                case 3:
                    i++;
                case 4:
                    break;
                case 5:
                    i++;
            }
        }
    }
    void methodFallThruC(int i, int j, boolean cond) {
        while (true) {
            switch (i) {
                case 0:
                    i++;
                case 1:
                    i++;
                case 2:
                    i++;
                case 3:
                    break;
                case 4:
                    i++;
            }
        }
    }
    void methodFallThruC2(int i, int j, boolean cond) {
        while (true) {
            switch (i) {
                case 0:
                    i++;
                case 1:
                    i++;
                case 2:
                    i++;
                case 3:
                    break;
                case 4:
                    i++;
            }
        }
    }
    void methodFallThruCOtherWords(int i, int j, boolean cond) {
        while (true) {
            switch (i) {
                case 0:
                    i++;
                case 1:
                    i++;
                case 2:
                    i++;
                case 3:
                    break;
                case 4:
                    i++;
            }
        }
    }
    void methodFallThruCCustomWords(int i, int j, boolean cond) {
        while (true) {
            switch (i) {
                case 0:
                    i++;
                case 1:
                    i++;
                case 2:
                    i++;
                case 3:
                    break;
                case 4:
                    i++;
            }
        }
    }
    void methodFallThruLastCaseGroup(int i, int j, boolean cond) {
        while (true) {
            switch (i) {
                case 0:
                    i++;
            }
            switch (i) {
                case 0:
                    i++;
            }
            switch (i) {
                case 0:
                    i++;
            }
        }
    }
    void method1472228(int i) {
        switch (i) {
            case 2:
                break;
            default:
        }
    }
    void nestedSwitches() {
        switch (hashCode()) {
            case 1:
                switch (hashCode()) {
                    case 1:
                }
            default:
        }
    }
    void nextedSwitches2() {
        switch (hashCode()) {
            case 1:
                switch (hashCode()) {
                }
            case 2:
                System.lineSeparator();
                break;
        }
    }
    void ifWithoutBreak() {
        switch (hashCode()) {
            case 1:
                {
                    System.lineSeparator();
                }
            case 2:
                System.lineSeparator();
                break;
        }
    }
    void noCommentAtTheEnd() {
        switch (hashCode()) {
            case 1:
                System.lineSeparator();
            case 2:
                System.lineSeparator();
                break;
        }
    }
    void synchronizedStatement() {
        switch (hashCode()) {
            case 1:
                synchronized (this) {
                    break;
                }
            case 2:
                {
                    synchronized (this) {
                        break;
                    }
                }
            case 3:
                synchronized (this) {}
            default:
                break;
        }
    }
    void multipleCasesOnOneLine() {
        int i = 0;
        switch (i) {
            case 0:
            case 1:
                i *= i;
            case 2:
            case 3:
                i *= i;
            case 4:
            case 5:
                i *= i;
            case 6:
            case 7:
                i *= i;
                break;
            default:
                throw new RuntimeException();
        }
    }
    void methodFallThruWithDash(int i, int j, boolean cond) {
        while (true) {
            switch (i) {
                case 0:
                    i++;
                case 1:
                    i++;
                case 2:
                    i++;
                case 3:
                    i++;
                case 4:
                    i++;
                case 5:
                    i++;
                case 6:
                    i++;
                case 7:
                    i++;
                case 8:
                    i++;
                case 9:
                    i++;
                case 10:
                    i++;
                case 11:
                    i++;
                case 12:
                    i++;
                case 13:
                    i++;
                case 14:
                    i++;
                case 15:
                    i++;
                case 16:
                    i++;
                default:
                    throw new RuntimeException();
            }
        }
    }
}
