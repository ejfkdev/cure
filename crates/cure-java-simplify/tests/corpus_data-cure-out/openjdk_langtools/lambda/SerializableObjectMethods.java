import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.ObjectInputStream;
import java.io.ObjectOutputStream;
import java.io.Serializable;

public class SerializableObjectMethods {
    interface I1 extends Serializable {
    }
    interface I2 extends I1 {
        @Override
        public int hashCode();
    }
    interface F<T, R> extends Serializable {
        R apply(T t);
    }
    public static void main(String[] args) throws Exception {
        new SerializableObjectMethods().run();
    }
    void run() throws IOException, ClassNotFoundException {
        saveLoad((F<I1, Integer>) I1::hashCode).apply(new I1() {});
        saveLoad((F<I2, Integer>) I2::hashCode).apply(new I2() {});
    }
    <T, R> F<T, R> saveLoad(F<T, R> value) throws IOException, ClassNotFoundException {
        ByteArrayOutputStream out = new ByteArrayOutputStream();
        try (ObjectOutputStream oos = new ObjectOutputStream(out)) {
            oos.writeObject(value);
        }
        try (ObjectInputStream ois = new ObjectInputStream(new ByteArrayInputStream(out.toByteArray()))) {
            return (F<T, R>) ois.readObject();
        }
    }
}
