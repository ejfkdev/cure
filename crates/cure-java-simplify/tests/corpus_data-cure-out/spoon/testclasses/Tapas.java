package spoon.test.targeted.testclasses;

import java.util.function.Consumer;

public class Tapas<T> {
    public interface SingleOnSubscribe<T> extends Consumer<SingleSubscriber<? super T>> {
    }
    public interface SingleSubscriber<T> {
    }
    public static <T> Tapas<T> create(SingleOnSubscribe<T> onSubscribe) {
        return new Tapas<T>() {
			class InnerSubscriber implements SingleSubscriber<T> {
				int index;
				public InnerSubscriber(int index) {
					this.index = index;
				}
			}
		};
    }
    public static <T> Tapas<T> create2() {
        return create((s) -> {
            class InnerSubscriber implements SingleSubscriber<T> {
            				long index;
            				public InnerSubscriber(int index) {
            					this.index = index;
            				}
            			}
        });
    }
    public static <T> Tapas<T> equals() {
        return create((s) -> {
            class InnerSubscriber implements SingleSubscriber<T> {
            				final int index;
            				public InnerSubscriber(int index) {
            					this.index = index;
            				}
            			}
        });
    }
}
