package spoon.test.filters.testclasses;

public class SubTostada extends Tostada {
    @Override
	public void prepare() {
        System.out.println("SubTostada");
        super.prepare();
    }
}
