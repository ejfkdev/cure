package com.puppycrawl.tools.checkstyle.grammar.java8;

import java.lang.annotation.ElementType;
import java.lang.annotation.Target;
import java.util.Map;

public class InputAnnotations8 {
    public void main(String[] args) {
        Map.Entry entry;
        MyObject myObject = new MyObject();
        myObject.myMethod();
        myObject.new @NotNull MyObject2();
    }
    @Target(ElementType.TYPE_USE) @interface NotNull {
    }
    @Target(ElementType.TYPE_USE) @interface NotBlank {
    }
    class MyObject {
        public void myMethod() {}
        class MyObject2 {
        }
    }
}
