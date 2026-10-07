public class PrimitivePatternsSwitchConstants {
    void testConstExpressions() {
        switch (42) {
            case byte _:
        }
        switch (42l) {
            case byte _:
        }
        switch (123456) {
            case byte _:
        }
        switch (16_777_216) {
            case float _:
        }
        switch (16_777_217) {
            case float _:
        }
        switch (42d) {
            case float _:
        }
        switch (1) {
            case long _:
        }
        int i = 42;
        switch (i) {
            case long _:
        }
        switch (1) {
            case Long _:
        }
        switch (42) {
            case byte bb -> {}
            case int ii -> {}
        }
        switch (42) {
            case 42 -> {}
            case int ii -> {}
        }
        switch (42) {
            case (byte) 42 -> {}
            case int ii -> {}
        }
        switch (42) {
            case 42 -> {}
            default -> {}
        }
        switch (42) {
            default -> {}
            case 42 -> {}
        }
    }
}
