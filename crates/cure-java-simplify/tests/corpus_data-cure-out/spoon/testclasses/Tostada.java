package spoon.test.filters.testclasses;

public class Tostada extends AbstractTostada implements Honey {
    @Override
	public ITostada make() {
        return new Tostada() {
			@Override
			public void prepare() {
			    	int a = 3;
				super.prepare();
			}
		};
    }
    @Override
	public void prepare() {}
    @Override
	public String toString() {
        return "";
    }
    @Override
	public void honey() {}
    public void foo() {}
}

interface Honey {
    void honey();
}
