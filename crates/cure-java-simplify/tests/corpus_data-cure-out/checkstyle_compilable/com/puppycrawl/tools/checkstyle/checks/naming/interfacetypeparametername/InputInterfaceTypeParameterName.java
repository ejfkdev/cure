package com.puppycrawl.tools.checkstyle.checks.naming.interfacetypeparametername;

import java.io.Serializable;

public class InputInterfaceTypeParameterName<t> {
    public <TT> void foo() {}
    <e_e> void foo(int i) {}
}

class Other<foo extends Serializable & Cloneable> {
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

class MoreOther<T extends Cloneable> {
    <E extends T> void getMore() {
        new Other() {
            <T> void getMoreFoo() {
            }
        };
    }
}

interface Boo<Input> {
    Input boo();
}

interface FooInterface<T> {
    T foo();
}

interface FooInterface2 {
    Input foo();
}

class Input {
}
