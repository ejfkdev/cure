import java.util.function.Function;

public class EnumTypeChangesNonPreview {
    public static void main(String... args) throws Exception {
        new EnumTypeChangesNonPreview().run();
    }
    void run() throws Exception {
        doRunExhaustive(this::expressionEnumExhaustive);
    }
    void doRunExhaustive(Function<EnumTypeChangesEnum, String> c) throws Exception {
        try {
            c.apply(EnumTypeChangesEnum.valueOf("C"));
            throw new AssertionError();
        } catch (IncompatibleClassChangeError e) {}
    }
    String expressionEnumExhaustive(EnumTypeChangesEnum e) {
        return switch (e) {
            case A -> "A";
            case B -> "B";
        };
    }
}

enum EnumTypeChangesEnum {
    A, B
}
