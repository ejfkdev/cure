package com.puppycrawl.tools.checkstyle.grammar.java25;

public class InputFlexibleConstructorBody {
    public InputFlexibleConstructorBody() {
        System.out.println("hello");
        super();
    }
}

class Jep512 {
    int i;
    String[] arr;
    Jep512() {}
    public int size() {
        return arr.length;
    }
    public boolean isEmpty() {
        return arr.length == 0;
    }
}

class Outer extends Jep512 {
    int i;
    String s = "hello";
    Outer(int number) {
        if (number > 0) {
            throw new IllegalArgumentException("number must be positive");
        }
        i = number;
        super();
    }
    Outer() {}
    public int size() {
        return super.size();
    }
    public boolean isEmpty() {
        return Outer.super.isEmpty();
    }
    class Inner {
        public Inner() {}
        public Inner(long arg) {}
    }
}

class Derived extends Outer.Inner {
    public Derived(Outer s) {
        s.super();
    }
    public Derived() {
        new Outer().super();
    }
    public Derived(int arg) {
        new Outer().super(arg + 1L);
    }
}
