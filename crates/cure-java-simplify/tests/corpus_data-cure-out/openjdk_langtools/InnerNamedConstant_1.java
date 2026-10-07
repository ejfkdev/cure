public class InnerNamedConstant_1 {
    static class Inner1 {
        static final int x = 1;
        static final int y = x * 5;
        static final String z = "foobar";
    }
    class Inner2 {
        static final int x = 1;
        static final int y = x * 5;
        static final String z = "foobar";
    }
    public static void main(String[] args) {
        System.out.println(Inner1.x + " " + Inner1.y + " " + Inner1.z);
        Inner2 inner2 = new InnerNamedConstant_1().new Inner2();
        System.out.println(inner2.x + " " + inner2.y + " " + inner2.z);
    }
}
