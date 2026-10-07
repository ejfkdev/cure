package spoon.test.fieldaccesses.testclasses;

public class Mouse {
    int[] is;
    int age;
    Mouse son;
    @SuppressWarnings("unused")
	public void meth1() {
        age = 3;
        son = new Mouse();
    }
    public void meth1b() {
        this.age = 3;
        son = new Mouse();
    }
    public void meth2() {
        this.son.age = 3;
    }
    public void meth3() {
        age = is.length;
    }
    public void meth4() {
        is[2] = 4;
    }
}
