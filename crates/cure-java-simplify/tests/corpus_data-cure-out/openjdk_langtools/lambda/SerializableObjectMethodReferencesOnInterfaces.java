import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.ObjectInputStream;
import java.io.ObjectOutputStream;
import java.io.Serializable;

public class SerializableObjectMethodReferencesOnInterfaces {
    public static void main(String[] args) throws Exception {
        new Test().run();
    }
    static class Test {
        interface I1 extends Serializable {
        }
        interface I2 extends I1 {
            @Override
            public int hashCode();
        }
        interface F<T, R> extends Serializable {
            R apply(T t);
        }
        enum E {
            ONE
        }
        void run() throws Exception {
            F<I1, Integer> f1 = I1::hashCode;
            F<I2, Integer> f2 = I2::hashCode;
            F<E, Integer> f3 = E::hashCode;
            F<Object, Integer> f4 = Object::hashCode;
            serialDeserial(f1).apply(new I1() {});
            serialDeserial(f2).apply(new I2() {});
            serialDeserial(f3).apply(E.ONE);
            serialDeserial(f4).apply(new Object());
        }
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
