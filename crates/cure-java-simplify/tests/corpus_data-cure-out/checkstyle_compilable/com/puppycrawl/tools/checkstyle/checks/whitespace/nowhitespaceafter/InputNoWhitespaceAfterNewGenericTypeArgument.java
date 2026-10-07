package com.puppycrawl.tools.checkstyle.checks.whitespace.nowhitespaceafter;

import java.io.PrintStream;

public class InputNoWhitespaceAfterNewGenericTypeArgument {
    static interface SomeInterface {
        int SEVENS = 777;
    }
    static interface AnotherInterface {
        int THIRDS = 33;
    }
    static class SomeClass implements SomeInterface {
        int i;
        SomeClass(int i) {
            this.i = i;
        }
    }
    static class ImmediateSubclass extends SomeClass implements SomeInterface {
        float f;
        ImmediateSubclass(int i, float f) {
            super(i);
            this.f = f;
        }
    }
    static final class FinalSubclass extends ImmediateSubclass implements AnotherInterface {
        double d;
        FinalSubclass(int i, float f, double d) {
            super(i, f);
            this.d = d;
        }
    }
    static int errorStatus = 0;
    static void errorAlert(PrintStream out, int errorLevel) {
        out.println("Test: failure #" + errorLevel);
        errorStatus = 2;
    }
    static SomeClass[] v2 = new FinalSubclass[4];
    public static int run(String[] args, PrintStream out) {
        int[] i;
        int[] j;
        SomeInterface[] u;
        AnotherInterface[] w;
        SomeClass[][] x = (SomeClass[][]) null;
        x[0] = (SomeClass[]) new ImmediateSubclass[4];
        if (!(x[0] instanceof ImmediateSubclass[])) 
            errorAlert(out, 8);
        if (!(x[1] instanceof FinalSubclass[])) 
            errorAlert(out, 10);
        w = (AnotherInterface[]) x[1];
        return errorStatus;
    }
}
