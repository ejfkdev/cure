package com.puppycrawl.tools.checkstyle.checks.coding.hiddenfield;

class InputHiddenField6 {
    private int hidden = 0;
    public InputHiddenField6() {}
    public InputHiddenField6(int hidden) {}
    public void shadow() {}
    public void shadowFor() {
        for (int hidden = 0; hidden < 1; hidden++) {}
    }
    public void shadowParam(int hidden) {}
    public class Inner {
        private int innerHidden = 0;
        public Inner() {}
        public Inner(int innerHidden) {}
        private void innerShadow() {}
        private void innerShadowFor() {
            for (int innerHidden = 0; innerHidden < 1; innerHidden++) {}
            for (int hidden = 0; hidden < 1; hidden++) {}
        }
        private void shadowParam(int innerHidden, int hidden) {}
        {}
    }
    {}
}

interface NothingHidden6 {
    public static int notHidden = 0;
    public void noShadow(int notHidden);
}

class PropertySetter16 {
    private int prop;
    public void setProp(int prop) {
        this.prop = prop;
    }
    public void setprop(int prop) {
        this.prop = prop;
    }
    public void setProp(int prop, int extra) {
        this.prop = prop;
    }
}

class PropertySetter26 {
    private int prop;
    public int setProp(int prop) {
        this.prop = prop;
        return 0;
    }
}

class StaticFields6 {
    private static int hidden;
    public static void staticMethod() {}
    public void method() {}
    static {}
    {}
}

class StaticMethods6 {
    private int notHidden;
    public static void method() {}
    static {}
    private int x;
    private static int y;
    static class Inner {
        void useX(int x) {
            x++;
        }
        void useY(int y) {
            y++;
        }
    }
}

enum HiddenEnum16 {
    A(129), B(283), C(1212)
    {
        /**
         * Should not be flagged as violation as we don't check
         * hidden class level fields
         */
        int hidden;

        public void doSomething()
        {
            //Should be flagged as hiding enum constant member
            int hidden = 0; // violation ''hidden' hides a field'
        }
    };
    int hidden;
    static int hiddenStatic;
    HiddenEnum16(int hidden) {}
    public void doSomething() {}
    public static void doSomethingStatic() {}
}

abstract class InputHiddenFieldBug10845126 {
    String x;
    public abstract void methodA(String x);
}

class Bug33709466 {
    private int xAxis;
    public void setxAxis(int xAxis) {
        this.xAxis = xAxis;
    }
}

class PropertySetter36 {
    private int prop;
    public PropertySetter36 setProp(int prop) {
        this.prop = prop;
        return this;
    }
}

enum PropertySetter46 {
    INSTANCE;
    private int prop;
    private int prop2;
    public void setProp(int prop) {
        this.prop = prop;
    }
    public PropertySetter46 setProp2(int prop2) {
        this.prop2 = prop2;
        return this;
    }
}

class OneLetterField6 {
    int i;
    void setI(int i) {
        this.i = i;
    }
    enum Inner {
    }
}

class DuplicateFieldFromPreviousClass6 {
    public void method() {}
}

class NestedEnum6 {
    enum Test {
        A, B, C;
        int i;
    }
    void method(int i) {}
}
