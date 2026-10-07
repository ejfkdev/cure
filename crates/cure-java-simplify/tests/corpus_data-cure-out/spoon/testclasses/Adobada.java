package spoon.test.delete.testclasses;

@Deprecated
public class Adobada {
    {}
    static {}
    public Adobada() {}
    public void m() {}
    public Adobada m2() {
        return new Adobada() {
			@Override
			public void m() {
				int i;
				int j;
			}
		};
    }
    public void m3() {
        switch (1) {
            case 1:
                int i;
                int j;
            default:
                int o;
                int b;
        }
    }
    public void m4(int i, float j, String s) {
        System.err.println("");
        int k;
        j = i = k = 3;
    }
    public void methodUsingjlObjectMethods() {
        notify();
    }
}
