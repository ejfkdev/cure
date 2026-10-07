import java.io.*;
import java.util.Map;
import java.util.HashMap;

public class LambdaLambdaSerialized {
    static int assertionCount = 0;
    static void assertTrue(boolean cond) {
        assertionCount++;
        if (!cond) 
            throw new AssertionError();
    }
    public static void main(String[] args) throws Exception {
        try {
            ByteArrayOutputStream baos = new ByteArrayOutputStream();
            ObjectOutput out = new ObjectOutputStream(baos);
            LSI<LSI<Map>> ssi = () -> () -> new HashMap();
            write(out, ssi);
            out.flush();
            out.close();
            ObjectInputStream in = new ObjectInputStream(new ByteArrayInputStream(baos.toByteArray()));
            readAssert(in, "[X]");
            in.close();
        } catch (IOException e) {
            e.printStackTrace();
            throw e;
        }
    }
    static void write(ObjectOutput out, LSI<LSI<Map>> lamb) throws IOException {
        out.writeObject(lamb);
    }
    static void readAssert(ObjectInputStream in, String expected) throws IOException, ClassNotFoundException {
        Map result = ((LSI<LSI<Map>>) in.readObject()).get().get();
        System.out.printf("Result: %s\n", result);
    }
    interface LSI<T> extends Serializable {
        T get();
    }
}
