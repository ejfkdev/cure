package com.puppycrawl.tools.checkstyle.checks.naming.methodtypeparametername;

import java.io.Serializable;

public class InputMethodTypeParameterName1<t> {
    public <TT> void foo() {}
    <e_e> void foo(int i) {}
}

class Other1<foo extends Serializable & Cloneable> {
    foo getOne() {
        return null;
    }
    <Tfo$o2T extends foo> Tfo$o2T getTwo(Tfo$o2T a) {
        return null;
    }
    <foo extends Runnable> foo getShadow() {
        return null;
    }
    static class Junk<foo> {
        <_fo extends foo> void getMoreFoo() {}
    }
}

class MoreOther1<T extends Cloneable> {
    <E extends T> void getMore() {
        new Other() {
            <T> void getMoreFoo() { // violation 'Name 'T' must match pattern'
            }
        };
    }
}

interface Boo1<Input> {
    Input boo();
}

interface FooInterface1<T> {
    T foo();
}

interface FooInterface3 {
    Input foo();
}

class Input1 {
}
