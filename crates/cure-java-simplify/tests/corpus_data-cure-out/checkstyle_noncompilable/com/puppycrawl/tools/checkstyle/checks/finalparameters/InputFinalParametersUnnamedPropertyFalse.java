package com.puppycrawl.tools.checkstyle.checks.finalparameters;

import java.util.PriorityQueue;
import java.util.Queue;

public class InputFinalParametersUnnamedPropertyFalse {
    void testUnnamedCatchParameter() {
        try {
            throw new Exception();
        } catch (Exception _) {}
        try {
            int x = 1 / 0;
        } catch (Exception __) {}
        try {
            int x = 1 / 0;
        } catch (Exception _e) {}
        try {
            int x = 1 / 0;
        } catch (Exception e_) {}
    }
    void testUnnamedForEachParameter() {
        Queue<Integer> q = new PriorityQueue<>();
        q.add(1);
        q.add(2);
        for (Integer _ : q) {
            var _ = q.poll();
        }
        for (Integer __ : q) {
            var _ = q.poll();
        }
        for (Integer _i : q) {
            var _ = q.poll();
        }
        for (Integer i_ : q) {
            var _ = q.poll();
        }
    }
}
