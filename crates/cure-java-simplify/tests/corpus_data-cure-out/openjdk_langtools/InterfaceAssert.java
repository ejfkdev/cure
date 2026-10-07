interface I {
    class C {
        C() {}
        public void test(String s) {
            assert s == "Yup";
        }
    }
}

public class InterfaceAssert {
    public static void main(String[] args) {
        new I.C().test("Yup");
    }
}
