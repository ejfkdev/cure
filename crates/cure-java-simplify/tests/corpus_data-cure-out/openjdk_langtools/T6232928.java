import java.io.*;
import java.lang.reflect.Modifier;

public class T6232928 {
    public static void main(String... args) throws Exception {
        int mod = Class.forName("T6232928.package-info").getModifiers();
        if (Modifier.isAbstract(mod) && Modifier.isInterface(mod)) {
            if ((mod & 0x1000) == 0) {
                throw new AssertionError("Test failed: interface package-info should be synthetic.");
            }
        } else {
            throw new AssertionError("Test failed: interface package-info should be abstract.");
        }
    }
}
