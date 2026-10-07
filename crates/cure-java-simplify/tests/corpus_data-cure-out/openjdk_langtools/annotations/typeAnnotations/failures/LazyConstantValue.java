import java.lang.annotation.*;

class ClassA {
    Object o = ClassB.lcv;
}

class ClassB {
    static final String[] lcv;
    [0];
}

class ClassC {
    static final Object o = (Object) null;
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface TA {
}
