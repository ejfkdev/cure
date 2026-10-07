package pack;

public class SuperClass {
    public static String sMessage = "Not OK";
    protected final void myDo() {
        message("OK!");
    }
    protected static final void myStaticDo() {
        sMessage = "OK!";
    }
    public void message(String s) {}
}
