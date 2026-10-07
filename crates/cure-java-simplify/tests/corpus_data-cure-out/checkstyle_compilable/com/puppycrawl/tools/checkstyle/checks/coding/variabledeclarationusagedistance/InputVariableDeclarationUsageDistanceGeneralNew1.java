package com.puppycrawl.tools.checkstyle.checks.coding.variabledeclarationusagedistance;

public class InputVariableDeclarationUsageDistanceGeneralNew1 {
    public boolean equals1(Object obj) {
        int a = 5;
        int b = 6;
        switch (1) {
            case 1:
                int count;
                a = a + b;
                b = a + a;
                count = b;
                break;
            case 2:
            case 3:
            case 4:
            case 5:
            case 6:
        }
        return false;
    }
}
