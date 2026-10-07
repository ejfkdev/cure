package spoon.test.loop.testclasses;

import java.util.ArrayList;
import java.util.Collection;
import static java.util.Collections.unmodifiableCollection;

public abstract class Join<T> extends Condition<T> {
    final Collection<Condition<? super T>> conditions;
    @SafeVarargs
  protected Join(Condition<? super T>... conditions) {
        if (conditions == null) 
            throw conditionsIsNull();
        this.conditions = new ArrayList<>();
        for (Condition<? super T> condition : conditions) 
            this.conditions.add(notNull(condition));
    }
    private static NullPointerException conditionsIsNull() {
        return new NullPointerException("The given conditions should not be null");
    }
    private static <T> Condition<T> notNull(Condition<T> condition) {
        return null;
    }
    protected final Collection<Condition<? super T>> conditions() {
        return unmodifiableCollection(conditions);
    }
}
