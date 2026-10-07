class BreakAcrossClass {
    public static void meth() {
        int i = 6;
        M:
            {
                class A {
                                {
                                    if (i != 5) break M;
                                }
                            }
                            System.out.println("failed : " + i);
            }
        System.out.println("passed : " + i);
    }
}
