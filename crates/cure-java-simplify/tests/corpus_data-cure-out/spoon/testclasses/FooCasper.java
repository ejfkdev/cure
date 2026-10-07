package spoon.test.prettyprinter.testclasses;

public class FooCasper {
    FooCasper f;
    public FooCasper bug1() {
        if (new FooCasper(1).foo() != null) {
            throw new Error();
        }
        f = new FooCasper(1).foo();
        System.out.println(f);
        f.bar();
        return null;
    }
    public FooCasper foo() {
        return foo2();
    }
    public FooCasper foo2() {
        return null;
    }
    public FooCasper foo3() {
        return f;
    }
    public void bar() {}
    public FooCasper foo5(FooCasper o) {
        return o;
    }
    public void bug2() {
        null.f.bar();
    }
    public void bug3() {
        null[0].bar();
    }
    public void bug4() {
        null.toString();
    }
    public FooCasper(int i) {}
    public FooCasper() {}
    public void toString_support() {
        null.toString();
    }
    public void array_support() {
        FooCasper[] array = new FooCasper[10];
        array[1] = null;
        array[2] = array[1];
        array[2].bar();
    }
    public void literal() {
        null.literal();
    }
    public void literal2() {
        FooCasper tab = new FooCasper();
        null.literal();
    }
}
