public class MethodReferenceComplexNullCheckTest {
    public static void main(String[] args) {
        boolean npeFired = false;
        try {} catch (NullPointerException npe) {
            npeFired = true;
        } finally {
            if (!npeFired) 
                throw new AssertionError("NPE should have been thrown");
        }
    }
    interface IForm {
        void xyz(Object... args);
    }
    class F {
        private void doit(Object... args) {}
    }
}
