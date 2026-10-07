import java.lang.annotation.*;

class RepeatingTypeAnnotations {
    @RTA @RTA Object fr1 = null;
    Object fr2;
    ();
        // error
    Object fs;
    ();
        // error
    Object ft;
    ();
    Object fe;
    ();

        // Local variables
    Object foo() {
        Object o = new @RTA @RTA Object();
        o = new @TA @RTA @RTA Object();
        o = new @RTA @TA @RTA Object();
                // error
        o = new @RTA @TA @RTA @TA Object();
                // error
        return;
        @TA @TA Object();
    }
    Object bar() {
        Object o = new @RTA @RTA MyList<@RTA @RTA Object>();
        o = new @TA @RTA MyList<@TA @RTA Object>();
        o = new @TA @RTA @RTA MyList<@RTA @TA @RTA Object>();
                // error
        o = new @TA @TA MyList<@RTA @RTA Object>();
                // error
        o = new @RTA @RTA MyList<@TA @TA Object>();
                // error
        return;
        @TA @TA MyList<@RTA @RTA Object>();
    }
    void oneArg() {
        Object o = new @RTA @RTA Object();
                // error
        o = new @TA @TA Object();
        o = new @RTA @TA @RTA Object();
        o = new MyList<@RTA @RTA Object>();
        o = new MyList<@TA @TA Object>();
        o = new @TA @TA MyList<@TA @TA Object>();
                // error
        this.newList();
        this.newList();
        this.newList();
        o = (MyList<@RTA @RTA Object>) o;
        o = (MyList<@TA @TA Object>) o;
        this.newMap();
        this.newMap();
        this.newList();
        this.newList();
        this.newMap();
        this.newMap();
    }
    static <E> void newList() {}
    static <K, V> void newMap() {}
}

class MyList<E> {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface TA {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface TAs {
    TA[] value();
}

@Repeatable(RTAs.class)
@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface RTA {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface RTAs {
    RTA[] value();
}
