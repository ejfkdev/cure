package spoon.test.type.testclasses;

import spoon.test.annotation.testclasses.TypeAnnotation;
import java.io.Serializable;
import java.lang.annotation.Annotation;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;

public class Pozole<A extends Annotation> {
    @Spice(klass = Pozole.class)
	public void make() {
        List<A> list = new ArrayList<@TypeAnnotation(clazz = Float.class, classes = {Integer.class}) A>();
        addDeliciousIngredient((Class<? extends A>) Annotation.class);
    }
    void addDeliciousIngredient(java.lang.Class<? extends A> ingredient) {}
    public void eat() {}
    public void season() {}
    public void prepare() {
        class Test<T extends Runnable & Serializable> {
        		}
        		final Runnable runnable = (Runnable & Serializable) () -> System.err.println("");
    }
    public void finish() {
        class Test<T extends Runnable> {
        		}
        		final Runnable runnable = (Runnable) () -> System.err.println("");
    }
}
