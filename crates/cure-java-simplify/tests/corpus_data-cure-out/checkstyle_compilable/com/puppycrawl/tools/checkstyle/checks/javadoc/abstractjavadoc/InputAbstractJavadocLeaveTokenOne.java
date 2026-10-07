package com.puppycrawl.tools.checkstyle.checks.javadoc.abstractjavadoc;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

class InputAbstractJavadocLeaveTokenOne {
    protected class B {
    }
    private/**nope*/ static class C {
    }
    @Component class D {
    }
    @Component/**nope*/ private class E {
    }
    private/**nope*/ @Component class F {
    }
}

class AA1 {
    AA1() {}
}

class BB1 {
    private BB1() {}
}

class DD1 {
    @Component DD1() {}
}

class EE1 {
    @Component/**nope*/ private EE1() {}
}

class FF1 {
    private/**nope*/ @Component FF1() {}
}

class AAA1 {
    void a() {}
}

class BBB1 {
    private void a() {}
}

class CCC1 {
    static/**nope*/ private void a() {}
}

class DDD1 {
    @Component void a() {}
}

class EEE1 {
    @Component/**nope*/ private void a() {}
}

class FFF1 {
    static/**nope*/ @Component/**nope*/ private void a() {}
}

class GGG1 {
    void a(@Component int a) {}
}

class HHH1 {
    java.lang.String a() {
        return null;
    }
}
