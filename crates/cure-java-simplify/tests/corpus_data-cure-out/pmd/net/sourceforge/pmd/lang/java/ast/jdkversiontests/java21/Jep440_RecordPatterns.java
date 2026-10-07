class Jep440_RecordPatterns {
    record Point(int x, int y) {
    }
    static void printSum(Object obj) {
        if (obj instanceof Point(int x, int y)) {
            System.out.println(x + y);
        }
    }
    enum Color {
        RED, GREEN, BLUE
    }
    record ColoredPoint(Point p, Color c) {
    }
    record Rectangle(ColoredPoint upperLeft, ColoredPoint lowerRight) {
    }
    static void printColorOfUpperLeftPoint(Rectangle r) {
        if (r instanceof Rectangle(ColoredPoint(Point p, Color c),
                                   ColoredPoint lr)) {
            System.out.println(c);
        }
    }
    static void printXCoordOfUpperLeftPointWithPatterns(Rectangle r) {
        if (r instanceof Rectangle(ColoredPoint(Point(var x, var y), var c),
                                   var lr)) {
            System.out.println("Upper-left corner: " + x);
        }
    }
    record Pair(Object x, Object y) {
    }
    static void patternsCanFailToMatch() {
        Pair p = new Pair(42, 42);
        if (p instanceof Pair(String s, String t)) {
            System.out.println(s + ", " + t);
        } else {
            System.out.println("Not a pair of strings");
        }
    }
    record MyPair<S,T>(S fst, T snd) {
    }
    static void recordInference(MyPair<String, Integer> pair) {
        switch (pair) {
            case MyPair(var f, var s) -> System.out.println("matched");
        }
    }
    record Box<T>(T t) {
    }
    static void test1(Box<Box<String>> bbs) {
        if (bbs instanceof Box<Box<String>>(Box(var s))) {
            System.out.println("String " + s);
        }
    }
    static void test2(Box<Box<String>> bbs) {
        if (bbs instanceof Box(Box(var s))) {
            System.out.println("String " + s);
        }
    }
}
