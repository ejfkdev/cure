public class Java9Identifier {
    public interface Lambda {
        public int a(int _);
        public default void t(Lambda l) {
            t((_) -> 0);
        }
    }
}
