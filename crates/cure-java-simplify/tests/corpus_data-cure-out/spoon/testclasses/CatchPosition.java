package spoon.test.position.testclasses;

import java.io.IOException;

public class CatchPosition {
    void method() {
        try {
            throw new IOException();
        } catch (IOException e) {
            throw new RuntimeException(e);
        } catch (@Deprecated /*4*/ ClassCastException e) {
            throw new RuntimeException(e);
        } catch (OutOfMemoryError | RuntimeException e) {
            throw new RuntimeException(e);
        }
    }
}
