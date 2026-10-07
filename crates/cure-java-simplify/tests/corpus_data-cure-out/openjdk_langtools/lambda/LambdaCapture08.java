import java.util.function.*;

interface LambdaCapture08 {
    Object O = new Object() {
        IntSupplier x(int m) {
            return () -> m;
        }
    };
}
