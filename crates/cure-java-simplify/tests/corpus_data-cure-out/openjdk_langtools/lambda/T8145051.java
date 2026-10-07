public class T8145051 {
    public static void main(String[] args) {
        pkg.T8145051 t8145051 = new pkg.T8145051();
        t8145051.new Sub();
        if (!t8145051.s.equals("Executed lambda")) 
            throw new AssertionError("Unexpected data"); else 
            System.out.println("OK");
    }
}
