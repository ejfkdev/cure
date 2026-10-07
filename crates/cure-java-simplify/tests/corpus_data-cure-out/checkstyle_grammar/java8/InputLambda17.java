package com.puppycrawl.tools.checkstyle.grammar.java8;

import java.util.function.Supplier;

public class InputLambda17 {
    void initPartialTraversalState() {
        SpinedBuffer<P_OUT> b = new SpinedBuffer<>();
        Supplier pusher = () -> new P_OUT().tryAdvance(b);
    }
    private class P_OUT {
        public Object tryAdvance(SpinedBuffer<P_OUT> b) {
            return null;
        }
    }
    class SpinedBuffer<T> {
    }
}
