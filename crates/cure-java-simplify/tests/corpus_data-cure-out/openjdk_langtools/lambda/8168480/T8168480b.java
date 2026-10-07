import java.util.function.Supplier;

class T8168480b {
    Supplier<Runnable> ssr = () -> () -> {
    while (true) ;
    System.err.println("Hello");
};
}
