public class T8305582 {
    record Point(int x, int y) {
    }
    enum Color {
        RED, GREEN, BLUE
    }
    record ColoredPoint(Point p, Color c) {
    }
    public static void foo(Object o) {
        switch (o) {
            case ColoredPoint(var (var x, var y), var c):
                break;
            default:
        }
    }
}
