package com.puppycrawl.tools.checkstyle.checks.naming.localvariablename;

import java.util.PriorityQueue;
import java.util.Queue;

public class InputLocalVariableNameUnnamedVariables {
    public void testLocalVariables() {
        Queue<Integer> q = new PriorityQueue<>();
        q.add(1);
        q.add(2);
        for (Integer element : q) {
            var _ = q.poll();
            var _ = q.poll();
        }
    }
    public void testLocalVariablesInForLoops() {
        Queue<Integer> q = new PriorityQueue<>();
        for (Integer _ : q) {
            var x1 = q.poll();
            var x2 = q.poll();
        }
        for (Integer __ : q) {
            var x1 = q.poll();
            var x2 = q.poll();
        }
        for (int ab = 0, _ = Integer.valueOf(1); ab < 3; ab++) {}
    }
}
