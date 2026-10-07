package com.puppycrawl.tools.checkstyle.checks.coding.finallocalvariable;

import java.util.PriorityQueue;
import java.util.Queue;

public class InputFinalLocalVariableValidateUnnamedVariablesTrue {
    public void testUnnamedVariables() {
        Queue<Integer> q = new PriorityQueue<>();
        q.add(1);
        q.add(2);
        for (Integer i : q) {
            var _ = q.poll();
            var __ = q.poll();
        }
        int _ = sideEffect();
        int _ = sideEffect();
        int _result = sideEffect();
    }
    static {
        Runnable _ = new Runnable() {
            public void run()
            {
            }
        };
    }
    public void testInEnhancedFor() {
        int[] squares = {0, 1, 4, 9, 16, 25};
        for (int i : squares) {}
        for (int _ : squares) {}
        for (int _ : squares) {}
        for (int __ : squares) {}
    }
    public int sideEffect() {
        return 0;
    }
}
