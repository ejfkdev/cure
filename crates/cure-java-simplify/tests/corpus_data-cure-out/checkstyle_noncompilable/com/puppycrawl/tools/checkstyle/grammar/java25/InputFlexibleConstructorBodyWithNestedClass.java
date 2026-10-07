package com.puppycrawl.tools.checkstyle.grammar.java25;

class InputFlexibleConstructorBodyWithNestedClass {
    int i;
    public void hello() {
        System.out.println("Hello");
    }
    class Inner {
        int j;
        Inner() {
            var y = InputFlexibleConstructorBodyWithNestedClass.this.i;
            hello();
            InputFlexibleConstructorBodyWithNestedClass.this.hello();
            super();
        }
    }
}
