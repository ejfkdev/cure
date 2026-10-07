import java.util.function.Consumer;

public class SealedTypeChanges {
    public static void main(String... args) throws Exception {
        new SealedTypeChanges().run();
    }
    void run() throws Exception {
        doRun(this::expressionIntf, this::validateMatchException);
        doRun(this::statementIntf, this::validateMatchException);
        doRun(this::expressionCls, this::validateMatchException);
        doRun(this::statementCls, this::validateMatchException);
        doRun(this::statementFallThrough, this::validateMatchException);
        doRun(this::expressionCoveredIntf, this::validateTestException);
        doRun(this::statementCoveredIntf, this::validateTestException);
        doRun(this::expressionCoveredCls, this::validateTestException);
        doRun(this::statementCoveredCls, this::validateTestException);
    }
    <T> void doRun(Consumer<T> t, Consumer<Throwable> validateException) throws Exception {
        t.accept((T) new A());
        try {
            t.accept((T) Class.forName("SealedTypeChangesClass").newInstance());
            throw new AssertionError("Expected an exception, but none thrown.");
        } catch (Throwable ex) {
            validateException.accept(ex);
        }
    }
    void validateMatchException(Throwable t) {
        if (!(t instanceof MatchException)) {
            throw new AssertionError("Unexpected exception", t);
        }
    }
    void validateTestException(Throwable t) {
        if (!(t instanceof TestException)) {
            throw new AssertionError("Unexpected exception", t);
        }
    }
    void statementIntf(SealedTypeChangesIntf obj) {
        switch (obj) {
            case A a -> System.err.println(1);
        }
    }
    int expressionIntf(SealedTypeChangesIntf obj) {
        return switch (obj) {
            case A a -> 0;
        };
    }
    void statementCls(SealedTypeChangesCls obj) {
        switch (obj) {
            case A a -> System.err.println(1);
        }
    }
    void statementFallThrough(SealedTypeChangesCls obj) {
        switch (obj) {
            case A a:
                System.err.println(1);
        }
    }
    int expressionCls(SealedTypeChangesCls obj) {
        return switch (obj) {
            case A a -> 0;
        };
    }
    void statementCoveredIntf(SealedTypeChangesIntf obj) {
        switch (obj) {
            case A a -> System.err.println(1);
            case SealedTypeChangesIntf o -> throw new TestException();
        }
    }
    int expressionCoveredIntf(SealedTypeChangesIntf obj) {
        return switch (obj) {
            case A a -> 0;
            case SealedTypeChangesIntf o -> throw new TestException();
        };
    }
    void statementCoveredCls(SealedTypeChangesCls obj) {
        switch (obj) {
            case A a -> System.err.println(1);
            case SealedTypeChangesCls o -> throw new TestException();
        }
    }
    int expressionCoveredCls(SealedTypeChangesCls obj) {
        return switch (obj) {
            case A a -> 0;
            case SealedTypeChangesCls o -> throw new TestException();
        };
    }
    final static class A extends SealedTypeChangesCls implements SealedTypeChangesIntf {
    }
    class TestException extends RuntimeException {
    }
}

sealed interface SealedTypeChangesIntf permits SealedTypeChanges.A {
}

sealed abstract class SealedTypeChangesCls permits SealedTypeChanges.A {
}
