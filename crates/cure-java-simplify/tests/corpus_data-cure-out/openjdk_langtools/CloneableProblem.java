interface A extends Cloneable {
    public Object clone() throws CloneNotSupportedException;
}

interface B extends A {
}

interface C extends A {
}

interface D extends B, C {
}

public class CloneableProblem implements D {
    private int i;
    public CloneableProblem(int i) {
        this.i = i;
    }
    public Object clone() {
        CloneableProblem theCloneableProblem = null;
        try {
            theCloneableProblem = (CloneableProblem) super.clone();
            theCloneableProblem.i = i;
        } catch (CloneNotSupportedException cnse) {}
        return theCloneableProblem;
    }
    public static void main(String[] argv) {
        try {
            A a1 = (A) new CloneableProblem(0).clone();
            B b1 = (B) new CloneableProblem(0).clone();
            C c1 = (C) new CloneableProblem(0).clone();
            D d1 = (D) new CloneableProblem(0).clone();
        } catch (CloneNotSupportedException cnse) {}
    }
}
