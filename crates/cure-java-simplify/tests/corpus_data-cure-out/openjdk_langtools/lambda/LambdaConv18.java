class LambdaConv18 {
    interface NonSAM {
        void m1();
        void m2();
    }
    NonSAM s1 = new NonSAM() { public void m1() {}
                              public void m2() {} };
    NonExistent s2 = new NonExistent() { public void m() {} };
}
