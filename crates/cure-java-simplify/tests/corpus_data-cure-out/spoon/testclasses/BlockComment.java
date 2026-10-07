package spoon.test.comment.testclasses;

public class BlockComment {
    private int field = 10;
    static {}
    public BlockComment() {}
    public void m() {}
    public void m1() {
        switch (1) {
            case 0:
            case 1:
                int i = 0;
            default:
        }
        for (int i = 0; i < 10; i++) {}
        new BlockComment();
        this.m();
        int i = 0;
        do {
            i++;
        } while (i < 10);
        try {
            i++;
        } catch (Exception e) {}
        synchronized (this) {}
    }
    public void m2(int i) throws Exception, Error {}
    public void m3() {
        m3();
    }
}
