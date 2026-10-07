class InnerMemberRegression {
    void test() {
        class Z {
                    class Y {
                        void test() {
                        }
                    }
                    Y y = new Y();
                    int m=100;

                    void test() {
                        y.test();
                    }
                }  //end of class Z
                Z z = new Z();
        z.test();
    }
    public  static void main(String[] s) {
        new InnerMemberRegression().test();
    }
}
