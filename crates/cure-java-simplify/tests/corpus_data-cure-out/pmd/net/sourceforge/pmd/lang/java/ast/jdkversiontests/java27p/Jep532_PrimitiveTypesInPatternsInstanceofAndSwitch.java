import java.util.Map;

public class Jep532_PrimitiveTypesInPatternsInstanceofAndSwitch {
    void primitiveTypePatterns() {
        var json = new JsonObject(Map.of("name", new JsonString("John"), "age", new JsonNumber(30)));
        if (json instanceof JsonObject(var map) && map.get("name") instanceof JsonString(String n) && map.get("age") instanceof JsonNumber(int age)) {
            System.out.printf("Name: %s Age: %d%n", n, age);
        }
    }
    sealed interface JsonValue {
    }
    record JsonString(String s) implements JsonValue {
    }
    record JsonNumber(double d) implements JsonValue {
    }
    record JsonObject(Map<String, JsonValue> map) implements JsonValue {
    }
    void topLevelPrimitiveTypePatterns() {
        X x = new X();
        String status = switch (x.getStatus()) {
            case 0 -> "okay";
            case 1 -> "warning";
            case 2 -> "error";
            case int i -> "unknown status: " + i;
        };
        System.out.println("status: " + status);
        switch (x.getYearlyFlights()) {
            case 0 -> x.noop();
            case 1 -> x.noop();
            case 2 -> x.issueDiscount();
            case int i when i >= 100 -> x.issueGoldCard();
            case int i -> x.issueDiscount();
        }
        if (x.getPopulation() instanceof float p) {}
    }
    class X {
        int getStatus() {
            return 0;
        }
        int getYearlyFlights() {
            return 1;
        }
        void noop() {}
        void issueDiscount() {}
        void issueGoldCard() {}
        int getPopulation() {
            return 1;
        }
    }
    void instanceofWithPrimitives() {}
    void switchOnAllPrimitives() {
        User user = new User();
        User.startProcessing(User.OrderStatus.NEW, switch (user.isLoggedIn()) {
            case true -> user.id();
            case false -> {
                User.log("Unrecognized user");
                yield -1;
            }
        });
        {
            long v = (long) Math.random() * Long.MAX_VALUE;
            switch (v) {
                case 1L -> System.out.println("v is 1L");
                case 2L -> System.out.println("v is 2L");
                case 10_000_000_000L -> System.out.println("v is 10b");
                case 20_000_000_000L -> System.out.println("v is 20b");
                case long x -> System.out.println("v is " + x);
            }
        }
        {
            Byte b = 2;
            switch (b) {
                case int p -> {}
            }
        }
        {
            int x = 42;
            switch (x) {
                case int _ -> {}
            }
        }
        {
            float v = 1.1f;
            float result = switch (v) {
                case 0f -> 5f;
                case float x when x == 1f -> 6f + x;
                case float x -> 7f + x;
            };
        }
        switch (true) {
            case true -> {}
            case false -> {}
        }
        {
            int i = 1;
            switch (i) {
                case double d -> System.out.println("double: " + d);
            }
        }
    }
    class User {
        boolean isLoggedIn() {
            return false;
        }
        int id() {
            return 42;
        }
        static void startProcessing(OrderStatus status, int userId) {}
        static void log(String s) {}
        enum OrderStatus {
            NEW
        }
    }
    void instanceofAsThePreconditionForSafeCasting() {
        assert 42 instanceof int;
        assert 42 instanceof byte;
        assert !(1000 instanceof byte);
        {
            int i = 16_777_217;
            assert !(i instanceof float);
            assert i instanceof double;
            assert i instanceof Integer;
            assert i instanceof Number;
        }
        {
            float f = 1000.0f;
            assert !(f instanceof byte);
            assert f instanceof int;
            assert f instanceof double;
        }
        {
            double d = 1000.0d;
            assert !(d instanceof byte);
            assert d instanceof int;
            assert d instanceof float;
        }
        {
            Integer ii = 1000;
            assert ii instanceof int;
            assert ii instanceof float;
            assert ii instanceof double;
        }
        {
            Integer ii = 16_777_217;
            assert !(ii instanceof float);
            assert ii instanceof double;
        }
    }
}
