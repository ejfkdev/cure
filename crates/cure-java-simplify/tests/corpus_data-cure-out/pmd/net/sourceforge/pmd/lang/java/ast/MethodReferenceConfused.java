import java.security.AccessController;
import java.security.PrivilegedAction;

public class MethodReferenceConfused {
    public void wrongVariableAccessor() {
        String result = AccessController.doPrivileged((PrivilegedAction<String>) ((I) null)::method);
    }
    interface I {
        String method();
    }
}
