public class T8301025 {
    record TestRecord<T extends String>(T t) {
    }
    public static void main(String[] argv) {
        TestRecord r = new TestRecord("a");
        switch (r) {
            case TestRecord(String cS) -> {
                System.out.println("String");
            }
            case TestRecord(Object cO) -> {
                System.out.println("Object");
            }
        }
    }
}
