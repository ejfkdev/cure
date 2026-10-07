public class StringAppendAccessMethodOnLHS {
    private String s = "";
    class Inner {
        void m() {
            s += "a";
            s += 'a';
        }
    }
    void test() {
        new Inner().m();
    }
    public static void main(String[] args) throws Exception {
        StringAppendAccessMethodOnLHS o = new StringAppendAccessMethodOnLHS();
        o.test();
        if (!o.s.equals("aa")) 
            throw new Exception();
    }
}
