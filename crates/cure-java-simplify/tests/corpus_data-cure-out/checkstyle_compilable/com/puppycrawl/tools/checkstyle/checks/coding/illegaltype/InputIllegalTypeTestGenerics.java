package com.puppycrawl.tools.checkstyle.checks.coding.illegaltype;

import java.io.Serializable;
import java.util.Map;
import java.util.Set;
import java.util.TreeSet;
import java.util.function.Consumer;

public abstract class InputIllegalTypeTestGenerics {
    private Set<Boolean> privateSet;
    private java.util.List<Map<Boolean, Foo>> privateList;
    public Set<Boolean> set;
    public java.util.List<Map<Boolean, Foo>> list;
    private void methodCall() {
        Bounded.foo();
    }
    public <T extends Boolean, U extends Serializable> void typeParameter(T a) {}
    public void fullName(java.util.ArrayList<? super Boolean> a) {}
    public abstract Set<Boolean> shortName(Set<? super Set<Boolean>> a);
    public Set<? extends Foo<Boolean>> typeArgument() {
        return new TreeSet<Foo<Boolean>>();
    }
    public class MyClass<Foo extends Boolean> {
    }
}

class Bounded {
    public boolean match = new TreeSet<Integer>().stream().allMatch(new TreeSet<>()::add);
    public static <Boolean> void foo() {}
}

class Foo<T extends Boolean & Serializable> {
    void foo() {}
}

@interface Annotation {
    Class<? extends Boolean>[] nonPublic();
    public Class<? extends Boolean>[] value();
        // violation above "Usage of type 'Boolean' is not allowed."
}
