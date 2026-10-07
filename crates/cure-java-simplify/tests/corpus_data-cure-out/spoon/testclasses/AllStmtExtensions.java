package spoon.test.statementComment.testclasses;

import java.util.Scanner;

public class AllStmtExtensions {
    public AllStmtExtensions() {}
    void m1() {
        assert false;
        {
            int j = 10;
        }
    }
    void m2() {}
    void m3() {
        try {
            throw new Exception();
        } catch (Exception e) {
            System.out.println(e);
        }
        int r = 30;
        r++;
    }
    void m4() {
        System.out.println("Seems right...");
    }
    void m5() {
        int t = new Scanner(System.in).nextInt();
        switch (t) {
            case 1:
                System.out.println("1");
                break;
            default:
                System.out.println("None");
        }
    }
    void m6() {
        Object obj = new Object();
        synchronized (obj) {
            System.out.println("Executing");
        }
        for (int i = 0; i < 10; ++i) {
            System.out.println(i);
        }
    }
}
