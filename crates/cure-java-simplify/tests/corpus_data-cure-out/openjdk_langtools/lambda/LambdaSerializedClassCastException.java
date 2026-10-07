import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.ObjectInputStream;
import java.io.ObjectOutputStream;
import java.io.Serializable;
import java.util.function.Function;

public class LambdaSerializedClassCastException {
    public static void main(String[] args) throws Exception {
        Function<String, String> lambda1 = (Function<String, String> & Serializable) Object::toString;
        serialDeserial((Function<Object, String> & Serializable) Object::toString).apply(new Object());
    }
    @SuppressWarnings("unchecked")
    static <T> T serialDeserial(T object) throws Exception {
        ByteArrayOutputStream baos = new ByteArrayOutputStream();
        try (ObjectOutputStream oos = new ObjectOutputStream(baos)) {
            oos.writeObject(object);
        }
        try (ObjectInputStream ois = new ObjectInputStream(new ByteArrayInputStream(baos.toByteArray()))) {
            return (T) ois.readObject();
        }
    }
}
