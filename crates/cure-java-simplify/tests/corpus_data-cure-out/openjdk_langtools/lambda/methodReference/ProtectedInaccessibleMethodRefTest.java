import pack.SuperClass;
import java.util.concurrent.Callable;

public final class ProtectedInaccessibleMethodRefTest extends SuperClass {
    static String message = "NOT OK";
    public void doTest() throws Exception {
        new Callable<Void>() {
            @Override
            public Void call() throws Exception {
                final Runnable r = ProtectedInaccessibleMethodRefTest.this::myDo;
                r.run();
                return null;
            }
        }.call();
        new Callable<Void>() {
            @Override
            public Void call() throws Exception {
                final Runnable r = ProtectedInaccessibleMethodRefTest::myStaticDo;
                r.run();
                return null;
            }
        }.call();
    }
    public void message(String s) {
        message = s;
    }
    public static void main(String[] args) throws Exception {
        new ProtectedInaccessibleMethodRefTest().doTest();
        if (!message.equals("OK!")) 
            throw new AssertionError("Unexpected output");
        if (!sMessage.equals("OK!")) 
            throw new AssertionError("Unexpected output");
    }
}
