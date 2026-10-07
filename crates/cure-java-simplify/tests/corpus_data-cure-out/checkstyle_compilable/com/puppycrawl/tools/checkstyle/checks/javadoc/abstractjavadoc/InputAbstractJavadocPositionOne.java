package com.puppycrawl.tools.checkstyle.checks.javadoc.abstractjavadoc;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

class InputAbstractJavadocPositionOne {
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

class AA {
    AA() {}
}

class BB {
    private BB() {}
}

class DD {
    @Component DD() {}
}

class EE {
    @Component/**nope*/ private EE() {}
}

class FF {
    private/**nope*/ @Component FF() {}
}

class AAA {
    void a() {}
}

class BBB {
    private void a() {}
}

class CCC {
    static/**nope*/ private void a() {}
}

class DDD {
    @Component void a() {}
}

class EEE {
    @Component/**nope*/ private void a() {}
}

class FFF {
    static/**nope*/ @Component/**nope*/ private void a() {}
}

class GGG {
    void a(@Component int a) {}
}

class HHH {
    java.lang.String a() {
        return null;
    }
}
