import java.util.List;
import p.A;
import p.B;

public final class MethodArguments {
    public static void main(String[] args) {
        B.one("bar");
        B b = new B();
        b.one(null);
        b.two("foo");
        b.three("foo");
        b.four("foo");
    }
    void five(@A String s) {}
    void five(@A String s) {}
    void six(List<@A String> s) {}
    void six(List<@A String> s) {}
}
