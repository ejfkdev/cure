import java.util.function.Consumer;

class CantFindSymbolImplicitLambdaAndDiamondTest {
    static class B<T> {
    }
    static class A1 {
        <T> A1(Consumer<T> cons) {}
    }
    static class A2<T> {
        A2(Consumer<T> cons) {}
    }
    public void mount() {
        new A1((inHours) -> new B<>() {{
                    System.out.println(inHours);
                }});
        new A2<>((inHours) -> new B<>() {{
                System.out.println(inHours);
            }});
    }
}
