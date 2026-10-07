package com.puppycrawl.tools.checkstyle.grammar.java8;

import java.lang.annotation.ElementType;
import java.lang.annotation.Target;
import java.util.List;
import java.util.function.Supplier;

public class InputMethodReferences7 {
    interface LambdaInt {
        <S, T> void m(S p1, T p2);
    }
    interface MyFunctionalInterface {
        void invokeMethodReference();
    }
    static class LambdaImpl implements LambdaInt {
        <S, T> LambdaImpl(S p1, T p2) {}
        public <S, T> void m(S p1, T p2) {}
    }
    public void m() {}
    public void main(String[] args) {
        MyFunctionalInterface fi = new InputMethodReferences7()::m;
        LambdaInt li = LambdaImpl::new;
        Supplier s;
        li = li::m;
        s = Bar::m;
    }
}

class Bar<T> {
    static Object m() {
        return null;
    }
}

@Target({ ElementType.TYPE_USE, ElementType.TYPE_PARAMETER }) @interface TA {
}

@Target({ ElementType.TYPE_USE, ElementType.TYPE_PARAMETER }) @interface TB {
}
