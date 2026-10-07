package spoon.test.comment.testclasses;

import java.util.ArrayList;

public class InlineComment extends ArrayList<String> {
    private int field = 10;
    static {}
    public InlineComment() {}
    public void m() {}
    public void m1() {
        switch (1) {
            case 0:
            case 1:
                int i = 0;
            default:
        }
        for (int i = 0; i < 10; i++) {}
        new InlineComment();
        this.m();
        int i = 0;
        do {
            i++;
        } while (i < 10);
        try {
            i++;
        } catch (Exception e) {}
        synchronized (this) {}
        Double dou = i == 1 ? null : new Double(2 / (double) (i - 1));
        int[] arr = {1, 2, 3};
    }
    public void m2(int i) throws Exception, Error {}
    public void m3() {
        m3();
    }
}
