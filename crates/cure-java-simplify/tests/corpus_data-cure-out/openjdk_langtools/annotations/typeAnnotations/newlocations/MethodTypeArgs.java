import java.lang.annotation.*;

class MethodTypeArgs {
    void oneArg() {
        this.newList();
        this.newList();
        MethodTypeArgs.newList();
        MethodTypeArgs.newList();
    }
    void twoArg() {
        this.newMap();
        this.newMap();
        MethodTypeArgs.newMap();
        MethodTypeArgs.newMap();
    }
    void withArraysIn() {
        this.newList();
        this.newList();
        this.newMap();
    }
    static <E> void newList() {}
    static <K, V> void newMap() {}
}

class MyList<E> {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface A {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface B {
    int value();
}
