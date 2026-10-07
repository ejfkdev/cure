public class InaccessibleMref02 {
    interface SAM {
        void m();
    }
    public static void main(String[] args) {
        SAM s = new p1.C()::m;
        s.m();
    }
}
