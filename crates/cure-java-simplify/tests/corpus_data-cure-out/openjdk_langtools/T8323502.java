public class T8323502 {
    public void m(Object o) {
        return switch (o) {
            default -> System.out.println("boom");
        };
    }
}
