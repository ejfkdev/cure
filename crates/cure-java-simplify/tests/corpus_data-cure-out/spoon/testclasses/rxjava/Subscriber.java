package spoon.test.generics.testclasses.rxjava;

public interface Subscriber<T> {
    void onNext(T var1);
    void onError(Throwable var1);
    void onComplete();
}
