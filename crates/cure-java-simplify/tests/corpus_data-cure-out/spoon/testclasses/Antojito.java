package spoon.test.filters.testclasses;

public class Antojito extends AbstractTostada {
    @Override
	public void prepare() {
        System.out.println("Antojito");
    }
    public void prepare(String ingredient) {}
}
