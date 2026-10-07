package spoon.test.filters.testclasses;

public abstract class AbstractTostada implements ITostada {
    @Override
	public ITostada make() {
        return new Tostada() {
			@Override
			public void prepare() {
				super.prepare();
			}

			@Override
			public ITostada make() {
				return super.make();
			}
		};
    }
    public abstract void prepare();
    public void honey() {}
}
