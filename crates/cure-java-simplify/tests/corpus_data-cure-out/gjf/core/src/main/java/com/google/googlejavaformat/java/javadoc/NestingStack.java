package com.google.googlejavaformat.java.javadoc;

import java.util.ArrayDeque;
import java.util.Collection;
import java.util.Deque;
import org.jspecify.annotations.Nullable;

final class NestingStack<E> {
    private final Deque<E> stack = new ArrayDeque<>();
    Iterable<E> bottomToTop() {
        return stack::descendingIterator;
    }
    void push(E value) {
        stack.push(value);
    }
    @Nullable E popIfIn(Collection<E> values) {
        return isEmpty() || !values.contains(stack.peek()) ? null : stack.pop();
    }
    void popUntil(E value) {
        if (stack.contains(value)) {
            E popped;
            do {
                popped = stack.pop();
            } while (!popped.equals(value));
        }
    }
    void popUntil(Class<? extends E> valueClass) {
        if (stack.stream().anyMatch(valueClass::isInstance)) {
            while (!valueClass.isInstance(stack.pop())) {}
        }
    }
    boolean contains(E value) {
        return stack.contains(value);
    }
    boolean containsAny(Collection<E> values) {
        return stack.stream().anyMatch(values::contains);
    }
    boolean isEmpty() {
        return stack.isEmpty();
    }
    void reset() {
        stack.clear();
    }
    static final class Int {
        private final Deque<Integer> stack = new ArrayDeque<>();
        private int total;
        int total() {
            return total;
        }
        void push(int value) {
            stack.push(value);
            total += value;
        }
        void push() {
            push(1);
        }
        void popIfNotEmpty() {
            if (!stack.isEmpty()) {
                total -= stack.pop();
            }
        }
        boolean isEmpty() {
            return stack.isEmpty();
        }
        void reset() {
            stack.clear();
            total = 0;
        }
    }
}
