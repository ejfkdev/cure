import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.ObjectInputStream;
import java.io.ObjectOutputStream;
import java.io.Serializable;
import java.util.function.*;

public class SerializedLambdaInLocalClass {
    public static void main(String[] args) {
        SerializedLambdaInLocalClass s = new SerializedLambdaInLocalClass();
        s.test(s::f_lambda_in_anon);
        s.test(s::f_lambda_in_local);
        s.test(s::f_lambda_in_lambda);
    }
    void test(IntFunction<Supplier<F>> fSupplier) {
        try {
            F f = fSupplier.apply(42).get();
            var baos = new ByteArrayOutputStream();
            try (var oos = new ObjectOutputStream(baos)) {
                oos.writeObject(f);
            }
            var bais = new ByteArrayInputStream(baos.toByteArray());
            try (var ois = new ObjectInputStream(bais)) {
                F f2 = (F) ois.readObject();
                if (f2.getValue() != f.getValue()) {
                    throw new AssertionError(String.format("Found: %d, expected %d", f2.getValue(), f.getValue()));
                }
            }
        } catch (IOException | ClassNotFoundException ex) {
            throw new AssertionError(ex);
        }
    }
    interface F extends Serializable {
        int getValue();
    }
    Supplier<F> f_lambda_in_anon(int x) {
        return new Supplier<F>() {
            @Override
            public F get() {
                return () -> x;
            }
        };
    }
    Supplier<F> f_lambda_in_local(int x) {
        class FSupplier implements Supplier<F> {
                    @Override
                    public F get() {
                        return () -> x;
                    }
                }
                return new FSupplier();
    }
    Supplier<F> f_lambda_in_lambda(int x) {
        return () -> () -> x;
    }
}
